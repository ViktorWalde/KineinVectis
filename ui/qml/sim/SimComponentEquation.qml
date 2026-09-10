pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// UMA equacao do sistema: `d<componente>/dt = ...`, com a ligacao dela.
//
// Por que ela nao e' o `SimFormulaField`. Aquele traz o resumo e a fonte do
// CONCEITO em cima do campo, e num sistema isso apareceria `n` vezes, dizendo
// `n` vezes a mesma coisa. O que muda entre as linhas e' o COMPONENTE — entao o
// cabecalho aqui e' ele, e o resumo do conceito fica uma vez so', no
// `SimSystemAuthoring`.
//
// Burra como as vizinhas: nao sabe o que e' uma formula, nao extrai variavel de
// texto e nao decide se a equacao esta' certa. Tudo isso vem do core.
Column {
    id: root

    // O componente que esta linha deriva: { id, label, unit }.
    property var stateComponent: null
    property string formula: ""
    // O veredito do core para ESTA equacao.
    property bool checkOk: false
    // As variaveis que o core achou nesta formula, e os problemas dela.
    property var variables: []
    property var issues: []
    // Os alvos de ligacao validos: grandezas, componentes de estado e o tempo.
    property var targets: []
    // A ligacao desta equacao, como mapa { variavel: alvo }.
    property var bindings: ({})

    signal edited(string texto)
    signal bindRequested(string variable, string target)

    spacing: Theme.spacingXSmall

    // O cabecalho DIZ que o campo e' a derivada, e nao o valor. `dx/dt` e `x`
    // sao coisas diferentes, e a forma vetorial e' de primeira ordem
    // justamente porque o autor escreve a derivada de cada componente
    // (`arquitetura/34` §13.1).
    Row {
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.stateComponent === null
                  ? "" : "d" + root.stateComponent.id + "/dt  ="
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizePanelTitle
            font.bold: true
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.stateComponent === null ? "" :
                  (root.stateComponent.unit === ""
                   ? root.stateComponent.label
                   : root.stateComponent.label + " (" + root.stateComponent.unit + ")")
            color: Theme.textMuted
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    Rectangle {
        width: root.width
        height: 32
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
        // A mesma regra da forma escalar: vazio nao e' erro, e' "ainda nao"; o
        // verde e o ambar vem do CORE, nunca de uma checagem local.
        border.color: root.formula.trim() === ""
                      ? Theme.borderSoft
                      : (root.checkOk ? Theme.successSoft : Theme.warningSoft)

        TextInput {
            id: entrada

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            verticalAlignment: TextInput.AlignVCenter
            clip: true
            text: root.formula
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeEditor
            selectByMouse: true
            selectionColor: Theme.accentDim

            onTextEdited: root.edited(entrada.text)

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: entrada.text === ""
                text: qsTr("escreva a derivada deste componente")
                color: Theme.textDisabled
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeEditor
            }
        }
    }

    SimIssueList {
        width: root.width
        visible: root.issues.length > 0
        issues: root.issues
        // O aviso da §5.1 sai UMA vez, no rodape da montagem, e nao uma vez por
        // equacao: repetido `n` vezes ele vira ruido e para de ser lido.
        caveat: ""
    }

    SimBindingTable {
        width: root.width
        visible: root.variables.length > 0
        variables: root.variables
        quantities: root.targets
        bindings: root.bindings
        onBindRequested: (variavel, alvo) => root.bindRequested(variavel, alvo)
    }

    Item {
        width: 1
        height: Theme.spacingSmall
    }
}
