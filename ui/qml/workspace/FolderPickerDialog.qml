// IDE-owned workspace picker. Directory data comes from kinein-core through
// workspace.browse; this component never touches the filesystem directly.
import QtQuick
import KineinVectis

Item {
    id: picker

    visible: false
    z: 100

    property alias currentPath: controller.currentPath
    property alias homePath: controller.homePath

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

    function openCreateProject(startPath, templateId) {
        picker.visible = true;
        controller.open(startPath);
        controller.createTemplate = templateId;
        controller.beginCreateProject();
    }

    function close() {
        picker.visible = false;
        controller.close();
    }

    function setListing(path, parent, entries) {
        controller.setListing(path, parent, entries);
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
        color: "#c0000000"

        MouseArea {
            anchors.fill: parent
            onClicked: picker.close()
        }
    }

    FolderPickerCard {
        id: card

        anchors.centerIn: parent
        controller: controller
        maxAvailableWidth: parent.width - 80
        maxAvailableHeight: parent.height - 80
        onCloseRequested: picker.close()
    }
}
