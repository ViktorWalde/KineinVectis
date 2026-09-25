import QtQuick
// Carrega o MarkdownPreviewController REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// O MODO DA PREVIA DE MARKDOWN (fatia V5/M1, 2026-09-25).
//
// Tres regras da especificacao, e todas sao faceis de quebrar sem perceber:
//
//   §3.1  o modo e' lembrado POR ARQUIVO durante a sessao;
//   §11   o padrao e' Editar — a IDE e' um editor, e abrir um `.md` nao pode
//         esconder o texto de quem veio edita-lo;
//   §3.1  so' `.md` e `.markdown` oferecem o modo.
//
// A identidade da V5 e' o que torna "por arquivo" dizivel sem ambiguidade: o
// modo e' guardado por DOCUMENTO, entao fechar uma aba de fundo nao mexe no
// modo de ninguem, e reabrir um arquivo comeca de novo em Editar.
Item {
    id: root

    property int failures: 0
    property int docId: 0
    property string path: ""

    QtObject {
        id: fakeEditor

        property int currentDocId: root.docId
        function currentFilePath() { return root.path; }
    }

    MarkdownPreviewController {
        id: preview

        editorController: fakeEditor
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    Component.onCompleted: {
        // SO' MARKDOWN oferece o modo. Um `.py` nao ganha um botao que nao faz
        // nada — anunciar acao inexistente e' o defeito que a V3 nomeou.
        root.docId = 1;
        root.path = "/p/src/main.py";
        check(!preview.available, 1, "um .py nao tem previa");
        root.path = "/p/docs/guia.md";
        check(preview.available, 2, "um .md tem previa");
        root.path = "/p/docs/LEIAME.MARKDOWN";
        check(preview.available, 4, ".markdown tambem, e a caixa nao decide");
        root.path = "/p/docs/notas.md.bak";
        check(!preview.available, 8, ".md no meio do nome nao conta");

        // O PADRAO E' EDITAR (§11), e vale para documento nunca visto.
        root.path = "/p/docs/guia.md";
        check(preview.mode === "edit", 16, "padrao: " + preview.mode);

        // O MODO E' DO DOCUMENTO. Trocar de aba leva o modo junto, e voltar
        // encontra o modo de antes.
        check(preview.setMode("preview"), 32, "trocar de modo no documento 1");
        check(preview.mode === "preview", 64, "documento 1 em preview");
        root.docId = 2;
        root.path = "/p/docs/outro.md";
        check(preview.mode === "edit", 128,
              "documento 2 herdou o modo do 1: " + preview.mode);
        root.docId = 1;
        root.path = "/p/docs/guia.md";
        check(preview.mode === "preview", 256, "documento 1 perdeu o modo");

        // SEM DOCUMENTO nao ha' modo para guardar, e pedir nao estoura.
        root.docId = 0;
        root.path = "";
        check(!preview.available, 512, "sem arquivo, sem previa");
        check(preview.setMode("preview") === false, 1024,
              "sem documento, trocar de modo devolve false");
        check(preview.mode === "edit", 2048, "sem documento o modo e' o padrao");

        // TROCAR DE PROJETO esquece tudo: os ids da sessao anterior nao voltam,
        // e um modo guardado para o id 1 nao pode pintar o novo documento 1.
        root.docId = 1;
        root.path = "/p/docs/guia.md";
        check(preview.mode === "preview", 4096, "antes do reset, o modo esta' la'");
        preview.reset();
        check(preview.mode === "edit", 8192, "depois do reset, volta ao padrao");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
