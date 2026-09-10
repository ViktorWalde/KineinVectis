pragma ComponentBehavior: Bound
import QtQuick

// Estado da SIMULACAO POR CONCEITO (etapa 28, docs/arquitetura/34).
//
// Guarda o que o core respondeu e o que o autor esta' montando. NAO decide
// nada: quem sabe o que e' um conceito, quais variaveis a formula usa e se as
// duas coisas batem e' o core.
//
// O PRINCIPIO QUE GOVERNA ESTA TELA (arquitetura/34 §2.1, autor 2026-09-05):
// **nada e' adivinhado**. Em particular:
//   - a ligacao entre variavel e grandeza e' escolha do usuario, nunca casada
//     por nome parecido;
//   - todo campo comeca VAZIO, e a conta nao roda com campo em branco;
//   - a IDE calcula e MOSTRA, e nunca preenche nem corrige por voce.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property var concepts: []
    property string selectedConcept: ""
    property bool panelVisible: false
    property string errorText: ""

    // A formula que o autor esta' digitando.
    property string formula: ""

    // O que o core achou na ultima checagem. `variables` vem dele: a UI NUNCA
    // extrai variavel de texto por conta propria — parsear formula na camada de
    // apresentacao seria regra de negocio na UI, que a ARCHITECTURE §2 proibe.
    property var variables: []
    property var issues: []
    property bool checkOk: false
    property string caveat: ""

    // A ligacao do autor: { variavel: idDaGrandeza }. Comeca vazia, e e' o
    // usuario que a preenche.
    property var bindings: ({})
    // Os valores: { idDaGrandeza: texto }. Texto, e nao numero, porque campo
    // vazio precisa ser distinguivel de zero.
    property var values: ({})

    // O resultado da ultima avaliacao.
    property bool hasResult: false
    property real resultValue: 0
    property var resultSteps: []
    property string resultError: ""

    // As simulacoes salvas em `.kinein/simulacoes/`.
    //
    // O que se salva e' o que o autor MONTOU — conceito, formula, ligacao,
    // valores e a escolha numerica. NUNCA o resultado (arquitetura/34 §9): e' a
    // licao do `.ipynb`, que e' texto e mesmo assim falha por misturar o
    // autoral com a saida da maquina.
    property var saved: []
    property string saveName: ""

    // Avisa quem guarda a parte numerica que o conceito mudou.
    signal conceptChanged()
    // Pede a montagem numerica ao vizinho, para gravar junto.
    signal numericsRequested(var destino)
    // Manda a parte numerica de volta ao vizinho ao carregar uma salva.
    signal numericsRestored(var salva)

    signal catalogRequested()
    signal savedListRequested()
    signal saveRequested(var simulation)
    signal forgetRequested(string name)
    signal checkRequested(string conceptId, string formula, var bindings)
    signal evaluateRequested(string conceptId, string formula, var bindings, var values)

    function open() {
        root.panelVisible = true;
        if (root.concepts.length === 0) {
            root.catalogRequested();
        }
        root.savedListRequested();
    }

    function save() {
        if (root.saveName.trim() === "" || root.selectedConcept === "") {
            return;
        }
        // A parte numerica vive no vizinho; ela e' recolhida na hora de gravar,
        // e nao duplicada aqui.
        const montagem = {
            name: root.saveName.trim(),
            concept: root.selectedConcept,
            formula: root.formula,
            bindings: root.bindingList(),
            values: root.valueList()
        };
        root.numericsRequested(montagem);
        root.saveRequested(montagem);
    }

    function forget(nome) {
        root.forgetRequested(nome);
    }

    // Carregar uma salva repoe a montagem inteira — inclusive a numerica, que
    // volta pelo vizinho.
    function load(salva) {
        root.selectedConcept = salva.concept;
        root.formula = salva.formula;
        root.saveName = salva.name;
        const lig = ({});
        for (let i = 0; i < salva.bindings.length; ++i) {
            lig[salva.bindings[i].variable] = salva.bindings[i].quantity;
        }
        root.bindings = lig;
        const val = ({});
        for (let j = 0; j < salva.values.length; ++j) {
            val[salva.values[j].quantity] = String(salva.values[j].value);
        }
        root.values = val;
        root.clearResult();
        root.numericsRestored(salva);
        root.scheduleCheck();
    }

    function handleSavedList(lista) {
        root.saved = lista;
    }

    function close() {
        root.panelVisible = false;
    }

    // O conceito inteiro, ou `null`. A tela le grandezas e vista daqui.
    function currentConcept() {
        for (let i = 0; i < root.concepts.length; ++i) {
            if (root.concepts[i].id === root.selectedConcept) {
                return root.concepts[i];
            }
        }
        return null;
    }

    // Trocar de conceito zera formula, ligacao e valores.
    //
    // Nao e' zelo: uma ligacao feita para "energia cinetica" apontando para as
    // grandezas dela nao significa nada em "lei de Ohm", e mante-la faria a
    // tela mostrar uma ligacao que o core vai recusar — com cara de escolha do
    // autor.
    function selectConcept(id) {
        if (root.selectedConcept === id) {
            return;
        }
        root.selectedConcept = id;
        root.formula = "";
        root.variables = [];
        root.issues = [];
        root.checkOk = false;
        root.caveat = "";
        root.bindings = ({});
        root.values = ({});
        root.clearResult();
        // Metodo, passo e afins vivem no SimRunController e zeram por la:
        // trocar de conceito muda a equacao E como resolve-la.
        root.conceptChanged();
    }

    function clearResult() {
        root.hasResult = false;
        root.resultValue = 0;
        root.resultSteps = [];
        root.resultError = "";
    }

    function editFormula(text) {
        root.formula = text;
        root.clearResult();
        root.scheduleCheck();
    }

    // A ligacao de UMA variavel. `""` desliga.
    function bindVariable(variable, quantityId) {
        const copia = Object.assign({}, root.bindings);
        if (quantityId === "") {
            delete copia[variable];
        } else {
            copia[variable] = quantityId;
        }
        root.bindings = copia;
        root.clearResult();
        root.scheduleCheck();
    }

    function editValue(quantityId, text) {
        const copia = Object.assign({}, root.values);
        copia[quantityId] = text;
        root.values = copia;
        root.clearResult();
    }

    // A lista de ligacoes na forma que o protocolo espera.
    function bindingList() {
        const saida = [];
        for (const variavel in root.bindings) {
            saida.push({ variable: variavel, quantity: root.bindings[variavel] });
        }
        return saida;
    }

    // Os valores na forma do protocolo. So' entra o que foi PREENCHIDO: campo
    // vazio nao vira zero, ele simplesmente nao vai — e o core recusa dizendo
    // qual grandeza falta.
    function valueList() {
        const saida = [];
        for (const grandeza in root.values) {
            const texto = String(root.values[grandeza]).trim();
            if (texto === "") {
                continue;
            }
            const numero = Number(texto);
            if (!isFinite(numero)) {
                continue;
            }
            saida.push({ quantity: grandeza, value: numero });
        }
        return saida;
    }

    // Toda grandeza obrigatoria ligada tem valor? A tela usa isto para habilitar
    // o botao — e nunca para preencher nada.
    function readyToEvaluate() {
        if (!root.checkOk) {
            return false;
        }
        const ligadas = {};
        for (const variavel in root.bindings) {
            ligadas[root.bindings[variavel]] = true;
        }
        for (const grandeza in ligadas) {
            const texto = String(root.values[grandeza] === undefined ? "" : root.values[grandeza]).trim();
            if (texto === "" || !isFinite(Number(texto))) {
                return false;
            }
        }
        return true;
    }

    function evaluate() {
        if (!root.readyToEvaluate()) {
            return;
        }
        root.evaluateRequested(root.selectedConcept, root.formula, root.bindingList(),
                               root.valueList());
    }

    // A checagem roda a cada tecla, com um respiro. O core mede 5,3 us por
    // checagem, entao o custo esta' na ida e volta pelo JSON-RPC, nao na conta;
    // 180 ms e' o mesmo respiro que o observador de arquivos usa (ADR-0001).
    Timer {
        id: respiro

        interval: 180
        onTriggered: {
            if (root.selectedConcept === "") {
                return;
            }
            root.checkRequested(root.selectedConcept, root.formula, root.bindingList());
        }
    }

    function scheduleCheck() {
        if (root.formula.trim() === "") {
            root.variables = [];
            root.issues = [];
            root.checkOk = false;
            root.caveat = "";
            respiro.stop();
            return;
        }
        respiro.restart();
    }

    // --- o que o roteador entrega ------------------------------------------

    function handleCatalog(concepts) {
        root.concepts = concepts;
        root.errorText = "";
    }

    function handleChecked(result) {
        root.variables = result.variables === undefined ? [] : result.variables;
        root.issues = result.issues === undefined ? [] : result.issues;
        root.checkOk = result.ok === true;
        root.caveat = result.caveat === undefined ? "" : result.caveat;
    }

    function handleEvaluated(result) {
        root.hasResult = true;
        root.resultValue = result.value;
        root.resultSteps = result.steps === undefined ? [] : result.steps;
        root.resultError = "";
    }

    function handleFailed(method, message) {
        if (method.indexOf("sim.") !== 0) {
            return;
        }
        if (method === "sim.evaluate") {
            root.resultError = message;
            root.hasResult = false;
            return;
        }
        root.errorText = message;
    }
}
