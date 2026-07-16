import QtQuick
import KineinVectis

Row {
    id: root

    property bool maximized: false

    signal minimizeRequested()
    signal maximizeRestoreRequested()
    signal closeRequested()

    spacing: 0

    KvIconButton {
        objectName: "windowMinimizeButton"
        width: 32
        height: root.height
        radius: 0
        iconName: "minimize"
        iconSize: 16
        tooltip: qsTr("Minimizar janela")
        accessibleName: qsTr("Minimizar janela")
        activeFocusOnTab: true
        onClicked: root.minimizeRequested()
    }

    KvIconButton {
        objectName: "windowMaximizeRestoreButton"
        width: 32
        height: root.height
        radius: 0
        iconName: root.maximized ? "restore" : "maximize"
        iconSize: 16
        tooltip: root.maximized ? qsTr("Restaurar janela")
                                  : qsTr("Maximizar janela")
        accessibleName: tooltip
        activeFocusOnTab: true
        onClicked: root.maximizeRestoreRequested()
    }

    KvIconButton {
        objectName: "windowCloseButton"
        width: 32
        height: root.height
        radius: 0
        iconName: "close"
        iconSize: 16
        tooltip: qsTr("Fechar janela")
        accessibleName: qsTr("Fechar janela")
        activeFocusOnTab: true
        danger: true
        onClicked: root.closeRequested()
    }
}
