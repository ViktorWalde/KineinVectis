pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A tela de SIMULACAO POR CONCEITO: escolher, escrever, ligar, preencher, ver.
//
// Burra de proposito, como as outras: nao calcula nada, nao extrai variavel de
// texto, nao decide se a formula bate com o conceito. Tudo isso vem do core.
//
// Ela atende as DUAS montagens, e a diferenca entre elas nao e' cosmetica:
//
//   ESCALAR    uma formula, uma tabela de ligacao, um estado inicial
//   VETORIAL   uma formula e uma tabela POR COMPONENTE, `n` estados iniciais,
//              e o metodo simpletico so' quando o conceito declara o par
//
// Por isso o ramo vetorial mora em arquivos proprios (`SimSystemAuthoring`,
// `SimSystemResultView`) e este painel so' escolhe qual dos dois mostrar. A
// escolha vem do CATALOGO, pelo `system.isSystem` — nunca de olhar a formula.
//
// A vista 3D continua fora: ela depende do `kinein-sim` (`arquitetura/34` §13.6),
// e a trajetoria de uma orbita plana e' 2D de verdade.
Item {
    id: root

    property var concepts: []
    property string selectedConcept: ""
    property string formula: ""
    property var variables: []
    property var issues: []
    property bool checkOk: false
    property string caveat: ""
    property var bindings: ({})
    property var values: ({})
    property bool hasResult: false
    property real resultValue: 0
    property var resultSteps: []
    property string resultError: ""
    property bool readyToEvaluate: false
    property string errorText: ""

    // A parte da corrida (formas EDO). O painel nao decide se o conceito
    // integra: ele recebe a resposta pronta do controller, que a le do
    // CATALOGO — declaracao auditada, nunca deducao a partir da formula.
    property bool integrates: false
    property bool needsInitialDerivative: false
    property string method: ""
    property string stepText: ""
    property string durationText: ""
    property string samplesText: ""
    property string y0Text: ""
    property string dy0Text: ""
    property var estimate: null
    property var run: null
    property string runError: ""
    // A ressalva de procedencia do valor exato, quando ha' uma.
    property string oracleNote: ""
    property bool readyToRun: false
    property var saved: []
    property string saveName: ""
    property bool canSave: false
    // Por que salvar esta' indisponivel, quando esta'. Vazio quando da' para
    // salvar — recusar sem dizer o motivo e' o que esta tela nao faz.
    property string saveNote: ""

    // O `SimSystemController`, quando o conceito escolhido e' vetorial.
    //
    // Entra como controller, e nao achatado em propriedades, porque o que a
    // montagem vetorial le e' POR COMPONENTE e cresceria com o numero deles.
    property var system: null

    readonly property bool isSystem: root.system !== null && root.system.isSystem

    signal conceptSelected(string id)
    signal formulaEdited(string text)
    signal bindRequested(string variable, string quantityId)
    signal valueEdited(string quantityId, string text)
    signal evaluateRequested()
    signal methodChosen(string id)
    signal runFieldEdited(string campo, string texto)
    signal runRequested()
    signal saveNameEdited(string texto)
    signal saveRequested()
    signal loadRequested(var salva)
    signal forgetRequested(string nome)
    signal closeRequested()

    function currentConcept() {
        for (let i = 0; i < root.concepts.length; ++i) {
            if (root.concepts[i].id === root.selectedConcept) {
                return root.concepts[i];
            }
        }
        return null;
    }

    // As grandezas que o autor ja' ligou — sao essas que pedem valor. Uma
    // grandeza que ele nao usou nao vira campo: o formulario segue a formula
    // dele, e nao o catalogo.
    function boundQuantities() {
        const conceito = root.currentConcept();
        if (conceito === null) {
            return [];
        }
        const usadas = {};
        for (const variavel in root.bindings) {
            usadas[root.bindings[variavel]] = true;
        }
        return conceito.quantities.filter(q => usadas[q.id] === true);
    }

    Row {
        anchors.fill: parent
        spacing: Theme.spacingMedium

        // A lista de conceitos e' o painel de ESCOLHER; o resto e' o de MONTAR.
        // Sao as duas metades da tela e as duas metades do trabalho.
        SimConceptList {
            width: Math.round(root.width * 0.32)
            height: parent.height
            concepts: root.concepts
            selectedConcept: root.selectedConcept
            onConceptSelected: id => root.conceptSelected(id)
        }

        // --- a direita: a montagem ----------------------------------------
        Flickable {
            width: parent.width - Math.round(root.width * 0.32) - Theme.spacingMedium
            height: parent.height
            clip: true
            contentHeight: coluna.implicitHeight
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: coluna

                width: parent.width
                spacing: Theme.spacingSmall

                Text {
                    visible: root.errorText !== ""
                    width: parent.width
                    wrapMode: Text.WordWrap
                    text: root.errorText
                    color: Theme.errorSoft
                    font.family: Theme.uiFont
                    font.pixelSize: Theme.fontSizeStatus
                }

                Text {
                    visible: root.selectedConcept === ""
                    width: parent.width
                    wrapMode: Text.WordWrap
                    text: qsTr("Escolha um conceito à esquerda. Você escreve a fórmula; "
                               + "a IDE confere se ela é daquele conceito enquanto você digita.")
                    color: Theme.textMuted
                    font.family: Theme.uiFont
                    font.pixelSize: Theme.fontSizeStatus
                }

                SimFormulaField {
                    width: parent.width
                    visible: root.selectedConcept !== "" && !root.isSystem
                    concept: root.currentConcept()
                    formula: root.formula
                    checkOk: root.checkOk
                    onEdited: text => root.formulaEdited(text)
                }

                SimIssueList {
                    width: parent.width
                    visible: root.selectedConcept !== "" && root.formula.trim() !== ""
                             && !root.isSystem
                    issues: root.issues
                    caveat: root.caveat
                }

                SimBindingTable {
                    width: parent.width
                    visible: root.variables.length > 0 && !root.isSystem
                    variables: root.variables
                    quantities: {
                        const c = root.currentConcept();
                        return c === null ? [] : c.quantities;
                    }
                    bindings: root.bindings
                    onBindRequested: (variavel, grandeza) => root.bindRequested(variavel, grandeza)
                }

                SimValueTable {
                    width: parent.width
                    visible: root.checkOk && !root.isSystem
                    quantities: root.boundQuantities()
                    values: root.values
                    onEdited: (grandeza, texto) => root.valueEdited(grandeza, texto)
                }

                SimSystemAuthoring {
                    width: parent.width
                    visible: root.isSystem
                    controller: root.system
                    values: root.values
                    onValueEdited: (grandeza, texto) => root.valueEdited(grandeza, texto)
                }

                SimRunControls {
                    width: parent.width
                    // A escolha numerica e' a MESMA nas duas formas; o que muda
                    // e' de quem vem o "ja' da' para escolher" e o estado
                    // inicial, que na forma vetorial sao `n` campos e ficam com
                    // a montagem.
                    visible: root.integrates
                             && (root.isSystem ? root.system.checkOk : root.checkOk)
                    showScalarInitial: !root.isSystem
                    symplecticReason: root.isSystem ? root.system.symplecticReason : ""
                    method: root.method
                    stepText: root.stepText
                    durationText: root.durationText
                    samplesText: root.samplesText
                    y0Text: root.y0Text
                    dy0Text: root.dy0Text
                    needsInitialDerivative: root.needsInitialDerivative
                    estimate: root.estimate
                    onMethodChosen: id => root.methodChosen(id)
                    onFieldEdited: (campo, texto) => root.runFieldEdited(campo, texto)
                }

                SimActions {
                    width: parent.width
                    integrates: root.integrates
                    disabledHint: root.isSystem
                                  ? qsTr("preencha as equações, as ligações, os valores, "
                                         + "o estado inicial, o método e o passo")
                                  : ""
                    canEvaluate: root.readyToEvaluate
                    canRun: root.readyToRun
                    onEvaluateRequested: root.evaluateRequested()
                    onRunRequested: root.runRequested()
                    onCloseRequested: root.closeRequested()
                }

                SimCalculation {
                    width: parent.width
                    visible: !root.integrates && !root.isSystem
                    steps: root.resultSteps
                    hasResult: root.hasResult
                    resultValue: root.resultValue
                    errorText: root.resultError
                }

                SimSavedList {
                    width: parent.width
                    visible: root.selectedConcept !== "" || root.saved.length > 0
                    saved: root.saved
                    saveName: root.saveName
                    canSave: root.canSave
                    note: root.saveNote
                    onNameEdited: t => root.saveNameEdited(t)
                    onSaveRequested: root.saveRequested()
                    onLoadRequested: s => root.loadRequested(s)
                    onForgetRequested: n => root.forgetRequested(n)
                }

                SimRunResultView {
                    width: parent.width
                    visible: root.integrates && !root.isSystem
                             && (root.run !== null || root.runError !== "")
                    run: root.run
                    errorText: root.runError
                    oracleNote: root.oracleNote
                }

                SimSystemResultView {
                    width: parent.width
                    visible: root.isSystem && root.system !== null
                             && (root.system.runResult !== null
                                 || root.system.runError !== "")
                    controller: root.system
                }
            }
        }
    }
}
