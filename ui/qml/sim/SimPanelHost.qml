pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de simulacao dentro de um dialogo, como os outros overlays.
//
// Separa o CONTEUDO (SimPanel, burro) do CHROME. Mesmo desenho do
// DataSourcePanelHost e do LibraryPanelHost.
Item {
    id: root

    property var controller: null
    property var runController: null
    // A AUTORIA da forma vetorial. Os tres sao donos diferentes e e' aqui que
    // eles se encontram: o `controller` guarda o conceito e os valores dos
    // parametros, o `systemController` guarda as `n` equacoes e o estado
    // inicial, e o `runController` guarda a escolha numerica — que e' a mesma
    // nas duas formas.
    property var systemController: null
    property real maxAvailableWidth: 900
    property real maxAvailableHeight: 640

    signal dismissRequested()

    function clearSystemRun() {
        if (root.systemController) {
            root.systemController.clearRun();
        }
    }

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        // Dimensionamento AUTOMATICO pelo conteudo, com PISO e com o teto da
        // janela (decisao do autor, 2026-09-04). A tabela de ligacao cresce com
        // o numero de variaveis, entao altura fixa cortaria conteudo.
        width: Math.min(Math.max(720, root.maxAvailableWidth * 0.72), root.maxAvailableWidth)
        height: Math.min(Math.max(460, root.maxAvailableHeight * 0.78), root.maxAvailableHeight)
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        SimPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium

            concepts: root.controller ? root.controller.concepts : []
            selectedConcept: root.controller ? root.controller.selectedConcept : ""
            formula: root.controller ? root.controller.formula : ""
            variables: root.controller ? root.controller.variables : []
            issues: root.controller ? root.controller.issues : []
            checkOk: root.controller ? root.controller.checkOk : false
            caveat: root.controller ? root.controller.caveat : ""
            bindings: root.controller ? root.controller.bindings : ({})
            values: root.controller ? root.controller.values : ({})
            hasResult: root.controller ? root.controller.hasResult : false
            resultValue: root.controller ? root.controller.resultValue : 0
            resultSteps: root.controller ? root.controller.resultSteps : []
            resultError: root.controller ? root.controller.resultError : ""
            readyToEvaluate: root.controller ? root.controller.readyToEvaluate() : false
            errorText: root.controller ? root.controller.errorText : ""

            system: root.systemController
            // A forma vetorial INTEGRA como as EDO escalares: o botao diz
            // "Integrar", nao "Calcular".
            integrates: root.systemController && root.systemController.isSystem
                        ? true
                        : (root.runController ? root.runController.integrates() : false)
            needsInitialDerivative: root.runController
                                    ? root.runController.needsInitialDerivative() : false
            method: root.runController ? root.runController.method : ""
            stepText: root.runController ? root.runController.stepText : ""
            durationText: root.runController ? root.runController.durationText : ""
            samplesText: root.runController ? root.runController.samplesText : ""
            y0Text: root.runController ? root.runController.y0Text : ""
            dy0Text: root.runController ? root.runController.dy0Text : ""
            estimate: root.runController ? root.runController.estimate : null
            run: root.runController ? root.runController.run : null
            runError: root.runController ? root.runController.runError : ""
            oracleNote: root.runController && root.runController.run
                        && root.runController.run.oracleNote !== undefined
                        ? root.runController.run.oracleNote : ""
            // Rodar um sistema exige as DUAS metades prontas: a montagem (`n`
            // equacoes checadas, `n` iniciais, valores) e a numerica (metodo,
            // passo, duracao, amostras). Cada dono confere a sua; juntar as
            // duas e' trabalho de fiacao, e e' aqui que ela mora.
            readyToRun: root.systemController && root.systemController.isSystem
                        ? (root.systemController.readyToAuthor(
                               root.controller ? root.controller.values : ({}))
                           && root.runController !== null
                           && root.runController.numerics() !== null)
                        : (root.runController ? root.runController.readyToRun() : false)
            saved: root.controller ? root.controller.saved : []
            saveName: root.controller ? root.controller.saveName : ""
            // Salvar um SISTEMA ainda nao existe, e a tela diz isso em vez de
            // gravar um arquivo incompleto: o `SimSaved` do protocolo carrega
            // UMA formula e UMA ligacao, e um sistema tem `n` de cada. Gravar
            // assim escreveria uma montagem que nao volta.
            canSave: root.controller
                     ? (root.controller.saveName.trim() !== ""
                        && root.controller.selectedConcept !== ""
                        && !(root.systemController && root.systemController.isSystem))
                     : false
            saveNote: root.systemController && root.systemController.isSystem
                      ? qsTr("Salvar ainda não vale para conceitos de várias equações: "
                             + "o arquivo de simulação guarda uma fórmula, e este "
                             + "conceito tem uma por componente.")
                      : ""

            onConceptSelected: id => root.controller.selectConcept(id)
            onFormulaEdited: text => root.controller.editFormula(text)
            onBindRequested: (variavel, grandeza) => root.controller.bindVariable(variavel, grandeza)
            // Mexer no que DECIDE o resultado invalida o resultado. Cada
            // controller ja' limpa o que e' dele; aqui a limpeza atravessa a
            // fronteira, porque o valor de um parametro e a escolha numerica
            // moram num dono e a corrida do sistema mora noutro. Sem isto a
            // tela continua mostrando o grafico da conta ANTERIOR, que e'
            // exatamente a forma de mentira que este dominio persegue.
            onValueEdited: (grandeza, texto) => {
                root.controller.editValue(grandeza, texto);
                root.clearSystemRun();
            }
            onEvaluateRequested: root.controller.evaluate()
            onMethodChosen: id => {
                root.runController.chooseMethod(id);
                root.clearSystemRun();
            }
            onRunFieldEdited: (campo, texto) => {
                root.runController.editRunField(campo, texto);
                root.clearSystemRun();
            }
            onRunRequested: {
                if (root.systemController && root.systemController.isSystem) {
                    root.systemController.requestRun(root.controller.valueList(),
                                                     root.runController.numerics());
                } else {
                    root.runController.runSimulation();
                }
            }
            onSaveNameEdited: t => root.controller.saveName = t
            onSaveRequested: root.controller.save()
            onLoadRequested: s => root.controller.load(s)
            onForgetRequested: n => root.controller.forget(n)
            onCloseRequested: root.dismissRequested()
        }
    }
}
