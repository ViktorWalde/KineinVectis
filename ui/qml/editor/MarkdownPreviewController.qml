import QtQuick

// O ESTADO DE APRESENTACAO DA PREVIA DE MARKDOWN (fatia V5/M1, 2026-09-25).
//
// A §6 da `especificacoes/markdown-preview-0.3.md` separa o que e' documento
// do que e' apresentacao, e poe o modo do lado da apresentacao, junto de scroll
// e splitter. Por isso ele NAO mora no dono da identidade: fechar e reabrir um
// arquivo comeca de novo em Editar, e isso esta' certo.
//
// O modo e' guardado POR DOCUMENTO, e nao por aba nem por caminho: e' o que a
// §3.1 pede ("lembrado por arquivo durante a sessao") e o que a identidade da
// V5 torna possivel dizer sem ambiguidade. Fechar uma aba de fundo nao muda o
// modo de ninguem.
//
// O padrao e' "edit", que e' a recomendacao da §11: a IDE e' um editor, e abrir
// um `.md` nao pode esconder o texto de quem veio edita-lo.
QtObject {
    id: root

    property var editorController: null

    readonly property PathRules rules: PathRules {}

    property var modeByDocId: ({})

    readonly property int currentDocId: editorController === null
                                        ? 0 : editorController.currentDocId

    // `.md`/`.markdown` e mais nada (§3.1). Um `.py` nao ganha um botao
    // "Preview" que nao faz nada.
    readonly property bool available: editorController !== null
                                      && rules.isMarkdown(editorController.currentFilePath())

    readonly property string mode: {
        const guardado = root.modeByDocId[root.currentDocId];
        return guardado === undefined ? "edit" : guardado;
    }

    // A LARGURA do lado a lado tambem e' apresentacao (§6) — mas, ao contrario
    // do modo, e' UMA SO' para a janela: o autor ajusta a divisao uma vez, e
    // nao por arquivo.
    readonly property real defaultSplitWidth: 420
    readonly property real minSplitWidth: 280

    property real splitWidth: defaultSplitWidth

    // Delta zero e' o gesto de RESETAR do splitter (duplo clique).
    function resizeSplit(delta) {
        if (delta === 0) {
            root.splitWidth = root.defaultSplitWidth;
            return;
        }
        root.splitWidth = Math.max(root.minSplitWidth, root.splitWidth + delta);
    }

    function setMode(mode) {
        if (root.currentDocId <= 0) {
            return false;
        }
        const modos = ({});
        for (const chave in root.modeByDocId) {
            modos[chave] = root.modeByDocId[chave];
        }
        modos[root.currentDocId] = mode;
        root.modeByDocId = modos;
        return true;
    }

    // Trocar de projeto esquece tudo: os ids da sessao anterior nao voltam.
    function reset() {
        root.modeByDocId = ({});
    }
}
