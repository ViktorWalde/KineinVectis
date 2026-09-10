// A TELA DA FORMA VETORIAL EXISTE, E O CONCEITO DE SISTEMA CHEGA NELA.
//
// POR QUE ESTE HARNESS EXISTE (2026-09-07). O motor vetorial foi entregue em
// 2026-09-06 — `sim.checkSystem`, `sim.runSystem`, integrador, oraculo,
// invariantes, 649 testes verdes — e a TELA nunca foi ligada nele. O
// `SimPlotSystem` estava no `CMakeLists`, tinha harness proprio que passava, e
// nao era instanciado em lugar nenhum do app; o `SimSystemController` nascia no
// `AppDomains` e ninguem o lia.
//
// O gate inteiro ficava verde, e o efeito para quem usa era pior que a ausencia:
// os tres conceitos de sistema APARECIAM na lista, a tela escalar os aceitava, e
// medido contra o binario real o `sim.checkFormula` aprovava `mu*2` na "Orbita
// de dois corpos" e o `sim.evaluate` devolvia 2. Motor certo, tela errada,
// nenhum vermelho.
//
// O que ele cobra:
//   1. conceito de sistema NAO cai na tela escalar
//   2. ha' uma equacao por componente, com altura para digitar
//   3. ha' um campo de estado inicial por componente
//   4. o resultado desenha o grafico VETORIAL
//   5. sem pareamento declarado, o simpletico e' recusado COM MOTIVO
//   6. mexer no que decide o resultado INVALIDA o resultado na tela
//
// MUTACOES QUE PROVAM O GATE:
//   - tire o `SimSystemAuthoring` do `SimPanel`      -> falha 2 e 4
//   - troque `visible: root.isSystem` por `true`
//     no `SimFormulaField`                            -> falha 1
//   - devolva `""` no `symplecticReason`              -> falha 16
//   - tire o `clearSystemRun()` do `onValueEdited`
//     no `SimPanelHost`                                 -> falha 128
import QtQuick
import KineinVectis

