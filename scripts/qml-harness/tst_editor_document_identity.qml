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

    // `real`, nao `int`: os bits passam de 2^31 e um int os perdia em silencio.
    property real failures: 0
    property string textOnScreen: ""
    property string pathOnScreen: ""
    property int writes: 0
    property var externalReads: []
    // Imita a superficie real: trocar de aba re-realca, e o realce emite
    // `textChanged` com o texto que AINDA esta' na tela (ver o caso no fim).
    property bool echoOnSwitch: false

    QtObject {
        id: fakeSurface

        property int cursorPosition: 0
    }

    QtObject {
        id: bridge

        property var editorSurface: fakeSurface
        property bool loadingText: false
        function ready() { return true; }
        function text() { return root.textOnScreen; }
        function setText(value) { root.textOnScreen = value; }
        function setPath(value) { root.pathOnScreen = value; }
    }

    EditorDocumentController {
        id: documents

        workspaceRoot: "/tmp/p"
        surfaceBridge: bridge
        onWriteFileRequested: root.writes++
    }

    Connections {
        target: documents

        function onCurrentTabChanged() {
            if (root.echoOnSwitch && !bridge.loadingText) {
                documents.markCurrentModified(root.textOnScreen);
            }
        }
    }

    EditorExternalPreviewController {
        id: externalPreview

        workspaceRoot: "/tmp/p"
        documentController: documents
        onReadFileRequested: path => root.externalReads.push(path)
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
        check(documents.hasUnsavedUnderPath("/tmp/p/sub"), 2097152,
              "pasta com buffer alterado pode ser excluida");
        check(!documents.hasUnsavedUnderPath("/tmp/p/submarine"), 4194304,
              "prefixo parecido foi tratado como ancestral");
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
        check(documents.pathOfDocument(c) !== "" && documents.pathOfDocument(d) !== "",
              8388608, "resposta tardia de exclusao fechou buffer alterado");
        documents.filesModel.setProperty(1, "content", "conteudo C");
        documents.filesModel.setProperty(1, "modified", false);
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

        // A aba externa compartilha a identidade do editor, mas nunca entra
        // no fluxo de edição nem pode emitir fs.write.
        const externo = "/tmp/fora/arquivo.txt";
        check(externalPreview.open(externo) && root.externalReads[0] === externo,
              1073741824, "drop externo nao pediu leitura explicita");
        externalPreview.handleLoaded(externo, "externo original");
        const externoId = documents.currentDocId;
        check(documents.currentReadOnly && root.textOnScreen === "externo original",
              16777216, "aba externa nao abriu em somente leitura");
        check(!documents.markCurrentModified("alteracao"), 33554432,
              "aba externa aceitou marcar edicao");
        root.textOnScreen = "tentativa de alterar";
        documents.saveCurrentFile();
        documents.selectDocument(bDeNovo);
        documents.selectDocument(externoId);
        check(root.writes === 0 && root.textOnScreen === "externo original",
              67108864, "aba externa escreveu ou guardou alteracao");
        check(documents.modifiedDocuments().length === 0, 134217728,
              "aba externa entrou no salvar tudo");
        externalPreview.open(externo);
        externalPreview.handleLoaded(externo, "externo atualizado");
        check(documents.currentDocId === externoId
              && root.textOnScreen === "externo atualizado", 268435456,
              "novo drop duplicou aba ou nao atualizou a previa");
        documents.selectDocument(bDeNovo);
        check(!documents.currentReadOnly, 536870912,
              "aba interna herdou o bloqueio de escrita");
        externalPreview.open("/tmp/fora/falha.txt");
        externalPreview.handleFailed("/tmp/fora/falha.txt", "negado");
        check(externalPreview.errorMessage.indexOf("negado") >= 0,
              2147483648, "falha da leitura externa ficou invisivel");
        externalPreview.open("/tmp/fora/atrasado.txt");
        externalPreview.workspaceRoot = "/tmp/outro";
        externalPreview.handleLoaded("/tmp/fora/atrasado.txt", "stale");
        check(documents.filesModel.count === 2, 4294967296,
              "resposta atrasada criou aba depois de trocar workspace");

        // TROCAR DE DOCUMENTO NAO E' EDITAR (2026-10-01). No drop real de um
        // arquivo externo, mudar `currentDocId` disparava diagnosticos ->
        // realce -> `textChanged` com o texto do documento ANTERIOR, antes de
        // `currentReadOnly` e do texto novo: a aba nascia com o ponto de
        // "modificado". O mesmo valia para ir de uma aba a outra.
        root.echoOnSwitch = true;
        documents.selectDocument(externoId);
        check(documents.filesModel.get(documents.currentTab).modified !== true,
              8589934592, "trocar para a aba externa a marcou como modificada");
        documents.selectDocument(bDeNovo);
        check(documents.modifiedDocuments().length === 0, 17179869184,
              "trocar de aba marcou o documento novo como modificado");
        root.echoOnSwitch = false;

        if (root.failures !== 0) console.error("FALHAS bitmask=" + root.failures);
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
