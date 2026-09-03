pragma ComponentBehavior: Bound
import QtQuick

// AS CAMADAS DE REALCE do documento aberto: Tree-sitter (sintatica) e semantic
// tokens do LSP (semantica).
//
// # Por que este arquivo existe, separado do EditorLanguageController
//
// Os dois nasceram do mesmo corte de 2026-09-02, e a catraca cobrou o primeiro
// deles ainda em 452 linhas. A pergunta da `ARCHITECTURE.md` §4 regra 9 — *"que
// responsabilidade esta misturada aqui?"* — tinha resposta clara: havia DUAS.
//
//   simbolo sob o CURSOR   pontual, posicional, abre popup ou dialogo
//                          (definition, hover, usos, code actions, rename)
//   realce do DOCUMENTO    continuo, versionado, alimenta a superficie
//                          (Tree-sitter + semantic tokens)               <- aqui
//
// Sao ritmos diferentes sobre o mesmo buffer: um responde a um gesto do
// usuario, o outro persegue cada tecla digitada. Junta-los foi o que fez o
// arquivo passar do limite mesmo depois de sair do `EditorController`.
//
// # Os dois relogios, e por que eles nao sao paranoia
//
// `syntaxVersion` e `semanticVersion` sobem a CADA pedido, e toda resposta e
// descartada se a versao nao bater — ou se o caminho do documento mudou. As
// duas camadas respondem em tempos diferentes sobre o mesmo texto: o
// Tree-sitter e local e rapido, o servidor LSP pode levar segundos. Aplicar "a
// ultima que chegou" pintaria realce de um texto que ja nao esta na tela.
//
// **Isto nao e precedencia por tempo de chegada; e identidade de
// documento/versao.** A composicao visual e outra coisa e vive no highlighter:
// regex fallback < Tree-sitter < semantic tokens < diagnosticos/busca.
//
// # Limpar e' obrigacao, nao gentileza
//
// Sem documento (ou com o editor ainda nao pronto), o realce anterior e
// APAGADO. Token de um texto que nao esta mais aberto e realce errado, e
// errado e pior que ausente — o usuario confia na cor.
Item {
    id: root

    property var surfaceBridge: null
    property var documentController: null
    // A superficie de texto (C++), dona de `setSemanticTokens` e
    // `setSyntaxSnapshot`. Sem ela, este controller so mantem versao.
    property var editorSurface: null

    property int syntaxVersion: 0
    property int semanticVersion: 0
    property string syntaxLanguage: "plain"
    property bool syntaxHasErrors: false
    // Estrutura do documento, consumida pelo painel de outline.
    property var syntaxOutline: []
    // Indice sintatico LOCAL, que alimenta o completion enquanto o servidor
    // nao respondeu. Nunca e promovido a referencia semantica.
    property var syntaxLocals: []

    signal semanticTokensRequested(string path, string content, int version)
    signal syntaxTreeRequested(string path, string content, int version)

    visible: false

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null && editorSurface !== null;
    }

    function currentPath() {
        return documentController === null ? "" : documentController.currentFilePath();
    }

    function refreshSemanticTokens() {
        const path = currentPath();
        semanticVersion++;
        if (path === "" || !ready()) {
            if (ready()) {
                editorSurface.clearSemanticTokens();
            }
            return;
        }
        semanticTokensRequested(path, surfaceBridge.text(), semanticVersion);
    }

    function handleSemanticTokensResolved(path, version, tokens) {
        if (path !== currentPath() || Number(version) !== semanticVersion || !ready()) {
            return;
        }
        editorSurface.setSemanticTokens(tokens);
    }

    function refreshSyntaxTree() {
        const path = currentPath();
        syntaxVersion++;
        if (path === "" || !ready()) {
            resetSyntaxState();
            if (ready()) {
                editorSurface.setSyntaxSnapshot([], []);
            }
            return;
        }
        syntaxTreeRequested(path, surfaceBridge.text(), syntaxVersion);
    }

    function handleSyntaxTreeResolved(path, version, language, hasErrors,
                                      highlights, foldingRanges, outline, locals) {
        if (path !== currentPath() || Number(version) !== syntaxVersion || !ready()) {
            return;
        }
        syntaxLanguage = language;
        syntaxHasErrors = hasErrors;
        syntaxOutline = outline;
        syntaxLocals = locals;
        editorSurface.setSyntaxSnapshot(highlights, foldingRanges);
    }

    // O texto MUDOU: invalida na hora o que estava em voo e apaga o realce
    // semantico, sem esperar resposta nenhuma.
    //
    // Por que apagar em vez de deixar o antigo: token de um texto que acabou de
    // mudar aponta para posicoes que ja nao existem — sublinha a palavra
    // errada. O Tree-sitter continua como fallback estrutural (ele reparseia
    // localmente em milissegundos), entao a tela nao fica sem cor; o que sai e
    // so a camada que depende do servidor.
    function invalidateForEdit() {
        semanticVersion++;
        syntaxVersion++;
        if (ready()) {
            editorSurface.clearSemanticTokens();
        }
        scheduleSyntaxRefresh();
    }

    // Digitacao normal: reparse depois que a edicao para. O intervalo curto
    // (180 ms) e proposital — o Tree-sitter e incremental e local, entao o
    // realce sintatico pode acompanhar a digitacao sem esperar o servidor.
    function scheduleSyntaxRefresh() {
        syntaxDebounce.restart();
    }

    function resetSyntaxState() {
        syntaxLanguage = "plain";
        syntaxHasErrors = false;
        syntaxOutline = [];
        syntaxLocals = [];
    }

    // Workspace trocou: a versao avanca para invalidar qualquer resposta em
    // voo do projeto anterior, e o estado visivel zera.
    function clear() {
        syntaxVersion++;
        resetSyntaxState();
    }

    Timer {
        id: syntaxDebounce

        interval: 180
        repeat: false
        onTriggered: root.refreshSyntaxTree()
    }
}
