// IDE-owned workspace picker. Directory data comes from kinein-core through
// workspace.browse; this component never touches the filesystem directly.
import QtQuick
import KineinVectis

Item {
    id: picker

    visible: false
    // Modal: acima do cabecalho (z 100, declarado depois no Main — no empate
    // ele cortava o topo do cartao alto da 0.3.9), abaixo das dicas.
    z: 500

    property alias currentPath: controller.currentPath
    property alias homePath: controller.homePath
    // Os recentes da tela inicial, na coluna de locais do "Abrir projeto".
    property var recentProjects: []

    signal browseRequested(string path)
    signal openRequested(string path)
    signal folderPicked(string purpose, string path)
    signal createFolderRequested(string parent, string name)
    signal createProjectRequested(string parent, string name, string templateId)

    function open(startPath) {
        picker.visible = true;
        controller.open(startPath);
    }

    // Escolher uma pasta para OUTRO fim (o SDK do kit): o mesmo navegador,
    // o botao vira "Escolher" e o caminho volta por folderPicked.
    function openFor(purpose, startPath) {
        picker.visible = true;
        controller.openFor(purpose, startPath);
    }

    // `templateId` vazio: a pessoa escolhe a linguagem e o ecossistema.
    function openCreateProject(startPath, templateId) {
        picker.visible = true;
        controller.open(startPath);
        controller.chooseTemplate(templateId);
        controller.beginCreateProject();
    }

    // Abrir ou criar: a mesma porta, com a intencao de quem pediu. Um projeto
    // novo nasce na home, nao dentro do workspace aberto.
    function openWith(intent, startPath) {
        if (intent === "createProject") {
            openCreateProject(controller.homePath, "");
        } else {
            open(startPath);
        }
    }

    function close() {
        picker.visible = false;
        controller.close();
    }

    function setListing(path, parent, entries, places) {
        controller.setListing(path, parent, entries, places);
    }

    function showError(message) {
        picker.visible = true;
        controller.showError(message);
    }

    function selectAfterRefresh(path) {
        controller.selectAfterRefresh(path);
    }

    FolderPickerController {
        id: controller

        onBrowseRequested: function(path) {
            picker.browseRequested(path);
        }
        onOpenRequested: function(path) {
            picker.openRequested(path);
        }
        onFolderPicked: function(purpose, path) {
            picker.close();
            picker.folderPicked(purpose, path);
        }
        onCreateFolderRequested: function(parent, name) {
            picker.createFolderRequested(parent, name);
        }
        onCreateProjectRequested: function(parent, name, templateId) {
            picker.createProjectRequested(parent, name, templateId);
        }
        onPathFocusRequested: card.focusPath()
        onCreateFocusRequested: card.focusCreateName()
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.scrim

        MouseArea {
            anchors.fill: parent
            onClicked: picker.close()
        }
    }

    // Entrada: aparece e desliza 8 px em `motionFast` (0.3.9, fluidez).
    property real entrance: 0
    onVisibleChanged: if (visible) entranceAnimation.restart()

    NumberAnimation {
        id: entranceAnimation

        target: picker
        property: "entrance"
        from: 0
        to: 1
        duration: Theme.motionFast
        easing.type: Theme.easingStandard
    }

    FolderPickerCard {
        id: card

        opacity: picker.entrance
        anchors.centerIn: parent
        anchors.verticalCenterOffset: 8 * (1 - picker.entrance)
        controller: controller
        maxAvailableWidth: parent.width - 80
        maxAvailableHeight: parent.height - 80
        recentProjects: picker.recentProjects
        onCloseRequested: picker.close()
    }
}
