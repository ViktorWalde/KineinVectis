pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller

    width: parent.width
    height: visible ? content.implicitHeight + 2 * Theme.spacingSmall : 0
    visible: root.controller.createMode !== ""
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderSoft
    border.width: 1

    function focusName() {
        createNameField.forceActiveFocus();
    }

    readonly property var catalog: root.controller.templateCatalog

    Column {
        id: content

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
                font.pixelSize: Theme.fontSizeSmall
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
                    font.pixelSize: Theme.fontSizeSmall
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

        // 1º a LINGUAGEM, 2º o ECOSSISTEMA dela (53 §13.0 item 6); os dois
        // vem do catalogo, e o painel so' desenha.
        Row {
            width: parent.width
            height: 22
            visible: root.controller.createMode === "project"
            spacing: Theme.spacingSmall

            Text {
                width: 78
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Linguagem")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
            }

            Repeater {
                model: root.catalog.languages

                delegate: KvToggleChip {
                    required property var modelData

                    labelText: modelData.label
                    active: root.controller.createLanguage === modelData.key
                    outlined: true
                    codeFont: false
                    onToggled: root.controller.chooseLanguage(modelData.key)
                }
            }
        }

        Row {
            width: parent.width
            height: 22
            visible: root.controller.createMode === "project"
            spacing: Theme.spacingSmall

            Text {
                width: 78
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Ecossistema")
                color: Theme.textSecondary
                font.pixelSize: Theme.fontSizeSmall
            }

            Repeater {
                model: root.catalog.ecosystems(root.controller.createLanguage)

                delegate: KvToggleChip {
                    required property var modelData

                    labelText: modelData.label
                    active: root.controller.createTemplate === modelData.template
                    outlined: true
                    codeFont: false
                    onToggled: root.controller.createTemplate = modelData.template
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - x
                text: root.controller.createLanguage === ""
                      ? qsTr("escolha a linguagem")
                      : (root.catalog.ecosystem(root.controller.createTemplate) !== null
                         ? root.catalog.ecosystem(root.controller.createTemplate).detail : "")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
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
                text: qsTr("Preview — arquivos e comandos\n")
                      + root.catalog.preview(root.controller.createTemplate,
                                             root.controller.createName)
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WrapAnywhere
            }
        }
    }
}
