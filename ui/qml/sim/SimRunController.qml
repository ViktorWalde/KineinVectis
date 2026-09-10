pragma ComponentBehavior: Bound
import QtQuick

// COMO RESOLVER a equacao: metodo, passo, duracao, amostragem e estado inicial.
//
// Separado do `SimController` porque sao decisoes DIFERENTES, tomadas em
// momentos diferentes: uma e' montar a equacao (conceito, formula, ligacao), a
// outra e' escolher a numerica com que ela se resolve. Trocar o metodo nao mexe
// na formula, e trocar a formula nao bagunca a escolha numerica — exceto quando
// o CONCEITO muda, e ai os dois zeram.
//
// O PRINCIPIO VALE INTEIRO AQUI (arquitetura/34 §2.1): campo comeca VAZIO e nao
// ganha padrao. A razao nao e' rigor — **o metodo muda o numero**. Medido no
// oscilador amortecido com dt=0,1: Euler explicito erra por 3,11 onde a
// resposta e' -0,276 (onze vezes a propria resposta), e o RK4 erra por 4,9e-5.
//
// E a ESTIMATIVA vem do core: contar passos e' regra de negocio, e a
// `ARCHITECTURE.md` §2 proibe regra na camada de apresentacao.
Item {
    id: root

    // Vem do SimController; esta tela nao os guarda, so' os le para saber o que
    // ainda falta preencher.
    property var concept: null
    property var bindings: ({})
    property var values: ({})
    property bool checkOk: false
    property string formula: ""
    property string conceptId: ""

    // --- os campos da corrida --------------------------------------------
    //
    // Campo comeca VAZIO e nao ganha padrao. Nem metodo, nem passo, nem
    // duracao, nem amostras (arquitetura/34 §2.1) — e a razao nao e' rigor: o
    // METODO MUDA O NUMERO. Medido, com dt=0,1 no oscilador amortecido, o Euler
    // erra por 3,11 onde a resposta e' -0,276, e o RK4 erra por 4,9e-5.
    property string method: ""
    property string stepText: ""
    property string durationText: ""
    property string samplesText: ""
    property string y0Text: ""
    property string dy0Text: ""

    // A estimativa que o core devolve, mostrada ANTES de rodar.
    property var estimate: null
    // O resultado da corrida.
    property var run: null
    property string runError: ""

    signal estimateRequested(real duration, real step, int samples)
    signal runRequested(string conceptId, string formula, var bindings, var values,
                        var initial, real duration, real step, string method, int samples)

    // A forma do conceito decide se ele INTEGRA ou so' avalia. Vem do catalogo,
    // que e' declaracao auditada — a UI nao deduz nada da formula.
    function integrates() {
        return root.concept !== null
               && (root.concept.form === "ode1" || root.concept.form === "ode2");
    }

    function needsInitialDerivative() {
        return root.concept !== null && root.concept.form === "ode2";
    }

    function numberOrNull(texto) {
        const limpo = String(texto === undefined ? "" : texto).trim();
        if (limpo === "") {
            return null;
        }
        const n = Number(limpo);
        return isFinite(n) ? n : null;
    }

    function editRunField(campo, texto) {
        if (campo === "step") root.stepText = texto;
        else if (campo === "duration") root.durationText = texto;
        else if (campo === "samples") root.samplesText = texto;
        else if (campo === "y0") root.y0Text = texto;
        else if (campo === "dy0") root.dy0Text = texto;
        root.run = null;
        root.runError = "";
        root.scheduleEstimate();
    }

    function chooseMethod(id) {
        root.method = root.method === id ? "" : id;
        root.run = null;
        root.runError = "";
    }

    // A estimativa e' pedida ao CORE: contar passos e' regra de negocio, e a
    // ARCHITECTURE §2 proibe regra na camada de apresentacao.
    Timer {
        id: respiroEstimativa

        interval: 180
        onTriggered: {
            const d = root.numberOrNull(root.durationText);
            const p = root.numberOrNull(root.stepText);
            const a = root.numberOrNull(root.samplesText);
            if (d === null || p === null || a === null || p <= 0 || d <= 0 || a <= 0) {
                root.estimate = null;
                return;
            }
            root.estimateRequested(d, p, Math.round(a));
        }
    }

    function scheduleEstimate() {
        respiroEstimativa.restart();
    }

    // A ESCOLHA NUMERICA fechada, ou `null` quando ainda falta campo.
    //
    // Ela e' a mesma nas duas formas — a escalar e a vetorial —, e por isso mora
    // num lugar so'. A forma vetorial guarda a AUTORIA no `SimSystemController`
    // e vem buscar a numerica aqui; o que difere entre elas e' o estado inicial
    // (um numero contra `n`), e ele fica com quem o guarda.
    function numerics() {
        if (root.method === "") {
            return null;
        }
        const p = root.numberOrNull(root.stepText);
        const d = root.numberOrNull(root.durationText);
        const a = root.numberOrNull(root.samplesText);
        if (p === null || d === null || a === null) {
            return null;
        }
        return { duration: d, step: p, method: root.method, samples: Math.round(a) };
    }

    // Tudo que decide o resultado esta preenchido?
    function readyToRun() {
        if (!root.checkOk || !root.integrates() || root.numerics() === null) {
            return false;
        }
        if (root.numberOrNull(root.y0Text) === null) return false;
        if (root.needsInitialDerivative() && root.numberOrNull(root.dy0Text) === null) {
            return false;
        }
        // Grandezas ligadas que NAO sao estado precisam de valor.
        const estado = { y: true, dy: true, t: true };
        for (const variavel in root.bindings) {
            const grandeza = root.bindings[variavel];
            if (estado[grandeza] === true) {
                continue;
            }
            if (root.numberOrNull(root.values[grandeza]) === null) {
                return false;
            }
        }
        return true;
    }

    function runSimulation() {
        if (!root.readyToRun()) {
            return;
        }
        const inicial = { y: root.numberOrNull(root.y0Text) };
        if (root.needsInitialDerivative()) {
            inicial.dy = root.numberOrNull(root.dy0Text);
        }
        const numerica = root.numerics();
        root.runRequested(root.conceptId, root.formula, root.bindingList(),
                          root.valueList(), inicial,
                          numerica.duration, numerica.step,
                          numerica.method, numerica.samples);
    }

    function handleEstimated(resultado) {
        root.estimate = resultado;
    }

    function handleRan(resultado) {
        root.run = resultado;
        root.runError = "";
    }


    // As listas na forma do protocolo, montadas a partir do que o vizinho
    // guarda. Sao as MESMAS regras do `SimController`: a ligacao vira lista de
    // pares, e valor vazio NAO vira zero — ele simplesmente nao vai, e o core
    // recusa dizendo qual grandeza falta.
    function bindingList() {
        const saida = [];
        for (const variavel in root.bindings) {
            saida.push({ variable: variavel, quantity: root.bindings[variavel] });
        }
        return saida;
    }

    function valueList() {
        const saida = [];
        for (const grandeza in root.values) {
            const numero = root.numberOrNull(root.values[grandeza]);
            if (numero !== null) {
                saida.push({ quantity: grandeza, value: numero });
            }
        }
        return saida;
    }

    // Acrescenta a parte numerica a' montagem que vai para o disco.
    //
    // O objeto e' preenchido NO LUGAR de proposito: quem monta o resto e' o
    // `SimController`, e duplicar a montagem nos dois seria dois lugares onde o
    // mesmo campo pode faltar.
    function fillNumerics(destino) {
        if (!root.integrates()) {
            return;
        }
        const inicial = { y: root.numberOrNull(root.y0Text) };
        if (root.needsInitialDerivative()) {
            inicial.dy = root.numberOrNull(root.dy0Text);
        }
        destino.initial = inicial;
        if (root.method !== "") destino.method = root.method;
        const p = root.numberOrNull(root.stepText);
        const d = root.numberOrNull(root.durationText);
        const a = root.numberOrNull(root.samplesText);
        if (p !== null) destino.step = p;
        if (d !== null) destino.duration = d;
        if (a !== null) destino.samples = Math.round(a);
    }

    // Repoe a numerica ao carregar uma simulacao salva.
    function restoreNumerics(salva) {
        root.reset();
        if (salva.method !== undefined && salva.method !== null) root.method = salva.method;
        if (salva.step !== undefined && salva.step !== null) root.stepText = String(salva.step);
        if (salva.duration !== undefined && salva.duration !== null) {
            root.durationText = String(salva.duration);
        }
        if (salva.samples !== undefined && salva.samples !== null) {
            root.samplesText = String(salva.samples);
        }
        if (salva.initial !== undefined && salva.initial !== null) {
            if (salva.initial.y !== undefined) root.y0Text = String(salva.initial.y);
            if (salva.initial.dy !== undefined && salva.initial.dy !== null) {
                root.dy0Text = String(salva.initial.dy);
            }
        }
        root.scheduleEstimate();
    }

    // Trocar de conceito zera a escolha numerica junto com a equacao.
    function reset() {
        root.method = "";
        root.stepText = "";
        root.durationText = "";
        root.samplesText = "";
        root.y0Text = "";
        root.dy0Text = "";
        root.estimate = null;
        root.run = null;
        root.runError = "";
    }

    function handleFailed(method, message) {
        if (method === "sim.run") {
            root.runError = message;
            root.run = null;
        } else if (method === "sim.estimate") {
            root.estimate = null;
        }
    }
}
