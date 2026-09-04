// A CAIXA DA PREVIA NAO PODE SUMIR.
//
// Por que existe (2026-09-04): o painel de Configuration Actions empilha
// titulo, campos, previa e rodape. Titulo e rodape sao ancorados; a previa
// ficava com O QUE SOBRASSE. Quando os campos ganharam descricao e chips de
// sugestao, sobrou menos — e numa tela 1366x768, com uma acao de seis campos,
// a caixa que mostra o CMakeLists media 10 PIXELS.
//
// Essa e a forma de falha que a §4 regra 11 manda cobrir: nao ha erro, nao ha
// warning, o botao "Ativar" continua funcionando. O usuario consente com um
// texto que ele nao teve como ler. Nenhum dos outros dezoito gates ve isso —
// qmllint acha o QML impecavel, porque ele E' impecavel.
//
// MUTACAO QUE PROVA O GATE: tire o `Math.min(..., root.height * 0.4)` do
// cabecalho no ConfigActionPreview.qml e a primeira assercao cai de 285 para
// 10. Tire o crescimento do dialogo e a segunda cai de 374 para 214.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 1920
    height: 1080

    // Acao com TRES campos: o caso comum (cmake.addTargetLinkLibraries).
    function acaoComum() {
        return {
            id: "cmake.addTargetLinkLibraries",
            title: "Linkar biblioteca a um alvo",
            description: "Acrescenta target_link_libraries ao alvo escolhido.",
            riskLabel: "edita configuração",
            riskExplanation: "edita um arquivo de configuração do projeto",
            state: "available",
            docs: [{ title: "CMake: target_link_libraries" }],
            params: [
                root.campo("target", "Alvo"),
                root.campo("libraries", "Bibliotecas"),
                root.campo("visibility", "Visibilidade")
            ]
        };
    }

    // Acao com SEIS campos: o pior caso que o catalogo permite hoje.
    function acaoLonga() {
        const acao = root.acaoComum();
        acao.params = acao.params.concat([
            root.campo("standard", "Padrão da linguagem"),
            root.campo("package", "Pacote"),
            root.campo("directories", "Diretórios de cabeçalho")
        ]);
        return acao;
    }

    function campo(nome, rotulo) {
        return {
            name: nome,
            label: rotulo,
            required: false,
            placeholder: "exemplo",
            description: "Uma frase inteira explicando o que este campo muda no projeto.",
            suggestions: ["um", "dois", "tres"]
        };
    }

    QtObject {
        id: bibliotecas

        property var plan: null
        property string errorText: ""
        property string selectedId: ""
        property string target: ""
    }

    ListModel {
        id: listaVazia
    }

    QtObject {
        id: controller

        property var actionsModel: listaVazia
        property var activeBuildSystems: ["cmake", "cargo"]
        property string scopeFilter: ""
        property string searchQuery: ""
        property string errorText: ""
        property string statusText: ""
        property var selectedAction: root.acaoComum()
        property string selectedId: "cmake.addTargetLinkLibraries"
        property var previewFiles: [{ path: "CMakeLists.txt", before: "a\n", after: "a\nb\n" }]
        property var previewReport: []
        property var previewNotes: []
        property string previewSummary: "Acrescenta target_link_libraries ao alvo escolhido."
        property bool previewLoading: false
        property string previewId: "cmake.addTargetLinkLibraries"

        function isLibrary(id) { return false; }
        function select(id) {}
        function paramValue(nome) { return ""; }
        function setParam(nome, valor) {}
        function requestPreview() {}
        function missingRequiredParam() { return ""; }
        function apply() {}
    }

    ConfigActionsDialog {
        id: dialogo

        anchors.fill: parent
        controller: controller
        libraryController: bibliotecas
        maxAvailableWidth: root.width - 48
        maxAvailableHeight: root.height - 48
    }

    // A caixa nao e filha direta do dialogo: ela mora dentro do painel da
    // direita. Procurar pelo TIPO evita depender da ordem dos filhos.
    function acharCaixa(item) {
        for (let indice = 0; indice < item.children.length; ++indice) {
            const filho = item.children[indice];
            if (String(filho).indexOf("ConfigActionDiffView") === 0) {
                return filho;
            }
            const achado = root.acharCaixa(filho);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    property int falhas: 0

    // Uma passada do layout precisa acontecer antes de medir: `implicitHeight`
    // de Column com Repeater dentro so existe depois que os delegates nascem.
    Timer {
        interval: 300
        running: true
        repeat: false

        onTriggered: {
            const caixa = root.acharCaixa(dialogo);
            if (caixa === null) {
                console.warn("caixa da previa nao encontrada no dialogo");
                Qt.exit(4);
                return;
            }

            // 1920x1080, acao comum: a previa tem de caber um CMakeLists de
            // verdade. Antes de 2026-09-04 dava 214px (~12 linhas).
            if (caixa.height < 300) {
                console.warn("previa espremida em tela grande: " + caixa.height.toFixed(0) + "px");
                root.falhas += 1;
            }

            // 1366x768 com seis campos: o cabecalho nao pode comer a previa.
            // Antes de 2026-09-04 dava 10px.
            root.width = 1366;
            root.height = 768;
            controller.selectedAction = root.acaoLonga();
            aperto.start();
        }
    }

    Timer {
        id: aperto

        interval: 300
        repeat: false

        onTriggered: {
            const caixa = root.acharCaixa(dialogo);
            if (caixa.height < 180) {
                console.warn("previa espremida em tela baixa: " + caixa.height.toFixed(0) + "px");
                root.falhas += 2;
            }
            Qt.exit(root.falhas);
        }
    }
}
