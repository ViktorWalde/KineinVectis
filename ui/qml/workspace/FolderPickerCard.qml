import QtQuick
import KineinVectis

// O cartao do seletor. Tres rostos:
//   abrir/escolher   o navegador AMPLO (0.3.9: locais, migalhas, lista larga)
//   criar projeto    linguagem, nome, local e previa; compacto, sem navegador
//   local do projeto o navegador amplo de novo, com "Usar esta pasta"
// O tamanho anda entre os rostos em `motionFast`, sem salto.
Rectangle {
    id: root

    property var controller
    property real maxAvailableWidth: 980
    property real maxAvailableHeight: 680
    property var recentProjects: []

    signal closeRequested()

    readonly property bool creatingProject: root.controller.creatingProject
    // Criando projeto, o navegador so' aparece em "Alterar local…".
    readonly property bool browsing: !creatingProject || root.controller.createBrowsing
    readonly property bool choosingLocation: creatingProject && root.controller.createBrowsing
    readonly property bool pickingFolder: root.controller.purpose !== "workspace"

    width: Math.min(root.maxAvailableWidth, root.browsing ? 980 : 720)
    // Criando projeto sem o navegador, o cartao encolhe ao conteudo: nada de
    // um vazio grande entre a previa e o "Criar projeto".
    height: root.browsing ? Math.min(root.maxAvailableHeight, 680)
                          : Math.min(root.maxAvailableHeight,
                                     topBlock.implicitHeight + footer.implicitHeight
                                     + 3 * Theme.spacingLarge)
    radius: Theme.radiusDialog
    color: Theme.surface1
    border.color: Theme.borderSoft
    border.width: 1

    Behavior on width {
        NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
    }
    Behavior on height {
        NumberAnimation { duration: Theme.motionFast; easing.type: Theme.easingStandard }
    }

    function focusPath() {
        browser.focusList();
    }

    function focusCreateName() {
        createPanel.focusName();
    }

    MouseArea {
        anchors.fill: parent
    }

    Column {
        id: topBlock

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        FolderPickerHeader {
            title: root.choosingLocation ? qsTr("Local do projeto")
                   : root.creatingProject ? qsTr("Criar projeto")
                   : (root.pickingFolder ? qsTr("Escolher pasta") : qsTr("Abrir projeto"))
            subtitle: root.choosingLocation ? qsTr("Escolha a pasta onde o projeto vai nascer.")
                      : root.creatingProject ? qsTr("Escolha a linguagem, dê um nome e pronto.")
                      : (root.pickingFolder ? ""
                         : qsTr("Escolha a pasta do projeto que já existe. Pastas de projeto vêm marcadas."))
            onCloseRequested: root.closeRequested()
        }

        FolderPickerCreatePanel {
            id: createPanel

            visible: root.creatingProject && !root.choosingLocation
            controller: root.controller
        }
    }

    FolderPickerBrowser {
        id: browser

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
        recentProjects: root.creatingProject || root.pickingFolder ? [] : root.recentProjects
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
            pickingFolder: root.pickingFolder
            creatingProject: root.creatingProject
            choosingLocation: root.choosingLocation
            canCreate: root.controller.canCreateProject
            selectedPath: root.controller.selectedPath !== "" ? root.controller.selectedPath
                                                              : root.controller.currentPath
            onCancelRequested: {
                if (root.choosingLocation) root.controller.createBrowsing = false;
                else root.closeRequested();
            }
            onOpenRequested: root.choosingLocation ? root.controller.useAsLocation()
                                                   : root.controller.openSelected()
            onCreateRequested: root.controller.submitCreate()
            onSwitchToCreateRequested: {
                root.controller.chooseTemplate("");
                root.controller.beginCreateProject();
            }
        }
    }
}
