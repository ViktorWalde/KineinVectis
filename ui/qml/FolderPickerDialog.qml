
// IDE-owned workspace picker. Directory data comes from kinein-core through
// workspace.browse; this component never touches the filesystem directly.
import QtQuick
import KineinVectis

Item {
    id: picker

    visible: false
    z: 100

    property string currentPath: ""
    property string parentPath: ""
    property string selectedPath: ""
    property string homePath: "/"
    property string errorText: ""
    property bool loading: false
    property string createMode: ""
    property string createTemplate: "cppCmake"
    property string pendingSelectionPath: ""

    signal browseRequested(string path)
    signal openRequested(string path)
    signal createFolderRequested(string parent, string name)
    signal createProjectRequested(string parent, string name, string templateId)

    function open(startPath) {
        const path = startPath !== "" ? startPath : "/";
        picker.visible = true;
        picker.errorText = "";
        picker.selectedPath = path;
        picker.createMode = "";
        picker.pendingSelectionPath = "";
        pathField.text = path;
        picker.browsePath(path);
        pathField.forceActiveFocus();
    }

    function close() {
        picker.visible = false;
        picker.loading = false;
        picker.errorText = "";
        picker.createMode = "";
    }

    function browsePath(path) {
        const cleanPath = path.trim();
        if (cleanPath === "") {
            return;
        }
        picker.loading = true;
        picker.errorText = "";
        picker.browseRequested(cleanPath);
    }

    function setListing(path, parent, entries) {
        picker.loading = false;
        picker.currentPath = path;
        picker.parentPath = parent;
        picker.selectedPath = picker.pendingSelectionPath !== ""
                ? picker.pendingSelectionPath : path;
        picker.pendingSelectionPath = "";
        picker.errorText = "";
        pathField.text = path;

        entryModel.clear();
        for (let i = 0; i < entries.length; i++) {
            entryModel.append({
                name: entries[i].name,
                path: entries[i].path
            });
        }
    }

    function showError(message) {
        picker.loading = false;
        picker.errorText = message;
        picker.visible = true;
    }

    function openSelected() {
        const path = picker.selectedPath !== "" ? picker.selectedPath : picker.currentPath;
        if (path !== "") {
            picker.openRequested(path);
        }
    }

    function beginCreateFolder() {
        picker.createMode = "folder";
        picker.errorText = "";
        createNameField.text = "";
        createNameField.forceActiveFocus();
    }

    function beginCreateProject() {
        picker.createMode = "project";
        picker.errorText = "";
        createNameField.text = "";
        createNameField.forceActiveFocus();
    }

    function cancelCreate() {
        picker.createMode = "";
        picker.errorText = "";
    }

    function submitCreate() {
        const name = createNameField.text.trim();
        if (name === "") {
            picker.errorText = qsTr("Informe um nome.");
            return;
        }
        if (picker.createMode === "folder") {
            picker.createFolderRequested(picker.currentPath, name);
        } else if (picker.createMode === "project") {
            picker.createProjectRequested(picker.currentPath, name, picker.createTemplate);
        }
    }

    function selectAfterRefresh(path) {
        picker.pendingSelectionPath = path;
    }

    ListModel {
        id: entryModel
    }

    Rectangle {
        anchors.fill: parent
        color: "#c0000000"

        MouseArea {
            anchors.fill: parent
            onClicked: picker.close()
        }
    }

    Rectangle {
        id: dialogCard

        anchors.centerIn: parent
        width: Math.min(parent.width - 80, 720)
        height: Math.min(parent.height - 80, 560)
        radius: Theme.radiusLarge
        color: Theme.surface1
        border.color: Theme.borderSoft
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Column {
            id: dialogColumn

            anchors.fill: parent
            anchors.margins: Theme.spacingLarge
            spacing: Theme.spacingMedium

            Row {
                id: titleRow

                width: parent.width
                height: 28
                spacing: Theme.spacingMedium

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Abrir ou criar projeto")
                    color: Theme.textPrimary
                    font.pixelSize: 16
                    font.bold: true
                }

                Item {
                    width: parent.width - x - closeButton.width
                    height: 1
                }

                Rectangle {
                    id: closeButton

                    anchors.verticalCenter: parent.verticalCenter
                    width: 26
                    height: 26
                    radius: Theme.radius
                    color: closeButtonArea.containsMouse ? Theme.surface2 : "transparent"
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        anchors.centerIn: parent
                        text: "x"
                        color: Theme.textSecondary
                        font.pixelSize: 13
                    }

                    MouseArea {
                        id: closeButtonArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.close()
                    }
                }
            }

            Rectangle {
                id: pathBox

                width: parent.width
                height: 34
                radius: Theme.radius
                color: Theme.background0
                border.color: pathField.activeFocus ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: pathField

                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    selectedTextColor: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    onAccepted: picker.browsePath(text)
                }
            }

            Row {
                id: navigationRow

                width: parent.width
                height: 30
                spacing: Theme.spacingSmall

                Rectangle {
                    width: homeText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: homeArea.containsMouse ? Theme.surface2 : Theme.background2
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: homeText

                        anchors.centerIn: parent
                        text: qsTr("Inicio")
                        color: Theme.textSecondary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: homeArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.browsePath(picker.homePath)
                    }
                }

                Rectangle {
                    width: upText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    enabled: picker.parentPath !== ""
                    opacity: enabled ? 1.0 : 0.45
                    color: upArea.containsMouse ? Theme.surface2 : Theme.background2
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: upText

                        anchors.centerIn: parent
                        text: qsTr("Subir")
                        color: Theme.textSecondary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: upArea

                        anchors.fill: parent
                        enabled: picker.parentPath !== ""
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.browsePath(picker.parentPath)
                    }
                }

                Rectangle {
                    width: goText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: goArea.pressed ? Theme.accentDim : Theme.accent

                    Text {
                        id: goText

                        anchors.centerIn: parent
                        text: qsTr("Ir")
                        color: Theme.background0
                        font.pixelSize: 12
                        font.bold: true
                    }

                    MouseArea {
                        id: goArea

                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.browsePath(pathField.text)
                    }
                }

                Rectangle {
                    width: newFolderText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: newFolderArea.containsMouse ? Theme.surface2 : Theme.background2
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: newFolderText

                        anchors.centerIn: parent
                        text: qsTr("+ pasta")
                        color: Theme.textSecondary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: newFolderArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.beginCreateFolder()
                    }
                }

                Rectangle {
                    width: newProjectText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: newProjectArea.containsMouse ? Theme.surface2 : Theme.background2
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: newProjectText

                        anchors.centerIn: parent
                        text: qsTr("+ projeto")
                        color: Theme.textSecondary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: newProjectArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.beginCreateProject()
                    }
                }
            }

            Rectangle {
                id: createPanel

                width: parent.width
                height: visible ? (picker.createMode === "project" ? 74 : 42) : 0
                visible: picker.createMode !== ""
                radius: Theme.radius
                color: Theme.background2
                border.color: Theme.borderSoft
                border.width: 1

                Column {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    Row {
                        width: parent.width
                        height: 26
                        spacing: Theme.spacingSmall

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: picker.createMode === "project"
                                  ? qsTr("Projeto") : qsTr("Pasta")
                            color: Theme.textSecondary
                            font.pixelSize: 11
                            font.bold: true
                        }

                        Rectangle {
                            width: parent.width - x - createButton.width
                                   - cancelCreateButton.width - 2 * Theme.spacingSmall
                            height: parent.height
                            radius: Theme.radius
                            color: Theme.background0
                            border.color: createNameField.activeFocus
                                          ? Theme.accent : Theme.borderSoft
                            border.width: 1

                            TextInput {
                                id: createNameField

                                anchors.fill: parent
                                anchors.margins: Theme.spacingSmall
                                verticalAlignment: TextInput.AlignVCenter
                                color: Theme.textPrimary
                                selectedTextColor: Theme.textPrimary
                                selectionColor: Theme.accentDim
                                font.pixelSize: 11
                                clip: true
                                selectByMouse: true
                                onAccepted: picker.submitCreate()
                            }
                        }

                        Rectangle {
                            id: createButton

                            width: createButtonText.width + 2 * Theme.spacingMedium
                            height: parent.height
                            radius: Theme.radius
                            color: createButtonArea.pressed ? Theme.accentDim : Theme.accent

                            Text {
                                id: createButtonText

                                anchors.centerIn: parent
                                text: qsTr("Criar")
                                color: Theme.background0
                                font.pixelSize: 11
                                font.bold: true
                            }

                            MouseArea {
                                id: createButtonArea

                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: picker.submitCreate()
                            }
                        }

                        Rectangle {
                            id: cancelCreateButton

                            width: 26
                            height: parent.height
                            radius: Theme.radius
                            color: cancelCreateArea.containsMouse
                                   ? Theme.surface2 : "transparent"
                            border.color: Theme.borderSoft
                            border.width: 1

                            Text {
                                anchors.centerIn: parent
                                text: "x"
                                color: Theme.textSecondary
                                font.pixelSize: 12
                            }

                            MouseArea {
                                id: cancelCreateArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: picker.cancelCreate()
                            }
                        }
                    }

                    Row {
                        width: parent.width
                        height: 24
                        visible: picker.createMode === "project"
                        spacing: Theme.spacingSmall

                        Repeater {
                            model: [
                                { key: "cppCmake", label: "C++ CMake" },
                                { key: "rustCargo", label: "Rust Cargo" },
                                { key: "empty", label: qsTr("Vazio") }
                            ]

                            delegate: Rectangle {
                                required property var modelData

                                width: templateLabel.width + 2 * Theme.spacingSmall
                                height: parent.height
                                radius: Theme.radius
                                color: picker.createTemplate === modelData.key
                                       ? Theme.accentDim
                                       : (templateArea.containsMouse
                                          ? Theme.surface2 : "transparent")
                                border.color: Theme.borderSoft
                                border.width: 1

                                Text {
                                    id: templateLabel

                                    anchors.centerIn: parent
                                    text: parent.modelData.label
                                    color: picker.createTemplate === parent.modelData.key
                                           ? Theme.accent : Theme.textSecondary
                                    font.pixelSize: 10
                                    font.bold: true
                                }

                                MouseArea {
                                    id: templateArea

                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: picker.createTemplate = parent.modelData.key
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                id: listFrame

                width: parent.width
                height: Math.max(150, parent.height - titleRow.height - pathBox.height
                                 - navigationRow.height - createPanel.height
                                 - actionRow.height
                                 - errorLabel.height - 6 * Theme.spacingMedium)
                radius: Theme.radius
                color: Theme.background0
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    anchors.centerIn: parent
                    visible: picker.loading
                    text: qsTr("Carregando...")
                    color: Theme.textMuted
                    font.pixelSize: 12
                }

                Text {
                    anchors.centerIn: parent
                    visible: !picker.loading && entryModel.count === 0
                    text: qsTr("Nenhum subdiretorio")
                    color: Theme.textMuted
                    font.pixelSize: 12
                }

                ListView {
                    id: folderList

                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    visible: !picker.loading && entryModel.count > 0
                    clip: true
                    model: entryModel

                    delegate: Rectangle {
                        id: folderRow

                        required property string name
                        required property string path

                        width: folderList.width
                        height: 28
                        radius: Theme.radius
                        color: picker.selectedPath === folderRow.path
                               ? Theme.accentDim
                               : (folderMouse.containsMouse ? Theme.surface2 : "transparent")

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.leftMargin: Theme.spacingSmall
                            anchors.rightMargin: Theme.spacingSmall
                            spacing: Theme.spacingSmall

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 16
                                text: "▸"
                                color: Theme.accent
                                font.pixelSize: 12
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 20
                                text: folderRow.name
                                color: picker.selectedPath === folderRow.path
                                       ? Theme.textPrimary : Theme.textSecondary
                                font.pixelSize: 12
                                elide: Text.ElideRight
                            }
                        }

                        MouseArea {
                            id: folderMouse

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: picker.selectedPath = folderRow.path
                            onDoubleClicked: picker.browsePath(folderRow.path)
                        }
                    }
                }
            }

            Text {
                id: errorLabel

                visible: picker.errorText !== ""
                width: parent.width
                height: visible ? implicitHeight : 0
                wrapMode: Text.WordWrap
                text: picker.errorText
                color: Theme.errorSoft
                font.pixelSize: 11
            }

            Row {
                id: actionRow

                width: parent.width
                height: 32
                spacing: Theme.spacingSmall

                Item {
                    width: parent.width - cancelButton.width - openButton.width
                           - Theme.spacingSmall
                    height: 1
                }

                Rectangle {
                    id: cancelButton

                    width: cancelText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: cancelArea.containsMouse ? Theme.surface2 : "transparent"
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: cancelText

                        anchors.centerIn: parent
                        text: qsTr("Cancelar")
                        color: Theme.textSecondary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: cancelArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.close()
                    }
                }

                Rectangle {
                    id: openButton

                    width: openText.width + 2 * Theme.spacingMedium
                    height: parent.height
                    radius: Theme.radius
                    color: openArea.pressed ? Theme.accentDim : Theme.accent

                    Text {
                        id: openText

                        anchors.centerIn: parent
                        text: qsTr("Abrir")
                        color: Theme.background0
                        font.pixelSize: 12
                        font.bold: true
                    }

                    MouseArea {
                        id: openArea

                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.openSelected()
                    }
                }
            }
        }
    }
}
