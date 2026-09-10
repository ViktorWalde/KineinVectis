pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O PAINEL DO CALCULO: a formula, a substituicao e o resultado.
//
// Isto e' o "passo a passo" que existe de verdade. Nenhuma ferramenta auditavel
// narra a resolucao algebrica (roadmaps/31 §13): o Maxima marcou o pedido como
// "won't fix", o SymPy nao tem para EDO, e o Wolfram tem com heuristica
// propria gerada POR FORA do motor que calcula. O que a Kinein mostra e' a
// SUBSTITUICAO NUMERICA — literalmente a conta que rodou, impossivel de
// divergir do resultado porque ELA E' o resultado.
Column {
    id: root

    property var steps: []
    property bool hasResult: false
    property real resultValue: 0
    property string errorText: ""

    spacing: Theme.spacingXSmall

    Text {
        text: qsTr("O cálculo")
        color: Theme.textPrimary
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Rectangle {
        visible: root.errorText !== ""
        width: root.width
        height: erroTexto.implicitHeight + 2 * Theme.spacingSmall
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.errorSoft

        Text {
            id: erroTexto

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            anchors.margins: Theme.spacingSmall
            wrapMode: Text.WordWrap
            text: root.errorText
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }
    }

    Text {
        visible: !root.hasResult && root.errorText === ""
        width: root.width
        wrapMode: Text.WordWrap
        text: qsTr("Escolha o conceito, escreva a fórmula, diga o que cada "
                   + "variável é e preencha os valores. Nada é preenchido por você.")
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }

    Repeater {
        model: root.hasResult ? root.steps : []

        delegate: Text {
            required property var modelData

            width: root.width
            wrapMode: Text.WrapAnywhere
            text: modelData.text
            color: modelData.kind === "result" ? Theme.textPrimary : Theme.textSecondary
            font.family: modelData.kind === "formula" ? Theme.uiFont : Theme.monoFont
            font.pixelSize: Theme.fontSizeStatus
            font.bold: modelData.kind === "result"
            topPadding: modelData.kind === "substitution" || modelData.kind === "result"
                        ? Theme.spacingXSmall : 0
        }
    }
}
