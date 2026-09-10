// A CORRIDA: nenhum campo tem padrao, e o metodo NAO e' escolhido pela IDE.
//
// Por que existe: o `SimRunController` decide quando a integracao pode partir.
// Se essa decisao quebrar, nenhum compilador reclama — a IDE so passa a
// integrar com passo que o autor nao escolheu, ou com um metodo que ela
// escolheu por ele. E metodo escolhido por engano nao e' detalhe: medido, o
// Euler explicito com dt=0,1 erra por 3,11 onde a resposta e' -0,276.
//
// MUTACAO QUE PROVA O GATE: faca `readyToRun()` ignorar o metodo vazio e a
// primeira assercao cai; tire o `reset()` de troca de conceito e a ultima cai.
import QtQuick
import "../../ui/qml/sim"

Item {
    id: root

    property var corridas: []
    property var estimativas: []

    SimRunController {
        id: controller

        concept: root.conceitoEdo()
        conceptId: "oscilador-amortecido"
        formula: "(-k*y - c*dy) / m"
        checkOk: true
        bindings: ({ k: "k", y: "y", c: "c", dy: "dy", m: "m" })
        values: ({ m: "2", k: "10", c: "0.5" })

        onRunRequested: function (conceptId, formula, bindings, values, initial,
                                 duration, step, method, samples) {
            root.corridas.push({ concept: conceptId, initial: initial, duration: duration,
                                 step: step, method: method, samples: samples,
                                 values: values });
        }
        onEstimateRequested: function (duration, step, samples) {
            root.estimativas.push({ duration: duration, step: step, samples: samples });
        }
    }

    // O SHAPE E' O DO CORE: `form` vem do catalogo, e e' ele que diz se o
    // conceito integra. A UI nao deduz isso da formula.
    function conceitoEdo() {
        return {
            id: "oscilador-amortecido",
            name: "Oscilador harmonico amortecido",
            form: "ode2",
            view: "plot2d",
            closedForm: true,
            quantities: [
                { id: "m", label: "massa", unit: "kg", required: true },
                { id: "k", label: "constante elastica", unit: "N/m", required: true },
                { id: "c", label: "amortecimento", unit: "N.s/m", required: false },
                { id: "y", label: "posicao", unit: "m", required: true },
                { id: "dy", label: "velocidade", unit: "m/s", required: false }
            ]
        };
    }

    function conceitoAlgebrico() {
        return { id: "energia-cinetica", name: "Energia cinetica", form: "algebraic",
                 view: "plot2d", closedForm: true, quantities: [] };
    }

    Component.onCompleted: {
        let failures = 0;

        // A forma vem do CATALOGO. Ode2 integra e pede y'(0).
        if (!controller.integrates()) failures += 1;
        if (!controller.needsInitialDerivative()) failures += 2;

        // NADA COMECA PREENCHIDO. Nem metodo, nem passo, nem duracao.
        if (controller.method !== "") failures += 4;
        if (controller.stepText !== "") failures += 8;
        if (controller.readyToRun()) failures += 16;

        // Preencher os numeros nao basta: sem METODO a corrida nao parte, e e'
        // por isso que a IDE nao escolhe um por voce.
        controller.editRunField("step", "0.001");
        controller.editRunField("duration", "10");
        controller.editRunField("samples", "50");
        controller.editRunField("y0", "1");
        controller.editRunField("dy0", "0");
        if (controller.readyToRun()) failures += 32;
        controller.runSimulation();
        if (root.corridas.length !== 0) failures += 64;

        // Com o metodo, parte.
        controller.chooseMethod("rk4");
        if (!controller.readyToRun()) failures += 128;
        controller.runSimulation();
        if (root.corridas.length !== 1) {
            failures += 256;
        } else {
            const c = root.corridas[0];
            if (c.method !== "rk4") failures += 512;
            // Os numeros viajam como NUMERO, nao como o texto do campo.
            if (typeof c.step !== "number" || typeof c.duration !== "number") failures += 1024;
            if (Math.abs(c.step - 0.001) > 1e-12) failures += 2048;
            if (c.initial.y !== 1 || c.initial.dy !== 0) failures += 4096;
            // Os parametros do conceito viajam junto, ja' como numero.
            if (c.values.length !== 3) failures += 8192;
        }

        // Clicar no metodo escolhido DESLIGA: desfazer tem de ser possivel.
        controller.chooseMethod("rk4");
        if (controller.method !== "") failures += 16384;
        if (controller.readyToRun()) failures += 32768;
        controller.chooseMethod("rk4");

        // Campo numerico invalido bloqueia, como campo vazio.
        controller.editRunField("step", "abc");
        if (controller.readyToRun()) failures += 65536;
        controller.editRunField("step", "0.001");

        // PRIMEIRA ORDEM nao pede y'(0): pedir seria pedir algo que a equacao
        // nao tem.
        const ode1 = root.conceitoEdo();
        ode1.form = "ode1";
        controller.concept = ode1;
        if (controller.needsInitialDerivative()) failures += 131072;
        controller.editRunField("dy0", "");
        if (!controller.readyToRun()) failures += 262144;
        controller.concept = root.conceitoEdo();
        controller.editRunField("dy0", "0");

        // FORMA ALGEBRICA nao integra: o painel nao mostra os controles, e o
        // botao nao habilita.
        controller.concept = root.conceitoAlgebrico();
        if (controller.integrates()) failures += 524288;
        if (controller.readyToRun()) failures += 1048576;
        controller.concept = root.conceitoEdo();

        // A ESTIMATIVA e' pedida ao core, e so' quando os tres numeros existem.
        const antes = root.estimativas.length;
        controller.editRunField("duration", "");
        controller.editRunField("duration", "20");
        if (root.estimativas.length < antes) failures += 2097152;

        // O resultado chega e some quando qualquer campo muda: um numero na
        // tela que nao corresponde mais aos parametros acima dele mente.
        controller.handleRan({ stepsTaken: 20000, sampleEvery: 400, trail: [],
                               methodLabel: "Runge-Kutta 4, dt = 1e-3",
                               accuracy: { exact: -0.2758, numeric: -0.2758,
                                           absoluteError: 1e-12 } });
        if (controller.run === null) failures += 4194304;
        controller.editRunField("step", "0.01");
        if (controller.run !== null) failures += 8388608;

        // TROCAR DE CONCEITO zera a escolha numerica junto com a equacao.
        controller.handleRan({ stepsTaken: 1, sampleEvery: 1, trail: [], methodLabel: "x" });
        controller.reset();
        if (controller.method !== "") failures += 16777216;
        if (controller.stepText !== "") failures += 33554432;
        if (controller.run !== null) failures += 67108864;
        if (controller.estimate !== null) failures += 134217728;

        // Erro de OUTRO dominio nao acende o aviso desta tela.
        controller.handleFailed("git.status", "nada a ver");
        if (controller.runError !== "") failures += 268435456;
        controller.handleFailed("sim.run", "a corrida pedida e longa demais");
        if (controller.runError === "") failures += 536870912;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
