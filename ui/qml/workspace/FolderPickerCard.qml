import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 560

    signal closeRequested()

    width: Math.min(root.maxAvailableWidth, 720)
    // Criando projeto sem o navegador aberto, o cartao encolhe ao conteudo:
    // nada de um vazio grande entre a previa e o "Criar projeto".
    height: root.browsing ? Math.min(root.maxAvailableHeight, 560)
                          : Math.min(root.maxAvailableHeight,
                                     topBlock.implicitHeight + footer.implicitHeight
                                     + 3 * Theme.spacingLarge)
    radius: Theme.radiusLarge
    color: Theme.surface1
    border.color: Theme.borderSoft
    border.width: 1

    function focusPath() {
        pathBar.focusPath();
    }

    function focusCreateName() {
        if (root.creatingProject) createPanel.focusName();
        else folderPanel.focusName();
    }

    MouseArea {
        anchors.fill: parent
    }

    readonly property bool creatingProject: root.controller.creatingProject
    // Criando projeto, o navegador de pastas so' aparece se pedido.
    readonly property bool browsing: !creatingProject || root.controller.createBrowsing

    // O bloco de cima, o pe' embaixo e a lista de pastas no meio.
    Column {
        id: topBlock

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        FolderPickerHeader {
            title: root.creatingProject ? qsTr("Criar projeto")
                   : (root.controller.purpose !== "workspace" ? qsTr("Escolher pasta")
                                                              : qsTr("Abrir projeto"))
            subtitle: root.creatingProject
                      ? qsTr("Escolha a linguagem, dê um nome e pronto.")
                      : (root.controller.purpose !== "workspace" ? ""
                         : qsTr("Escolha a pasta do projeto que já existe."))
            onCloseRequested: root.closeRequested()
        }

        FolderPickerCreatePanel {
            id: createPanel

            visible: root.creatingProject
            controller: root.controller
        }

        FolderPickerPathBar {
            id: pathBar

            visible: root.browsing
            controller: root.controller
        }

        FolderPickerCreatePanel {
            id: folderPanel

            visible: root.controller.creatingFolder
            controller: root.controller
        }
    }

    FolderPickerDirectoryList {
        anchors.top: topBlock.bottom
        anchors.topMargin: Theme.spacingMedium
        anchors.bottom: footer.top
        anchors.bottomMargin: Theme.spacingMedium
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingLarge
        anchors.rightMargin: Theme.spacingLarge
        visible: root.browsing
        controller: root.controller
    }

    Column {
        id: footer

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingSmall

        Text {
            visible: root.controller.errorText !== ""
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.controller.errorText
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeSmall
        }

        FolderPickerActionRow {
            pickingFolder: root.controller.purpose !== "workspace"
            creatingProject: root.creatingProject
            canCreate: root.controller.canCreateProject
            selectedPath: root.controller.selectedPath !== "" ? root.controller.selectedPath
                                                              : root.controller.currentPath
            onCancelRequested: root.closeRequested()
            onOpenRequested: root.controller.openSelected()
            onCreateRequested: root.controller.submitCreate()
            onSwitchToCreateRequested: {
                root.controller.chooseTemplate("");
                root.controller.beginCreateProject();
            }
        }
    }
}
