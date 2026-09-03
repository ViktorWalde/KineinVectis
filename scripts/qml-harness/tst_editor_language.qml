// Camada de linguagem e camada de realce do editor, depois do corte de
// 2026-09-02 (roadmap 30, etapa 6).
//
// Por que existe: o `EditorController` tinha 1.070 linhas e implementava a
// inteligencia de linguagem INLINE. Ao mover isso para donos proprios
// (`EditorLanguageController` e `EditorHighlightController`), o comportamento
// atravessou uma fronteira nova — e refatoracao sem teste e' mudanca com os
// olhos fechados. Nenhum dos dois arquivos tinha teste; agora tem.
//
// O alvo mais afiado sao OS DOIS RELOGIOS. `syntaxVersion` e `semanticVersion`
// existem para descartar resposta obsoleta: as duas camadas respondem em tempos
// diferentes sobre o mesmo buffer, e aplicar "a ultima que chegou" pintaria
// realce de um texto que ja nao esta na tela. Isso nao quebra build, nao acende
// qmllint e so aparece como cor errada — a definicao de falha silenciosa.
import QtQuick
import "../../ui/qml/editor"

Item {
    id: root

    // --- Dublês --------------------------------------------------------
    property string caminhoAtual: "/tmp/projeto/main.cpp"
    property var tokensAplicados: null
    property var snapshotAplicado: null
    property int limpezasDeToken: 0
    property var pedidos: []

    QtObject {
        id: superficieFalsa

        property string text: "int main() {}"
        property int cursorPosition: 0

        function setSemanticTokens(tokens) { root.tokensAplicados = tokens; }
        function clearSemanticTokens() { root.limpezasDeToken += 1; }
        function setSyntaxSnapshot(highlights, folding) {
            root.snapshotAplicado = { highlights: highlights, folding: folding };
        }
    }

    QtObject {
        id: pontefalsa

        function ready() { return true; }
        function text() { return superficieFalsa.text; }
        function focusEditor() {}
    }

    ListModel {
        id: abasFalsas
    }

    QtObject {
        id: documentosFalsos

        property var filesModel: abasFalsas
        property int currentTab: 0

        function currentFilePath() { return root.caminhoAtual; }
        function relativeToRoot(path) { return path.replace("/tmp/projeto/", ""); }
        function openDiagnostic(path, line, column) {
            root.pedidos.push({ tipo: "abrir", path: path, line: line });
        }
        function handleRenameApplied(files) { root.pedidos.push({ tipo: "renomeado" }); }
    }

    QtObject {
        id: completionFalso

        function dismiss() { root.pedidos.push({ tipo: "completionDismiss" }); }
    }

    QtObject {
        id: textoFalso

        function cursorLineColumn() { return { line: 7, column: 3 }; }
        function currentWord() { return "simbolo"; }
    }

    EditorHighlightController {
        id: highlight

        surfaceBridge: pontefalsa
        documentController: documentosFalsos
        editorSurface: superficieFalsa
        onSemanticTokensRequested: function (path, content, version) {
            root.pedidos.push({ tipo: "semantic", version: version });
        }
        onSyntaxTreeRequested: function (path, content, version) {
            root.pedidos.push({ tipo: "syntax", version: version });
        }
    }

    EditorLanguageController {
        id: language

        surfaceBridge: pontefalsa
        documentController: documentosFalsos
        textController: textoFalso
        completionController: completionFalso
        editorSurface: superficieFalsa
        onDefinitionRequested: function (path, content, line, column) {
            root.pedidos.push({ tipo: "definition", line: line, column: column });
        }
        onRenameRequested: function (path, content, line, column, newName) {
            root.pedidos.push({ tipo: "rename", nome: newName });
        }
        onWorkspaceEditApplyRequested: function (id) {
            root.pedidos.push({ tipo: "weApply", id: id });
        }
    }

    function pedidosDe(tipo) {
        return root.pedidos.filter(function (p) { return p.tipo === tipo; });
    }

    Component.onCompleted: {
        let failures = 0;

        // ---- Realce: os dois relogios --------------------------------------
        highlight.refreshSemanticTokens();
        const versaoSemantica = highlight.semanticVersion;
        // Resposta com a versao ATUAL: aplica.
        highlight.handleSemanticTokensResolved(root.caminhoAtual, versaoSemantica, ["ok"]);
        if (root.tokensAplicados === null) failures += 1;

        // Resposta ATRASADA (versao anterior): tem que ser DESCARTADA.
        root.tokensAplicados = null;
        highlight.handleSemanticTokensResolved(root.caminhoAtual, versaoSemantica - 1, ["velho"]);
        if (root.tokensAplicados !== null) failures += 2;

        // Resposta de OUTRO arquivo, com a versao certa: tambem descartada.
        highlight.handleSemanticTokensResolved("/tmp/projeto/outro.cpp",
                                               highlight.semanticVersion, ["outro"]);
        if (root.tokensAplicados !== null) failures += 4;

        // Sem documento aberto, o realce anterior e APAGADO — cor de um texto
        // que nao esta mais na tela e pior que ausencia de cor.
        root.caminhoAtual = "";
        const limpezasAntes = root.limpezasDeToken;
        highlight.refreshSemanticTokens();
        if (root.limpezasDeToken !== limpezasAntes + 1) failures += 8;
        highlight.refreshSyntaxTree();
        if (highlight.syntaxLanguage !== "plain" || highlight.syntaxOutline.length !== 0) {
            failures += 16;
        }
        root.caminhoAtual = "/tmp/projeto/main.cpp";

        // A arvore sintatica com a versao certa entrega outline e locals.
        highlight.refreshSyntaxTree();
        highlight.handleSyntaxTreeResolved(root.caminhoAtual, highlight.syntaxVersion,
                                           "cpp", false, ["h"], ["f"], ["out"], ["loc"]);
        if (highlight.syntaxLanguage !== "cpp") failures += 32;
        if (highlight.syntaxOutline.length !== 1 || highlight.syntaxLocals.length !== 1) {
            failures += 64;
        }
        if (root.snapshotAplicado === null) failures += 128;

        // ---- Linguagem: pedidos posicionais --------------------------------
        language.hoverVisible = true;
        language.requestDefinition();
        const definicoes = root.pedidosDe("definition");
        if (definicoes.length !== 1 || definicoes[0].line !== 7) failures += 256;
        // Pedir definicao ESCONDE o hover: dois popups sobre o mesmo simbolo
        // ao mesmo tempo e ruido.
        if (language.hoverVisible) failures += 512;

        // Hover vazio nao acende popup vazio.
        language.handleHoverResolved("   ");
        if (language.hoverVisible) failures += 1024;
        language.handleHoverResolved("int main()");
        if (!language.hoverVisible || language.hoverText !== "int main()") failures += 2048;

        // ---- Code actions: o indice nunca sai da lista ---------------------
        language.handleCodeActionsResolved([{ title: "a" }, { title: "b", kind: "quickfix" }]);
        if (!language.actionsVisible || language.actionsModel.count !== 2) failures += 4096;
        language.moveActions(-5);
        if (language.actionsIndex !== 0) failures += 8192;
        language.moveActions(99);
        if (language.actionsIndex !== 1) failures += 16384;
        language.dismissActions();
        if (language.actionsVisible || language.actionsModel.count !== 0) failures += 32768;

        // ---- Rename: recusa com OUTRA aba suja -----------------------------
        abasFalsas.append({ path: "/tmp/projeto/main.cpp", modified: false });
        abasFalsas.append({ path: "/tmp/projeto/outro.cpp", modified: true });
        language.confirmRename("novoNome");
        if (root.pedidosDe("rename").length !== 0) failures += 65536;
        if (language.renameError === "") failures += 131072;

        // Com as outras limpas, o pedido sai.
        abasFalsas.setProperty(1, "modified", false);
        language.confirmRename("novoNome");
        if (root.pedidosDe("rename").length !== 1) failures += 262144;

        // ---- WorkspaceEdit: sem transacao, nada e enviado ------------------
        language.applyWorkspaceEdit();
        if (root.pedidosDe("weApply").length !== 0) failures += 524288;
        language.handleWorkspaceEditPreview("tx-1", "Renomear", ["a.cpp"], 3);
        if (!language.workspaceEditPreviewVisible || language.workspaceEditCount !== 3) {
            failures += 1048576;
        }
        language.applyWorkspaceEdit();
        if (root.pedidosDe("weApply").length !== 1) failures += 2097152;

        // ---- A recusa do core vira estado de tela --------------------------
        language.handleRequestFailed("lsp.rename", "conflito");
        if (language.renameError !== "conflito" || !language.renameDialogVisible) {
            failures += 4194304;
        }
        // Erro de outro dominio nao mexe aqui.
        language.handleRequestFailed("git.status", "nada a ver");
        if (language.renameError !== "conflito") failures += 8388608;

        // ---- clear(): trocar de workspace apaga o que esta aceso -----------
        language.clear();
        if (language.hoverVisible || language.actionsVisible
                || language.renameDialogVisible || language.workspaceEditPreviewVisible) {
            failures += 16777216;
        }

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
