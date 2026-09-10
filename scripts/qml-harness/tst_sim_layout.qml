// A TABELA DE LIGACAO NAO PODE SUMIR.
//
// Por que existe: o painel de simulacao empilha resumo do conceito, fonte,
// campo da formula, lista de problemas, TABELA DE LIGACAO, tabela de valores,
// botoes e o painel do calculo. A tabela de ligacao cresce com o numero de
// variaveis da formula — e ela e' onde o autor toma a decisao que a IDE se
// recusa a tomar por ele (arquitetura/34 §5.0). Se ela ficar espremida ou
// invisivel, a tela vira "escreva a formula e confie", que e' o oposto do que
// esta etapa decidiu.
//
// E' a mesma forma de falha que a previa do CMakeLists teve em 2026-09-04: sem
// erro, sem warning, qmllint impecavel — e o usuario decidindo sobre algo que
// ele nao consegue ver. Nenhum dos outros gates ve isso.
//
// MUTACAO QUE PROVA O GATE: troque o `Flickable` do SimPanel por um `Column`
// simples e a assercao de rolagem cai; tire o `visible` da tabela ligado a
// `variables.length` e a contagem de linhas para de bater.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 1920
    height: 1080

    // Um conceito com CINCO grandezas: o pior caso do catalogo de hoje
    // (`gases-ideais` e `coulomb` tem quatro; cinco da folga para crescer).
    function conceito() {
        return {
            id: "teste-cinco",
            name: "Conceito de cinco grandezas",
            summary: "Um resumo de uma linha, como os do catalogo.",
            course: "Fisica basica",
            form: "algebraic",
            view: "plot2d",
            closedForm: true,
            source: "Halliday, Resnick & Walker; revisado em 2026-09-05",
            quantities: [
                { id: "a", label: "primeira grandeza", unit: "m", required: true },
                { id: "b", label: "segunda grandeza", unit: "m/s", required: true },
                { id: "c", label: "terceira grandeza", unit: "kg", required: true },
                { id: "d", label: "quarta grandeza", unit: "N", required: true },
                { id: "e", label: "quinta grandeza", unit: "J", required: false }
            ]
        };
    }

    QtObject {
        id: controller

        property var concepts: [root.conceito()]
        property string selectedConcept: "teste-cinco"
        property string formula: "a*b + c*d - e"
        property var variables: ["a", "b", "c", "d", "e"]
        property var issues: []
        property bool checkOk: true
        property string caveat: "Conceito certo e formula valida NAO significam resultado certo."
        property var bindings: ({ a: "a", b: "b", c: "c", d: "d", e: "e" })
        property var values: ({})
        property bool hasResult: false
        property real resultValue: 0
        property var resultSteps: []
        property string resultError: ""
        property string errorText: ""
        // O duble espelha o CONTRATO do controller real. Quando o painel ganhou
        // persistencia em 2026-09-05, este harness caiu com
        // "Cannot call method 'trim' of undefined" — e foi ele que avisou que o
        // host passara a exigir campos novos.
        property var saved: []
        property string saveName: ""

        function readyToEvaluate() { return false; }
        function selectConcept(id) {}
        function editFormula(t) {}
        function bindVariable(v, q) {}
        function editValue(q, t) {}
        function evaluate() {}
        function close() {}
        function save() {}
        function load(s) {}
        function forget(n) {}
    }

    SimPanelHost {
        id: host

        anchors.fill: parent
        controller: controller
        maxAvailableWidth: root.width - 48
        maxAvailableHeight: root.height - 48
    }

    // Procura pelo TIPO evita depender da ordem dos filhos.
    function achar(item, tipo) {
        for (let i = 0; i < item.children.length; ++i) {
            const filho = item.children[i];
            if (String(filho).indexOf(tipo) === 0) {
                return filho;
            }
            const achado = root.achar(filho, tipo);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    property int falhas: 0

    // Uma passada do layout precisa acontecer antes de medir: `implicitHeight`
    // de Column com Repeater dentro so' existe depois que os delegates nascem.
    Timer {
        interval: 400
        running: true
        repeat: false

        onTriggered: {
            const tabela = root.achar(host, "SimBindingTable");
            if (tabela === null) {
                console.warn("tabela de ligacao nao encontrada no painel");
                Qt.exit(4);
                return;
            }

            // A tabela existe e tem altura: cinco variaveis nao podem caber em
            // zero pixel. Cada linha tem 26px de chip mais respiro.
            if (!tabela.visible) {
                console.warn("tabela de ligacao invisivel com 5 variaveis");
                root.falhas += 1;
            }
            if (tabela.height < 5 * 24) {
                console.warn("tabela espremida: " + tabela.height.toFixed(0)
                             + "px para 5 variaveis");
                root.falhas += 2;
            }

            // O campo da FORMULA e' onde tudo comeca; ele nao pode ser fino
            // demais para digitar.
            const campo = root.achar(host, "SimFormulaField");
            if (campo === null || campo.height < 60) {
                console.warn("campo da formula pequeno demais: "
                             + (campo === null ? "ausente" : campo.height.toFixed(0)));
                root.falhas += 4;
            }

            // Numa tela de notebook o conteudo nao cabe — e a resposta certa e'
            // ROLAR, nao cortar em silencio. Sem o Flickable, `contentHeight`
            // nao existe e o final da tela fica inalcancavel.
            root.width = 1366;
            root.height = 768;
            medir.restart();
        }
    }

    Timer {
        id: medir

        interval: 400
        repeat: false

        onTriggered: {
            const rolagem = root.achar(host, "QQuickFlickable");
            if (rolagem === null) {
                console.warn("o painel nao rola: conteudo alto vira corte silencioso");
                root.falhas += 8;
            } else if (rolagem.contentHeight <= 0) {
                console.warn("contentHeight zerado: " + rolagem.contentHeight);
                root.falhas += 16;
            }

            const tabela = root.achar(host, "SimBindingTable");
            if (tabela === null || !tabela.visible || tabela.height < 5 * 24) {
                console.warn("tabela de ligacao sumiu na tela pequena: "
                             + (tabela === null ? "ausente" : tabela.height.toFixed(0) + "px"));
                root.falhas += 32;
            }

            if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
            Qt.exit(root.falhas === 0 ? 0 : 1);
        }
    }
}
