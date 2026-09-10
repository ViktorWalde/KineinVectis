pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os botoes. "Calcular" so' habilita quando TUDO que decide o resultado esta'
// preenchido — e a razao de estar desabilitado fica escrita ao lado, em vez de
// o autor ficar clicando num botao morto.
Row {
    id: root

    property bool canEvaluate: false
    // Formas EDO INTEGRAM; a algebrica AVALIA. Sao acoes diferentes, e o botao
    // diz qual e' — "Calcular" numa EDO deixaria ambiguo se ela vai andar no
    // tempo ou so' avaliar a derivada uma vez.
    property bool integrates: false
    property bool canRun: false
    // O que ainda falta, quando o botao esta' desabilitado. Vazio usa a frase
    // padrao; a forma vetorial troca porque o que falta nela e' outro — `n`
    // formulas e `n` estados iniciais, nao uma de cada.
    property string disabledHint: ""

    signal evaluateRequested()
    signal runRequested()
    signal closeRequested()

    spacing: Theme.spacingSmall

    Rectangle {
        readonly property bool habilitado: root.integrates ? root.canRun : root.canEvaluate

        width: 108
        height: 30
        radius: Theme.radiusXSmall
        color: habilitado ? Theme.accent : Theme.background2
        border.width: 1
        border.color: habilitado ? Theme.accentActive : Theme.borderSoft

        Text {
            anchors.centerIn: parent
            text: root.integrates ? qsTr("Integrar") : qsTr("Calcular")
            color: parent.habilitado ? Theme.background0 : Theme.textDisabled
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }

        MouseArea {
            anchors.fill: parent
            enabled: parent.habilitado
            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: root.integrates ? root.runRequested() : root.evaluateRequested()
        }
    }

    Rectangle {
        width: 82
        height: 30
        radius: Theme.radiusXSmall
        color: Theme.background2
        border.width: 1
        border.color: Theme.borderSoft

        Text {
            anchors.centerIn: parent
            text: qsTr("Fechar")
            color: Theme.textSecondary
            font.family: Theme.uiFont
            font.pixelSize: Theme.fontSizeStatus
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.closeRequested()
        }
    }

    Text {
        anchors.verticalCenter: parent.verticalCenter
        visible: root.integrates ? !root.canRun : !root.canEvaluate
        text: root.disabledHint !== "" ? root.disabledHint
              : (root.integrates
                 ? qsTr("preencha a fórmula, as ligações, os valores, o método e o passo")
                 : qsTr("preencha a fórmula, as ligações e os valores"))
        color: Theme.textMuted
        font.family: Theme.uiFont
        font.pixelSize: Theme.fontSizeStatus
    }
}
