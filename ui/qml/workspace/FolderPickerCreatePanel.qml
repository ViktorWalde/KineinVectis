import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller

    width: parent.width
    height: visible ? (root.controller.createMode === "project" ? 74 : 42) : 0
    visible: root.controller.createMode !== ""
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderSoft
    border.width: 1

    function focusName() {
        createNameField.forceActiveFocus();
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

            FolderPickerButton {
                id: cancelCreateButton

                width: 26
                height: parent.height
                text: "x"
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
                    { key: "empty", label: qsTr("Vazio") }
                ]

                delegate: Rectangle {
                    id: templateChip

                    required property var modelData

                    width: templateLabel.width + 2 * Theme.spacingSmall
                    height: parent.height
                    radius: Theme.radius
                    color: root.controller.createTemplate === modelData.key
                           ? Theme.accentDim
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
    }
}
