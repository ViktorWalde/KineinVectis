import QtQuick
// Carrega o MarkdownRenderGate REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// O DEBOUNCE DA PREVIA, E A GUARDA POR DOCUMENTO (V5/M2, 2026-09-25).
//
// §7: "renderizacao nao roda no caminho sincrono do TextEdit de codigo". O
// renderer do Qt reconstroi o documento inteiro a cada `setMarkdown`, no thread
// da UI — ligar o buffer direto nele fazia um documento inteiro ser remontado a
// cada caractere digitado.
//
// §6 e §10: "`documentId` evita que debounce de uma aba atualize outra" e
// "resposta atrasada nunca aparece na aba errada". E' o mesmo principio da
// identidade da V5, uma camada adiante: o que espera 150 ms pode ser de outro
// documento quando o prazo vence.
//
// A regra mora separada do widget de proposito: o espelho plano do harness nao
// tem os tipos registrados em C++, entao um componente que toque neles fica sem
// teste. Separando, ela volta a ser testavel.
Item {
    id: root

    property int failures: 0

    MarkdownRenderGate {
        id: preview

        docId: 1
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    Component.onCompleted: {
        // ABRIR NAO ESPERA: previa em branco por 150 ms ao abrir pareceria
        // defeito. O prazo e' para digitacao, nao para troca de documento.
        preview.content = "# um";
        check(preview.rendered === "# um", 1,
              "abrir deveria renderizar na hora: " + preview.rendered);
        check(preview.renderedDocId === 1, 2, "o render e' do documento 1");

        // DIGITAR ESPERA: o texto novo NAO chega ao renderer na hora.
        preview.content = "# um dois";
        check(preview.rendered === "# um", 4,
              "digitar nao pode renderizar na hora: " + preview.rendered);

        // TROCAR DE DOCUMENTO no meio da espera. A previa do anterior sai da
        // tela na hora — deixa-la seria mostrar um documento com o nome de
        // outro.
        preview.docId = 2;
        check(preview.rendered === "", 8,
              "a previa do documento 1 ficou na tela: " + preview.rendered);
        check(preview.renderedDocId === 2, 16, "o render passou a ser do 2");

        // E o texto do documento 2 entra na hora, sem esperar o prazo.
        preview.content = "# outro";
        check(preview.rendered === "# outro", 32,
              "o documento novo deveria renderizar na hora: " + preview.rendered);

        wait1.start();
    }

    // O prazo armado pelo documento 1 foi CANCELADO na troca: o texto dele nao
    // pode pintar na aba do documento 2, e o cancelamento e' o que garante.
    Timer {
        id: wait1

        interval: 400
        onTriggered: {
            check(preview.rendered === "# outro", 64,
                  "o texto do documento 1 ressuscitou: " + preview.rendered);
            check(preview.renderedDocId === 2, 128, "o render continua do 2");

            // Uma rajada de digitacao vira UM render, nao cinco.
            preview.content = "a";
            preview.content = "ab";
            preview.content = "abc";
            check(preview.rendered === "# outro", 256,
                  "a rajada nao pode renderizar durante o prazo");
            wait2.start();
        }
    }

    Timer {
        id: wait2

        interval: 400
        onTriggered: {
            check(preview.rendered === "abc", 512,
                  "depois do prazo entra o ULTIMO texto: " + preview.rendered);
            wait3.start();
        }
    }

    // DOIS DOCUMENTOS COM O MESMO TEXTO: `onContentChanged` nao dispara, porque
    // a string nao mudou. Sem o prazo curto do `settle`, a previa ficaria em
    // branco para sempre.
    Timer {
        id: wait3

        interval: 100
        onTriggered: {
            preview.docId = 3;
            check(preview.rendered === "", 1024, "trocar de documento zera a previa");
            // Nada e' atribuido a `content`: ele ja' e' "abc", igual ao do
            // documento anterior.
            wait4.start();
        }
    }

    Timer {
        id: wait4

        interval: 300
        onTriggered: {
            check(preview.rendered === "abc", 2048,
                  "texto identico deixou a previa em branco: [" + preview.rendered + "]");
            check(preview.renderedDocId === 3, 4096, "e o render e' do documento 3");
            Qt.exit(root.failures === 0 ? 0 : 1);
        }
    }
}
