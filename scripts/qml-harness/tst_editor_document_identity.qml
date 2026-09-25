import QtQuick
// Carrega o EditorDocumentController REAL (arquivo do projeto, sem copia).
import "../../ui/qml/editor"

// A ABA E' UM DOCUMENTO, NAO UMA POSICAO (fatia V5, 2026-09-25).
//
// O aceite da V5 no roadmap 47: "nenhuma operacao de dominio depende da posicao
// visual da aba", e "harness remove e prova que o documento errado nao recebe
// acao". Estes eram os defeitos, os dois medidos no codigo antes da fatia:
//
//   1. fechar uma aba DE FUNDO trocava o documento na tela, porque o proximo
//      selecionado era `Math.min(index, count - 1)` — a posicao de quem saiu;
//   2. fechar qualquer aba passava por `selectTab` com `currentTab` ja'
//      invalidado (-1), e o `storeCurrentEditor` de dentro dele descartava o
//      texto que estava na superficie.
//
// Prova tambem que renomear NAO troca a identidade: o `path` muda, o documento
// continua o mesmo — que e' por isso que o id e' sintetico e nao e' o caminho.
Item {
    id: root

    property int failures: 0
    property string textOnScreen: ""
    property string pathOnScreen: ""

    QtObject {
        id: fakeSurface

        property int cursorPosition: 0
    }

    QtObject {
        id: bridge

        property var editorSurface: fakeSurface
        function ready() { return true; }
        function text() { return root.textOnScreen; }
        function setText(value) { root.textOnScreen = value; }
        function setPath(value) { root.pathOnScreen = value; }
    }

    EditorDocumentController {
        id: documents

        workspaceRoot: "/tmp/p"
        surfaceBridge: bridge
    }

    function openFile(caminho, conteudo) {
        documents.handleFileLoaded(caminho, conteudo);
        return documents.currentDocId;
    }

    function check(ok, bit, message) {
        if (!ok) {
            root.failures += bit;
            console.error(message);
        }
    }

    Component.onCompleted: {
        const a = openFile("/tmp/p/a.rs", "conteudo A");
        const b = openFile("/tmp/p/b.rs", "conteudo B");
        const c = openFile("/tmp/p/sub/c.rs", "conteudo C");

        // IDENTIDADE: tres documentos, tres ids distintos, nenhum deles zero.
        check(a !== b && b !== c && a !== c, 1, "ids repetidos: " + [a, b, c]);
        check(a > 0 && b > 0 && c > 0, 2, "id zero e' 'nenhum documento'");
        check(documents.currentDocId === c, 4, "openFile seleciona o que abriu");
        check(documents.currentTab === 2, 8, "o indice segue o documento");

        // FECHAR UMA ABA DE FUNDO NAO TROCA O DOCUMENTO DA TELA. Este era o
        // defeito 1: fechava-se a primeira aba e a tela pulava para outra.
        root.textOnScreen = "conteudo C editado";
        documents.closeDocument(a);
        check(documents.currentDocId === c, 16,
              "fechar aba de fundo trocou o documento: " + documents.currentDocId);
        check(documents.currentTab === 1, 32,
              "o indice do documento atual devia cair para 1, deu " + documents.currentTab);
        check(root.pathOnScreen === "/tmp/p/sub/c.rs", 64,
              "a superficie trocou de arquivo: " + root.pathOnScreen);

        // E O TEXTO DA TELA NAO E' DESCARTADO. Defeito 2: o buffer atual ia
        // para o modelo com o indice ja' invalidado.
        check(root.textOnScreen === "conteudo C editado", 128,
              "o texto na tela foi trocado: " + root.textOnScreen);
        documents.selectDocument(b);
        documents.selectDocument(c);
        check(root.textOnScreen === "conteudo C editado", 256,
              "o buffer nao sobreviveu a ida e volta: " + root.textOnScreen);

        // RENOMEAR NAO TROCA A IDENTIDADE: o caminho muda, o documento fica.
        documents.applyPathRenameToTabs("/tmp/p/sub", "/tmp/p/outro");
        check(documents.currentDocId === c, 512, "renomear trocou o documento");
        check(documents.pathOfDocument(c) === "/tmp/p/outro/c.rs", 1024,
              "o caminho nao acompanhou: " + documents.pathOfDocument(c));

        // FECHAR VARIAS SOB UM CAMINHO: o laco por indice fechava a aba errada
        // quando duas vizinhas caiam sob o mesmo prefixo. Aqui so' `c` cai.
        const d = openFile("/tmp/p/outro/d.rs", "conteudo D");
        documents.selectDocument(b);
        documents.closeTabsUnderPath("/tmp/p/outro");
        check(documents.currentDocId === b, 2048,
              "fechar por caminho levou junto o documento atual");
        check(documents.filesModel.count === 1, 4096,
              "sobrou aba a mais ou de menos: " + documents.filesModel.count);
        check(documents.pathOfDocument(b) === "/tmp/p/b.rs", 8192, "sobrou o errado");
        check(documents.pathOfDocument(c) === "" && documents.pathOfDocument(d) === "",
              16384, "documento fechado ainda responde");

        // ID DE DOCUMENTO QUE NAO EXISTE: resultado observavel, sem estrago.
        const before = documents.currentDocId;
        documents.closeDocument(99999);
        documents.selectDocument(99999);
        check(documents.currentDocId === before, 32768,
              "id inexistente mexeu no documento atual");
        check(documents.filesModel.count === 1, 65536, "id inexistente fechou alguma aba");

        // FECHAR O ULTIMO: nao sobra documento, e a tela esvazia.
        documents.closeDocument(b);
        check(documents.currentDocId === 0, 131072, "sem abas, nao ha' documento atual");
        check(documents.currentTab === -1, 262144, "sem abas, o indice e' -1");
        check(root.textOnScreen === "" && root.pathOnScreen === "", 524288,
              "a superficie ficou com o arquivo fechado");

        // ID NUNCA E' REAPROVEITADO: reabrir o mesmo caminho da' documento novo.
        const bDeNovo = openFile("/tmp/p/b.rs", "conteudo B");
        check(bDeNovo !== b, 1048576, "o id do documento fechado voltou");

        if (root.failures !== 0) console.error("FALHAS bitmask=" + root.failures);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
