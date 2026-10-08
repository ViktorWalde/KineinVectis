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

    property alias currentPath: folderPickerController.currentPath
    property alias homePath: folderPickerController.homePath
    // Os recentes da tela inicial, na coluna de locais do "Abrir projeto".
    property var recentProjects: []

    signal browseRequested(string path)
    signal openRequested(string path)
    signal folderPicked(string purpose, string path)
    signal createFolderRequested(string parent, string name)
    signal createProjectRequested(string parent, string name, string templateId)

    function open(startPath) {
        picker.visible = true;
        picker.forceActiveFocus();
        folderPickerController.open(startPath);
    }

    // Escolher uma pasta para OUTRO fim (o SDK do kit): o mesmo navegador,
    // o botao vira "Escolher" e o caminho volta por folderPicked.
    function openFor(purpose, startPath) {
        picker.visible = true;
        picker.forceActiveFocus();
        folderPickerController.openFor(purpose, startPath);
    }

    // `templateId` vazio: a pessoa escolhe a linguagem e o ecossistema.
    function openCreateProject(startPath, templateId) {
        picker.visible = true;
        picker.forceActiveFocus();
        folderPickerController.open(startPath);
        folderPickerController.chooseTemplate(templateId);
        folderPickerController.beginCreateProject();
    }

    // Abrir ou criar: a mesma porta, com a intencao de quem pediu. Os dois
    // comecam no Inicio (/home/<usuario>): um projeto novo nasce la', e o
    // "Abrir" mostra os projetos de la' — com o aberto ja' marcado (2026-10-04,
    // pedido do autor: abrir DENTRO do projeto mostrava "pastas dentro de
    // pastas" que nao sao projeto).
    function openWith(intent, startPath) {
        if (intent === "createProject") {
            openCreateProject(folderPickerController.homePath, "");
        } else if (folderPickerController.homePath !== "") {
            picker.visible = true;
            picker.forceActiveFocus();
            folderPickerController.openFor("workspace", folderPickerController.homePath, startPath);
        } else {
            open(startPath);
        }
    }

    function close() {
        picker.visible = false;
        folderPickerController.close();
    }

    // Esc fecha, como nos outros dialogos (pente fino 0.3.9: o seletor so'
    // fechava no botao). Os campos de dentro (trilha editavel, pasta nova)
    // tratam o Esc deles primeiro — so' o Esc que ninguem usou chega aqui.
    Keys.onEscapePressed: function(event) {
        picker.close();
        event.accepted = true;
    }

    function setListing(path, parent, entries, places) {
        folderPickerController.setListing(path, parent, entries, places);
    }

    function showError(message) {
        picker.visible = true;
        folderPickerController.showError(message);
    }

    function selectAfterRefresh(path) {
        folderPickerController.selectAfterRefresh(path);
    }

    FolderPickerController {
        id: folderPickerController

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

        KvBackdrop {
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
        controller: folderPickerController
        maxAvailableWidth: parent.width - 80
        maxAvailableHeight: parent.height - 80
        recentProjects: picker.recentProjects
        onCloseRequested: picker.close()
    }
}
