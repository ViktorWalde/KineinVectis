pragma ComponentBehavior: Bound
import QtQuick

// O ESTADO DA AUTORIA de um sistema: `n` formulas, `n` tabelas de ligacao e o
// estado inicial.
//
// Espelho do `SimController` para a forma vetorial, e ele existe separado pelo
// mesmo motivo que o `corrida_sistema.rs` existe separado do `corrida.rs`: as
// REGRAS sao outras. Na forma escalar ha' uma formula e uma tabela; aqui ha'
// uma por componente, e o que o core cobra do sistema inteiro (toda grandeza
// obrigatoria ligada em ALGUMA equacao) nao e' o que ele cobra de cada uma.
//
// O que ele NAO faz: nao checa formula, nao decide metodo, nao conta passo e
// nao preenche campo. Ele guarda o que o autor escreveu e pergunta ao core.
Item {
    id: root

    // O conceito escolhido, como veio do `sim.catalog`.
    property var concept: null
    // Uma entrada por componente: { component, formula, bindings: [] }.
    property var equations: []
    // Um valor por componente, na ordem do conceito. Guarda TEXTO, e nao numero,
    // pelo mesmo motivo do `SimController.values`: campo em branco precisa ser
    // distinguivel de zero, e digitar "1." nao pode virar "1" embaixo do dedo.
    // Comeca VAZIO — a IDE nao preenche estado inicial (`arquitetura/34` §2.1).
    property var initialState: []
    // O ultimo resultado de `sim.checkSystem`.
    property var checkResult: null
    // O ultimo resultado de `sim.runSystem`.
    property var runResult: null
    // A recusa do core, quando ela vem. Neste dominio recusa e' informacao de
    // produto: "falta o valor de mu" e' frase para o autor ler.
    property string runError: ""
    // "components" ou "trajectory".
    property string plotMode: "components"

    // Verdadeiro so' quando o conceito e' vetorial.
    readonly property bool isSystem: root.concept !== null
                                     && root.concept.form === "odeSystem"
    // O conceito declara o pareamento posicao/velocidade?
    //
    // Quando NAO declara, o metodo simpletico nao esta' definido e a tela o
    // desabilita com o motivo, em vez de deixar o autor pedir e receber recusa
    // (`arquitetura/34` §13.3).
    readonly property bool supportsSymplectic: root.isSystem
                                               && root.concept.pairing !== undefined
                                               && root.concept.pairing.length > 0
    readonly property var plane: root.isSystem && root.concept.plane !== undefined
                                 ? root.concept.plane : null

    signal checkRequested(string conceptId, var equations)
    signal runRequested(string conceptId, var equations, var values, var initial,
                        real duration, real step, string method, int samples)

    visible: false

    // Prepara as `n` equacoes VAZIAS quando o conceito muda.
    //
    // Vazias de proposito: o campo comeca em branco e quem escreve a equacao e'
    // o autor. Pre-preencher com a formula canonica responderia a pergunta
    // errada — o pedido de 2026-09-01 e' que a equacao seja informada por ele.
    onConceptChanged: {
        if (!root.isSystem) {
            root.equations = [];
            root.initialState = [];
            root.checkResult = null;
            root.runResult = null;
            root.runError = "";
            return;
        }
        const novas = [];
        const estado = [];
        for (let i = 0; i < root.concept.components.length; ++i) {
            novas.push({
                component: root.concept.components[i].id,
                formula: "",
                bindings: []
            });
            estado.push("");
        }
        root.equations = novas;
        root.initialState = estado;
        root.checkResult = null;
        root.runResult = null;
        root.runError = "";
        root.plotMode = "components";
    }

    // O autor digitou na equacao de um componente.
    function setFormula(componentId, texto) {
        const copia = root.equations.slice();
        for (let i = 0; i < copia.length; ++i) {
            if (copia[i].component === componentId) {
                copia[i] = { component: componentId, formula: texto,
                             bindings: copia[i].bindings };
            }
        }
        root.equations = copia;
        root.clearRun();
        root.scheduleCheck();
    }

    // O autor ligou uma variavel a uma grandeza ou a um componente.
    function setBinding(componentId, variavel, grandeza) {
        const copia = root.equations.slice();
        for (let i = 0; i < copia.length; ++i) {
            if (copia[i].component !== componentId) {
                continue;
            }
            const ligacoes = copia[i].bindings.filter(b => b.variable !== variavel);
            if (grandeza !== "") {
                ligacoes.push({ variable: variavel, quantity: grandeza });
            }
            copia[i] = { component: componentId, formula: copia[i].formula,
                         bindings: ligacoes };
        }
        root.equations = copia;
        root.clearRun();
        root.scheduleCheck();
    }

    function setInitial(indice, texto) {
        const copia = root.initialState.slice();
        copia[indice] = texto;
        root.initialState = copia;
        root.clearRun();
    }

    // Verdadeiro quando todo componente tem um valor inicial preenchido.
    //
    // Campo em branco e zero sao coisas DIFERENTES: branco e' recusa, nunca
    // zero implicito.
    readonly property bool initialComplete: {
        if (!root.isSystem || root.initialState.length === 0) {
            return false;
        }
        for (let i = 0; i < root.initialState.length; ++i) {
            if (root.numberOrNull(root.initialState[i]) === null) {
                return false;
            }
        }
        return true;
    }

    function numberOrNull(texto) {
        const limpo = String(texto === undefined || texto === null ? "" : texto).trim();
        if (limpo === "") {
            return null;
        }
        const n = Number(limpo);
        return isFinite(n) ? n : null;
    }

    // O estado inicial na forma do protocolo: `n` numeros na ordem do conceito.
    function initialList() {
        const saida = [];
        for (let i = 0; i < root.initialState.length; ++i) {
            saida.push(root.numberOrNull(root.initialState[i]));
        }
        return saida;
    }

    // --- o que a tela le, e nao deriva por conta propria --------------------
    //
    // Estas funcoes existem aqui, e nao no componente visual, porque sao
    // LEITURA DE ESTADO: qual o mapa de ligacao daquela equacao, quais
    // variaveis o core achou nela, o que ele reprovou. A tela desenha o que
    // elas devolvem (`ARCHITECTURE.md` §2).

    // A checagem passou inteira?
    readonly property bool checkOk: root.checkResult !== null && root.checkResult.ok === true

    // O aviso da §5.1, que aparece justamente quando esta' tudo certo.
    readonly property string caveat: root.checkResult === null
                                     || root.checkResult.caveat === undefined
                                     || root.checkResult.caveat === null
                                     ? "" : root.checkResult.caveat

    function equationFor(componentId) {
        for (let i = 0; i < root.equations.length; ++i) {
            if (root.equations[i].component === componentId) {
                return root.equations[i];
            }
        }
        return null;
    }

    // A ligacao daquela equacao como MAPA, que e' a forma que a tabela le. Ela
    // e' guardada como lista porque e' assim que o protocolo a transporta.
    function bindingMapFor(componentId) {
        const equacao = root.equationFor(componentId);
        const mapa = ({});
        if (equacao === null) {
            return mapa;
        }
        for (let i = 0; i < equacao.bindings.length; ++i) {
            mapa[equacao.bindings[i].variable] = equacao.bindings[i].quantity;
        }
        return mapa;
    }

    function checkFor(componentId) {
        if (root.checkResult === null || root.checkResult.components === undefined) {
            return null;
        }
        for (let i = 0; i < root.checkResult.components.length; ++i) {
            if (root.checkResult.components[i].component === componentId) {
                return root.checkResult.components[i].result;
            }
        }
        return null;
    }

    // As variaveis que o CORE achou naquela formula. A UI nunca parseia texto.
    function variablesFor(componentId) {
        const r = root.checkFor(componentId);
        return r === null || r.variables === undefined ? [] : r.variables;
    }

    function issuesFor(componentId) {
        const r = root.checkFor(componentId);
        return r === null || r.issues === undefined ? [] : r.issues;
    }

    function componentOkFor(componentId) {
        const r = root.checkFor(componentId);
        return r !== null && r.ok === true;
    }

    // Os ALVOS de ligacao de uma equacao do sistema: as grandezas do conceito,
    // os COMPONENTES do estado e o tempo.
    //
    // A lista e' a mesma que o `corrida_sistema::checar` monta no core, e a
    // razao de ela existir aqui e' que o protocolo transporta o conceito, nao a
    // lista. Se as duas divergirem, quem manda e' o core: ele recusa a ligacao
    // com `unknownQuantity`, e o erro aparece na tela em vez de sumir.
    //
    // A forma escalar nao precisa disto porque nela `y`, `dy` e `t` sao
    // GRANDEZAS declaradas do conceito; num sistema o estado sao os
    // componentes, e eles vivem em campo proprio (`arquitetura/34` §13.2).
    function bindingTargets() {
        if (!root.isSystem) {
            return [];
        }
        const alvos = [];
        const doEstado = ({});
        for (let i = 0; i < root.concept.components.length; ++i) {
            const c = root.concept.components[i];
            doEstado[c.id] = true;
            alvos.push({ id: c.id, label: c.label, unit: c.unit });
        }
        for (let j = 0; j < root.concept.quantities.length; ++j) {
            const q = root.concept.quantities[j];
            // Um id que e' componente E grandeza nao vira dois chips. No core a
            // ordem de resolucao e' tempo, componente, parametro — o componente
            // GANHA, e o valor do parametro seria ignorado calado
            // (`corrida_sistema::executar`). Dois chips iguais fariam o autor
            // escolher entre coisas que nao sao duas.
            if (doEstado[q.id] !== true) {
                alvos.push({ id: q.id, label: q.label, unit: q.unit });
            }
        }
        alvos.push({ id: "t", label: qsTr("tempo"), unit: "s" });
        return alvos;
    }

    // As grandezas que o autor ligou em ALGUMA equacao — sao essas que pedem
    // valor. Componente e tempo nao pedem: um vem do estado, o outro do
    // integrador.
    function boundQuantities() {
        if (!root.isSystem) {
            return [];
        }
        const usadas = ({});
        for (let i = 0; i < root.equations.length; ++i) {
            const ligacoes = root.equations[i].bindings;
            for (let j = 0; j < ligacoes.length; ++j) {
                usadas[ligacoes[j].quantity] = true;
            }
        }
        return root.concept.quantities.filter(q => usadas[q.id] === true);
    }

    // O respiro antes de perguntar ao core, igual ao da forma escalar: checar a
    // cada tecla e' de graca no core (5,3 us), mas a ida e volta pelo JSON-RPC
    // nao e'.
    Timer {
        id: respiro

        interval: 180
        onTriggered: {
            if (root.concept !== null) {
                root.checkRequested(root.concept.id, root.equations);
            }
        }
    }

    function scheduleCheck() {
        if (root.isSystem) {
            respiro.restart();
        }
    }

    // Tudo que decide o resultado esta' preenchido — menos a numerica, que vive
    // no `SimRunController` e e' conferida por ele.
    //
    // A separacao nao e' cerimonia: trocar o metodo nao mexe nas equacoes, e
    // trocar uma equacao nao bagunca a escolha numerica. Cada um confere o que
    // guarda, e quem os junta e' o host.
    function readyToAuthor(values) {
        if (!root.isSystem || !root.checkOk || !root.initialComplete) {
            return false;
        }
        const grandezas = root.boundQuantities();
        for (let i = 0; i < grandezas.length; ++i) {
            const texto = values[grandezas[i].id];
            if (root.numberOrNull(texto) === null) {
                return false;
            }
        }
        return true;
    }

    // `numerics` vem do `SimRunController` como `{ duration, step, method,
    // samples }`, ou `null` quando ainda falta campo. Nada e' suprido aqui: sem
    // os quatro, a corrida simplesmente nao sai.
    function requestRun(values, numerics) {
        if (root.concept === null || numerics === null || numerics === undefined) {
            return;
        }
        if (!root.readyToAuthor(root.valuesMap(values))) {
            return;
        }
        root.runError = "";
        root.runRequested(root.concept.id, root.equations, values,
                          root.initialList(), numerics.duration, numerics.step,
                          numerics.method, numerics.samples);
    }

    // A lista de valores do protocolo de volta em mapa, para a conferencia
    // acima ler pelo id da grandeza.
    function valuesMap(values) {
        const mapa = ({});
        for (let i = 0; i < values.length; ++i) {
            mapa[values[i].quantity] = values[i].value;
        }
        return mapa;
    }

    // O motivo da recusa do simpletico, dito na tela em vez de escondido
    // (`arquitetura/34` §13.3). Vazio quando ele esta' disponivel.
    readonly property string symplecticReason:
        root.isSystem && !root.supportsSymplectic
        ? qsTr("Este conceito não declara quais componentes são posição e "
               + "velocidade um do outro, então o método simplético não está "
               + "definido para ele.")
        : ""

    function setPlotMode(modo) {
        root.plotMode = modo;
    }

    function clearRun() {
        root.runResult = null;
        root.runError = "";
    }

    function applyCheck(resultado) {
        root.checkResult = resultado;
    }

    function applyRun(resultado) {
        root.runResult = resultado;
        root.runError = "";
    }

    function handleFailed(method, message) {
        if (method === "sim.runSystem") {
            root.runError = message;
            root.runResult = null;
        }
    }
}
