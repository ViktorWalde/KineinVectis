pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O campo da FORMULA, com o resumo do conceito em cima.
//
// A borda muda de cor pelo `checkOk`, que vem do CORE — a UI nao decide se a
// formula esta' certa, ela desenha o veredito.
Column {
    id: root

    property var concept: null
    property string formula: ""
    property bool checkOk: false

    signal edited(string text)

    spacing: Theme.spacingXSmall

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        text: root.concept === null ? "" : root.concept.summary
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Text {
        width: root.width
        wrapMode: Text.WordWrap
        // A FONTE, com data. Mesma regra do `setup.list`: sem fonte, a IDE nao
        // afirma — e mostrar a fonte e' o que separa "a IDE diz" de "alguem
        // com nome respondeu".
        text: root.concept === null ? "" : qsTr("Formulação: %1").arg(root.concept.source)
        color: Theme.textDisabled
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus - 1
    }

    Text {
        text: qsTr("A fórmula")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
        topPadding: Theme.spacingXSmall
    }

    Rectangle {
        width: root.width
        height: 34
        radius: Theme.radiusXSmall
        color: Theme.backgroundEditor
        border.width: 1
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
                text: qsTr("escreva a equação deste conceito")
                color: Theme.textDisabled
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeEditor
            }
        }
    }
}
