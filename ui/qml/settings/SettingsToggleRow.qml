import QtQuick
import KineinVectis

// Linha de setting booleano (fatia M4.1): label + hint à esquerda, pill
// switch à direita. Sem QtQuick.Controls — o switch é próprio.
Row {
    id: root

    property string label: ""
    property string hint: ""
    property bool checked: false

    signal toggled()

    spacing: Theme.spacingMedium

    Column {
        width: root.width - toggle.width - Theme.spacingMedium
        spacing: 2

        Text {
            text: root.label
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeMedium
        }

        Text {
            width: parent.width
            visible: root.hint !== ""
            text: root.hint
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            wrapMode: Text.WordWrap
        }
    }

    Rectangle {
        id: toggle

        anchors.verticalCenter: parent.verticalCenter
        width: 40
        height: 22
        radius: height / 2
        color: root.checked ? Theme.accent : Theme.surface1
        border.color: root.checked ? Theme.accent : Theme.borderStrong
        border.width: 1

        Rectangle {
            width: 16
            height: 16
            radius: height / 2
            anchors.verticalCenter: parent.verticalCenter
            x: root.checked ? parent.width - width - 3 : 3
            color: root.checked ? Theme.background0 : Theme.textSecondary

            Behavior on x {
                NumberAnimation {
                    duration: Theme.motionFast
                    easing.type: Theme.easingStandard
                }
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.toggled()
        }
    }
}