Item {
    id: root

    width: 1600
    height: 1000

    property int falhas: 0

    // Uma orbita, na forma que o `sim.catalog` a devolve.
    function conceitoOrbita() {
        return {
            id: "orbita-dois-corpos",
            name: "Orbita de dois corpos",
            summary: "Corpo em orbita sob gravitacao newtoniana, no plano.",
            course: "Fisica I",
            form: "odeSystem",
            view: "plot2d",
            closedForm: false,
            source: "Halliday, Resnick & Walker; revisado em 2026-09-06",
            quantities: [
                { id: "mu", label: "parametro gravitacional (G*M)", unit: "m^3/s^2",
                  required: true }
            ],
            components: [
                { id: "x", label: "posicao x", unit: "m" },
                { id: "y", label: "posicao y", unit: "m" },
                { id: "vx", label: "velocidade x", unit: "m/s" },
                { id: "vy", label: "velocidade y", unit: "m/s" }
            ],
            pairing: [{ first: 0, second: 2 }, { first: 1, second: 3 }],
            plane: { first: 0, second: 1 },
            invariants: [
                { id: "energia", label: "energia mecanica especifica", unit: "J/kg" }
            ]
        };
    }

    // O MESMO conceito sem o pareamento: e' o caso em que o simpletico nao esta'
    // definido, e a tela tem de dizer por que em vez de oferece-lo.
    function conceitoSemPar() {
        const c = root.conceitoOrbita();
        c.id = "sistema-sem-par";
        c.pairing = [];
        return c;
    }

    // O duble do `SimController`: ele guarda o conceito e os VALORES dos
    // parametros, que sao os mesmos nas duas formas.
    QtObject {
        id: controller

        property var concepts: [root.conceitoOrbita()]
        property string selectedConcept: "orbita-dois-corpos"
        property string formula: ""
        property var variables: []
        property var issues: []
        property bool checkOk: false
        property string caveat: ""
        property var bindings: ({})
        property var values: ({ mu: "1" })
        property bool hasResult: false
        property real resultValue: 0
        property var resultSteps: []
        property string resultError: ""
        property string errorText: ""
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
        function valueList() { return [{ quantity: "mu", value: 1 }]; }
    }

    // Os controllers REAIS do projeto, nao copias.
    SimSystemController {
        id: sistema

        concept: controller.concepts[0]

        // O core responde; aqui o duble responde na hora, com a forma que o
        // `sim.checkSystem` devolve.
        onCheckRequested: (conceptId, equations) => {
            const linhas = [];
            for (let i = 0; i < equations.length; ++i) {
                linhas.push({
                    component: equations[i].component,
                    result: { ok: true, issues: [], variables: ["vx"] }
                });
            }
            sistema.applyCheck({ ok: true, components: linhas,
                                 caveat: "Conceito certo e formula valida NAO significam "
                                         + "resultado certo." });
        }
    }

    SimRunController {
        id: numerica

        concept: controller.concepts[0]
        conceptId: controller.selectedConcept
    }

    SimPanelHost {
        id: host

        anchors.fill: parent
        controller: controller
        runController: numerica
        systemController: sistema
        maxAvailableWidth: root.width - 48
        maxAvailableHeight: root.height - 48
    }

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

    function contar(item, tipo) {
        let quantos = 0;
        for (let i = 0; i < item.children.length; ++i) {
            const filho = item.children[i];
            if (String(filho).indexOf(tipo) === 0) {
                quantos += 1;
            }
            quantos += root.contar(filho, tipo);
        }
        return quantos;
    }

    Timer {
        interval: 500
        running: true
        repeat: false

        onTriggered: {
            // 1. A tela ESCALAR nao pode aceitar um conceito de sistema. Foi
            //    exatamente isso que o binario real fazia em 2026-09-06.
            const escalar = root.achar(host, "SimFormulaField");
            if (escalar !== null && escalar.visible) {
                console.warn("o campo de formula ESCALAR esta visivel num conceito "
                             + "de sistema: a tela aceita o que nao sabe montar");
                root.falhas += 1;
            }

            // 2. Uma equacao por componente, com altura para digitar.
            const montagem = root.achar(host, "SimSystemAuthoring");
            if (montagem === null || !montagem.visible) {
                console.warn("a montagem vetorial nao esta na tela");
                root.falhas += 2;
            }
            const equacoes = root.contar(host, "SimComponentEquation");
            if (equacoes !== 4) {
                console.warn("equacoes na tela: " + equacoes + ", esperado 4");
                root.falhas += 4;
            }

            // 3. Um campo de estado inicial por componente. Os campos numericos
            //    da corrida (passo, duracao, pontos) so' nascem depois da
            //    checagem, e ela ja' respondeu — por isso o piso, e nao a
            //    igualdade.
            const numeros = root.contar(host, "SimNumberField");
            if (numeros < 4) {
                console.warn("campos numericos na tela: " + numeros
                             + ", esperado ao menos 4 (o estado inicial)");
                root.falhas += 8;
            }

            proximo.restart();
        }
    }

    Timer {
        id: proximo

        interval: 400
        repeat: false

        onTriggered: {
            // 5. Sem pareamento declarado, o simpletico e' RECUSADO com motivo.
            //    Recusar dizendo por que e' o que separa esta tela de uma que
            //    integra outra coisa em silencio (`arquitetura/34` §13.3).
            const antes = sistema.symplecticReason;
            if (antes !== "") {
                console.warn("a orbita DECLARA o pareamento e mesmo assim recusa o "
                             + "simpletico: " + antes);
                root.falhas += 16;
            }
            sistema.concept = root.conceitoSemPar();
            depois.restart();
        }
    }

    Timer {
        id: depois

        interval: 400
        repeat: false

        onTriggered: {
            if (sistema.symplecticReason === "") {
                console.warn("conceito SEM pareamento nao recusa o simpletico: a tela "
                             + "oferece um metodo que nao esta definido para ele");
                root.falhas += 16;
            }

            // 4. Com resultado, o grafico VETORIAL aparece. E' o componente que
            //    existia, tinha harness e nao estava em tela nenhuma.
            sistema.applyRun({
                stepsTaken: 628,
                sampleEvery: 4,
                methodLabel: "Runge-Kutta 4, dt = 0,01",
                trail: [
                    { t: 0, values: [1, 0, 0, 1] },
                    { t: 1, values: [0.54, 0.84, -0.84, 0.54] },
                    { t: 2, values: [-0.42, 0.91, -0.91, -0.42] }
                ],
                invariants: [
                    { id: "energia", initial: -0.5, finalValue: -0.5, drift: 8.7e-11 }
                ]
            });
            desenho.restart();
        }
    }

    Timer {
        id: desenho

        interval: 400
        repeat: false

        onTriggered: {
            const grafico = root.achar(host, "SimPlotSystem");
            if (grafico === null || !grafico.visible || grafico.height < 100) {
                console.warn("o grafico vetorial nao esta na tela: "
                             + (grafico === null ? "ausente"
                                                 : grafico.height.toFixed(0) + "px"));
                root.falhas += 32;
            }

            const resultado = root.achar(host, "SimSystemResultView");
            if (resultado === null || !resultado.visible) {
                console.warn("o resultado do sistema nao esta na tela");
                root.falhas += 64;
            }

            // 6. Mexer no que DECIDE o resultado invalida o resultado. O valor
            //    do parametro mora no `SimController` e a corrida no
            //    `SimSystemController`: sem alguem atravessando essa fronteira,
            //    a tela segue mostrando o grafico da conta ANTERIOR.
            const painel = root.achar(host, "SimPanel");
            if (painel === null) {
                console.warn("painel nao encontrado");
                root.falhas += 128;
            } else {
                painel.valueEdited("mu", "2");
                if (sistema.runResult !== null) {
                    console.warn("mudar o valor de um parametro deixou o resultado "
                                 + "ANTERIOR na tela");
                    root.falhas += 128;
                }
            }

            if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
            Qt.exit(root.falhas === 0 ? 0 : 1);
        }
    }
}
