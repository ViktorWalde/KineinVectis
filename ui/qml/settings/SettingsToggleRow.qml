import QtQuick
import KineinVectis

// Uma preferencia de liga/desliga: rotulo e explicacao a esquerda, o
// interruptor a direita. A LINHA INTEIRA e' o alvo do clique e acende ao
// pairar (2026-10-03, o redesenho das Configuracoes: antes so' a bolinha
// respondia, e nada dizia que o resto da linha nao era clicavel).
//
// `fromProject`: o valor vem do .kinein/settings.json do projeto — o selo
// diz isso, porque e' ali que a troca vai ser gravada.
Rectangle {
    id: root

    property string label: ""
    property string hint: ""
    property bool checked: false
    property bool fromProject: false

    signal toggled()

    implicitHeight: texts.implicitHeight + 2 * Theme.spacingMedium
    radius: Theme.radius
    color: area.containsMouse ? Theme.surface2 : "transparent"
    Accessible.role: Accessible.CheckBox
    Accessible.name: root.label
    Accessible.checked: root.checked

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    Column {
        id: texts

        anchors.left: parent.left
        anchors.right: track.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        Row {
            spacing: Theme.spacingSmall

            Text {
                text: root.label
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeMedium
            }

            SettingsProjectBadge {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.fromProject
            }
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

    // O interruptor: ambar e bolinha a direita quando ligado.
    Rectangle {
        id: track

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        width: 36
        height: 20
        radius: height / 2
        color: root.checked ? Theme.accent : Theme.surface1
        border.color: root.checked ? Theme.accent : (area.containsMouse ? Theme.textMuted : Theme.borderStrong)
        border.width: 1

        Behavior on color {
            ColorAnimation { duration: Theme.motionFast }
        }

        Rectangle {
            width: 14
            height: 14
            radius: height / 2
            anchors.verticalCenter: parent.verticalCenter
            x: root.checked ? parent.width - width - 3 : 3
            color: root.checked ? Theme.background0 : Theme.textSecondary

            Behavior on x {
                NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
            }
        }
    }

    MouseArea {
        id: area

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.toggled()
    }
}
