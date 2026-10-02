import QtQuick
import KineinVectis

// A barra do navegador (0.3.9): voltar, avancar e subir a esquerda, o caminho
// em migalhas no meio, e a direita "nova pasta" e "mostrar ocultas" — a
// arrumacao do seletor da JetBrains, sem botao "Ir" (Enter no campo navega).
Item {
    id: root

    property var controller

    signal pathEditingFinished()

    height: 32

    function beginEditing(seed) {
        breadcrumb.beginEditing(seed);
    }

    Row {
        id: navigation

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        KvIconButton {
            iconName: "back"
            iconSize: 16
            tooltip: qsTr("Voltar")
            focus: false
            enabled: root.controller.backStack.length > 0
            onClicked: root.controller.goBack()
        }

        KvIconButton {
            iconName: "back"
            iconRotation: 180
            iconSize: 16
            tooltip: qsTr("Avançar")
            focus: false
            enabled: root.controller.forwardStack.length > 0
            onClicked: root.controller.goForward()
        }

        KvIconButton {
            iconName: "back"
            iconRotation: 90
            iconSize: 16
            tooltip: qsTr("Pasta de cima (Backspace)")
            focus: false
            enabled: root.controller.parentPath !== ""
            onClicked: root.controller.browsePath(root.controller.parentPath)
        }
    }

    FolderPickerBreadcrumb {
        id: breadcrumb

        anchors.left: navigation.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.right: tools.left
        anchors.rightMargin: Theme.spacingSmall
        height: parent.height
        controller: root.controller
        onEditingFinished: root.pathEditingFinished()
    }

    Row {
        id: tools

        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        spacing: 2

        KvIconButton {
            iconName: "add"
            iconSize: 16
            tooltip: qsTr("Nova pasta aqui")
            focus: false
            onClicked: root.controller.beginCreateFolder()
        }

        KvIconButton {
            iconName: root.controller.showHidden ? "eye" : "eye-off"
            iconSize: 16
            active: root.controller.showHidden
            tooltip: root.controller.showHidden ? qsTr("Esconder pastas ocultas")
                                                : qsTr("Mostrar pastas ocultas")
            focus: false
            onClicked: root.controller.setShowHidden(!root.controller.showHidden)
        }
    }
}
