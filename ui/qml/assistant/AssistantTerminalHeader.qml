import QtQuick
import KineinVectis

Rectangle {
    id: root

    property string profileName: ""
    property string command: ""
    property bool maximized: false

    signal maximizeToggleRequested()
    signal switchRequested()
    signal exitRequested()

    implicitHeight: 40
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    KvIcon {
        id: terminalIcon

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        name: "terminal"
        size: 18
        active: true
    }

    Column {
        anchors.left: terminalIcon.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: maximizeButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        spacing: 1

        Text {
            width: parent.width
            text: root.profileName
            color: Theme.textPrimary
            font.pixelSize: 12
            font.bold: true
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.command
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 9
            elide: Text.ElideMiddle
        }
    }

    KvIconButton {
        id: maximizeButton

        anchors.right: switchButton.left
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 28
        height: 28
        iconName: root.maximized ? "collapse" : "expand"
        iconSize: 16
        tooltip: root.maximized
                 ? qsTr("Restaurar painel") : qsTr("Ampliar terminal")
        onClicked: root.maximizeToggleRequested()
    }

    KvIconButton {
        id: switchButton

        anchors.right: exitButton.left
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 28
        height: 28
        iconName: "refresh"
        iconSize: 16
        tooltip: qsTr("Trocar Claude/Codex")
        onClicked: root.switchRequested()
    }

    KvIconButton {
        id: exitButton

        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 28
        height: 28
        iconName: "stop"
        iconSize: 15
        tooltip: qsTr("Encerrar sessão")
        onClicked: root.exitRequested()
    }
}
