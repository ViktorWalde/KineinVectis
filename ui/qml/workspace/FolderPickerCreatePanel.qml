pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller

    width: parent.width
    height: visible ? (root.controller.createMode === "project" ? 194 : 42) : 0
    visible: root.controller.createMode !== ""
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderSoft
    border.width: 1

    function focusName() {
        createNameField.forceActiveFocus();
    }

    function previewText() {
        const name = root.controller.createName.trim() !== ""
                     ? root.controller.createName.trim() : qsTr("meu-projeto");
        if (root.controller.createTemplate === "cppCmake") {
            return name + "/\n"
                 + "  CMakeLists.txt  · C++23 target-based\n"
                 + "  CMakePresets.json  · Debug + Release / Ninja\n"
                 + "  src/main.cpp\n"
                 + "  include/  tests/\n"
                 + "  .gitignore  README.md\n"
                 + qsTr("Geração interna: nenhum comando externo");
        }
        if (root.controller.createTemplate === "rustCargo") {
            return name + "/\n"
                 + "  Cargo.toml\n"
                 + "  src/main.rs\n\n"
                 + qsTr("Comando: cargo new --bin --vcs none %1").arg(name);
        }
        if (root.controller.createTemplate === "python") {
            return name + "/\n"
                 + "  pyproject.toml  · PEP 621, pytest em [dev], ruff\n"
                 + "  main.py  · o ponto de entrada do Executar\n"
                 + "  " + name.replace(/-/g, "_") + "/__init__.py  tests/test_main.py\n"
                 + "  .gitignore  README.md\n"
                 + qsTr("Geração interna: nenhum comando externo; o .venv é um clique depois");
        }
        return name + "/  " + qsTr("(diretório vazio)");
    }

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
                text: root.controller.createMode === "project"
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
                    text: root.controller.createName
                    color: Theme.textPrimary
                    selectedTextColor: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    font.pixelSize: 11
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.controller.createName = text
                    onAccepted: root.controller.submitCreate()
                }
            }

            FolderPickerButton {
                id: createButton

                text: qsTr("Criar")
                height: parent.height
                primary: true
                onClicked: root.controller.submitCreate()
            }

            KvIconButton {
                id: cancelCreateButton

                compact: true
                iconName: "close"
                tooltip: qsTr("Cancelar criação")
                onClicked: root.controller.cancelCreate()
            }
        }

        Row {
            width: parent.width
            height: 24
            visible: root.controller.createMode === "project"
            spacing: Theme.spacingSmall

            Repeater {
                model: [
                    { key: "cppCmake", label: "C++ CMake" },
                    { key: "rustCargo", label: "Rust Cargo" },
                    { key: "python", label: "Python" },
                    { key: "empty", label: qsTr("Vazio") }
                ]

                delegate: Rectangle {
                    id: templateChip

                    required property var modelData

                    width: templateLabel.width + 2 * Theme.spacingSmall
                    height: parent.height
                    radius: Theme.radius
                    color: root.controller.createTemplate === modelData.key
                           ? Theme.surfaceSelected
                           : (templateArea.containsMouse
                              ? Theme.surface2 : "transparent")
                    border.color: Theme.borderSoft
                    border.width: 1

                    Text {
                        id: templateLabel

                        anchors.centerIn: parent
                        text: templateChip.modelData.label
                        color: root.controller.createTemplate === templateChip.modelData.key
                               ? Theme.accent : Theme.textSecondary
                        font.pixelSize: 10
                        font.bold: true
                    }

                    MouseArea {
                        id: templateArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.controller.createTemplate = templateChip.modelData.key
                    }
                }
            }
        }

        Rectangle {
            width: parent.width
            height: 106
            visible: root.controller.createMode === "project"
            radius: Theme.radius
            color: Theme.background0
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                text: qsTr("Preview — arquivos e comandos\n") + root.previewText()
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: 10
                wrapMode: Text.WrapAnywhere
            }
        }
    }
}
