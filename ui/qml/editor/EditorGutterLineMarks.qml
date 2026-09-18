pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As marcas da faixa ESQUERDA de uma linha da calha: a barra do diff do
// git, a barra da cobertura (P5), a seta de dobra e o ponto do breakpoint.
// Saiu do EditorGutter em 2026-09-17, quando a cobertura e a lampada o
// levaram a 300 linhas — a calha continua dona das faixas e dos numeros;
// aqui e' so' a pintura de UMA linha, sem estado.
Item {
    id: root

    property string diffKind: ""
    property string coverageKind: ""
    property bool foldable: false
    property bool folded: false
    property bool hasBreakpoint: false
    property int breakpointLeft: 0
    property int breakpointSize: 8

    signal foldToggleRequested()

    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        width: 3
        height: root.diffKind === "removed" ? 3 : parent.height
        visible: root.diffKind !== ""
        color: root.diffKind === "added" ? Theme.successSoft
               : (root.diffKind === "modified" ? Theme.infoSoft : Theme.errorSoft)
    }

    // A cobertura: uma segunda barra de 3px, a direita da do diff — verde
    // coberta, vermelha instrumentada e nunca executada.
    Rectangle {
        anchors.left: parent.left
        anchors.leftMargin: 4
        anchors.top: parent.top
        width: 3
        height: parent.height
        visible: root.coverageKind !== ""
        color: root.coverageKind === "covered" ? Theme.successSoft : Theme.errorSoft
    }

    Text {
        anchors.left: parent.left
        anchors.leftMargin: 5
        anchors.verticalCenter: parent.verticalCenter
        visible: root.foldable
        text: root.folded ? "▸" : "▾"
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeEditor - 3

        MouseArea {
            anchors.fill: parent
            anchors.margins: -4
            cursorShape: Qt.PointingHandCursor
            onClicked: function(mouse) {
                root.foldToggleRequested();
                mouse.accepted = true;
            }
        }
    }

    Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: root.breakpointLeft
        width: root.breakpointSize
        height: root.breakpointSize
        radius: root.breakpointSize / 2
        visible: root.hasBreakpoint
        color: Theme.errorSoft
        border.width: 1
        border.color: Theme.background0
    }
}
