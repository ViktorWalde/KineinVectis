// Simulacao por conceito: a LIGACAO decide, e campo vazio nao vira zero.
//
// Por que existe: o `SimController` guarda a montagem do autor — conceito,
// formula, ligacao de cada variavel e valores — e decide quando a conta pode
// rodar. Se essa decisao quebrar, nenhum compilador reclama: a IDE so passa a
// habilitar "Calcular" com campo em branco, ou a mandar ao core uma ligacao
// que nao e' a que esta' na tela.
//
// O ALVO PRINCIPAL e' o principio da arquitetura/34 §2.1: **nada e'
// adivinhado**. Em concreto, tres coisas que este harness prova:
//   1. campo vazio NAO vira zero — a conta nao parte, e o valor nem e' enviado;
//   2. trocar de conceito ZERA ligacao e valores, porque uma ligacao feita para
//      outro conceito nao significa nada aqui e ficaria com cara de escolha;
//   3. a variavel sem papel bloqueia a conta, mesmo com a formula valida.
import QtQuick
import "../../ui/qml/sim"

Item {
    id: root

    property var checagens: []
    property var avaliacoes: []
    property int catalogos: 0
    property var salvamentos: []

    SimController {
        id: controller

        onCatalogRequested: root.catalogos += 1
        onSaveRequested: function (simulation) { root.salvamentos.push(simulation); }
        onCheckRequested: function (conceptId, formula, bindings) {
            root.checagens.push({ concept: conceptId, formula: formula, bindings: bindings });
        }
        onEvaluateRequested: function (conceptId, formula, bindings, values) {
            root.avaliacoes.push({ concept: conceptId, formula: formula,
                                   bindings: bindings, values: values });
        }
    }

    // O SHAPE E' O DO CORE, campo por campo — o mesmo que `sim.catalog` devolve.
    // Fixture inventada testaria um protocolo que nao existe.
    function catalogo() {
        return [
            { id: "energia-cinetica", name: "Energia cinetica", summary: "...",
              course: "Fisica basica", form: "algebraic", view: "plot2d",
              closedForm: true, source: "Halliday; revisado em 2026-09-05",
              quantities: [
                  { id: "m", label: "massa", unit: "kg", required: true },
                  { id: "v", label: "velocidade", unit: "m/s", required: true }
              ] },
            { id: "lei-de-ohm", name: "Lei de Ohm", summary: "...",
              course: "Fisica basica", form: "algebraic", view: "plot2d",
              closedForm: true, source: "Halliday; revisado em 2026-09-05",
              quantities: [
                  { id: "i", label: "corrente", unit: "A", required: true },
                  { id: "r", label: "resistencia", unit: "ohm", required: true }
              ] }
        ];
    }

    Component.onCompleted: {
        let failures = 0;

        // Abrir pede o catalogo uma vez; abrir de novo com catalogo em maos nao
        // repete o pedido.
        controller.open();
        if (root.catalogos !== 1) failures += 1;
        controller.handleCatalog(root.catalogo());
        controller.close();
        controller.open();
        if (root.catalogos !== 1) failures += 2;

        controller.selectConcept("energia-cinetica");
        if (controller.currentConcept() === null) failures += 4;
        if (controller.currentConcept().quantities.length !== 2) failures += 8;

        // A UI NAO extrai variavel de texto: ela so' sabe o que o core disse.
        controller.editFormula("0.5*m*v^2");
        if (controller.variables.length !== 0) failures += 16;
        controller.handleChecked({
            ok: false, variables: ["m", "v"], issues: [
                { kind: "unboundVariable", variable: "m" },
                { kind: "unboundVariable", variable: "v" }
            ]
        });
        if (controller.variables.length !== 2) failures += 32;
        if (controller.checkOk) failures += 64;

        // Ligar cada variavel. A lista que vai ao core sai da LIGACAO, e nao da
        // ordem em que as variaveis chegaram.
        controller.bindVariable("m", "m");
        controller.bindVariable("v", "v");
        const lig = controller.bindingList();
        if (lig.length !== 2) failures += 128;
        let achouM = false;
        let achouV = false;
        for (let i = 0; i < lig.length; ++i) {
            if (lig[i].variable === "m" && lig[i].quantity === "m") achouM = true;
            if (lig[i].variable === "v" && lig[i].quantity === "v") achouV = true;
        }
        if (!achouM || !achouV) failures += 256;

        controller.handleChecked({
            ok: true, variables: ["m", "v"], issues: [],
            caveat: "Conceito certo e formula valida NAO significam resultado certo."
        });
        if (!controller.checkOk) failures += 512;
        if (controller.caveat === "") failures += 1024;

        // CAMPO VAZIO NAO VIRA ZERO. Sem valores, a conta nao parte.
        if (controller.readyToEvaluate()) failures += 2048;
        controller.evaluate();
        if (root.avaliacoes.length !== 0) failures += 4096;

        // Um valor so' tambem nao basta.
        controller.editValue("m", "2");
        if (controller.readyToEvaluate()) failures += 8192;

        // Valor nao-numerico e' tao "nao preenchido" quanto vazio.
        controller.editValue("v", "abc");
        if (controller.readyToEvaluate()) failures += 16384;

        controller.editValue("v", "3");
        if (!controller.readyToEvaluate()) failures += 32768;
        controller.evaluate();
        if (root.avaliacoes.length !== 1) {
            failures += 65536;
        } else {
            const pedido = root.avaliacoes[0];
            if (pedido.values.length !== 2) failures += 131072;
            // Os valores viajam como NUMERO, nao como o texto do campo.
            for (let j = 0; j < pedido.values.length; ++j) {
                if (typeof pedido.values[j].value !== "number") failures += 262144;
            }
        }

        // LIMPAR UM CAMPO nao faz o valor virar zero na lista que vai ao core.
        // Este e' o caminho real: o autor digita, apaga, e o `values` fica com
        // string vazia guardada. Sem o filtro, `Number("")` e' 0 e a IDE
        // mandaria um zero que o autor nao escreveu.
        controller.editValue("v", "");
        if (controller.readyToEvaluate()) failures += 262144 * 2;
        const listaComVazio = controller.valueList();
        for (let k = 0; k < listaComVazio.length; ++k) {
            if (listaComVazio[k].quantity === "v") failures += 262144 * 4;
        }
        controller.editValue("v", "3");

        // O resultado chega e a trilha vem junto.
        controller.handleEvaluated({
            value: 9, steps: [
                { kind: "formula", text: "Energia cinetica = 0.5*m*v^2" },
                { kind: "substitution", text: "0.5*2*3^2" },
                { kind: "result", text: "= 9" }
            ]
        });
        if (!controller.hasResult) failures += 524288;
        if (controller.resultSteps.length !== 3) failures += 1048576;

        // Editar qualquer coisa APAGA o resultado: um numero na tela que nao
        // corresponde mais a' formula em cima dele e' pior que nenhum numero.
        controller.editValue("v", "4");
        if (controller.hasResult) failures += 2097152;

        // TROCAR DE CONCEITO ZERA TUDO. Uma ligacao para "energia cinetica"
        // apontando para `m` e `v` nao significa nada na "lei de Ohm", e
        // mante-la mostraria uma escolha que o autor nao fez.
        controller.selectConcept("lei-de-ohm");
        if (controller.formula !== "") failures += 4194304;
        if (controller.bindingList().length !== 0) failures += 8388608;
        if (controller.valueList().length !== 0) failures += 16777216;
        if (controller.checkOk) failures += 33554432;
        if (controller.hasResult) failures += 67108864;

        // Erro de OUTRO dominio nao acende o erro desta tela.
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 134217728;
        // Recusa da avaliacao vai para o painel do calculo, nao para o topo.
        controller.handleFailed("sim.evaluate", "o resultado nao e um numero utilizavel");
        if (controller.resultError === "") failures += 268435456;
        if (controller.errorText !== "") failures += 536870912;

        // --- salvar e carregar --------------------------------------------
        //
        // O que vai ao disco e' o que o autor MONTOU. Se alguem acrescentar o
        // resultado a` montagem, a assercao abaixo cai — e ela existe porque a
        // licao do `.ipynb` e' que texto diffavel nao basta: o que quebra e'
        // misturar o autoral com a saida da maquina.
        controller.handleCatalog(root.catalogo());
        controller.selectConcept("energia-cinetica");
        controller.editFormula("0.5*m*v^2");
        controller.handleChecked({ ok: true, variables: ["m", "v"], issues: [] });
        controller.bindVariable("m", "m");
        controller.bindVariable("v", "v");
        controller.editValue("m", "2");
        controller.editValue("v", "3");

        // Sem nome, nao salva.
        controller.save();
        if (root.salvamentos.length !== 0) failures += 1073741824;

        controller.saveName = "meu teste";
        controller.save();
        if (root.salvamentos.length !== 1) {
            failures += 2147483648;
        } else {
            const m = root.salvamentos[0];
            if (m.name !== "meu teste") failures += 4294967296;
            if (m.concept !== "energia-cinetica") failures += 8589934592;
            if (m.formula !== "0.5*m*v^2") failures += 17179869184;
            if (m.bindings.length !== 2) failures += 34359738368;
            if (m.values.length !== 2) failures += 68719476736;
            // O RESULTADO NAO VAI: nem trilha, nem exatidao, nem passos.
            for (const proibido of ["trail", "accuracy", "stepsTaken", "steps", "result"]) {
                if (m[proibido] !== undefined) failures += 137438953472;
            }
        }

        // Carregar repoe a montagem inteira.
        controller.selectConcept("lei-de-ohm");
        controller.handleSavedList([{
            name: "meu teste", concept: "energia-cinetica", formula: "0.5*m*v^2",
            bindings: [{ variable: "m", quantity: "m" }, { variable: "v", quantity: "v" }],
            values: [{ quantity: "m", value: 2 }, { quantity: "v", value: 3 }]
        }]);
        if (controller.saved.length !== 1) failures += 274877906944;
        controller.load(controller.saved[0]);
        if (controller.selectedConcept !== "energia-cinetica") failures += 549755813888;
        if (controller.formula !== "0.5*m*v^2") failures += 1099511627776;
        if (controller.bindingList().length !== 2) failures += 2199023255552;
        // Os valores voltam como TEXTO no campo, e como numero na lista.
        if (controller.valueList().length !== 2) failures += 4398046511104;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
