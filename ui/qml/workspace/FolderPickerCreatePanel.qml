pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Criar PROJETO: a tela inteira do "Criar projeto". (A "nova pasta" do
// navegador e' a FolderPickerNewFolderRow, desde a 0.3.9.)
//
// 0.3.8 (retorno do autor de 2026-10-02: "quando clico em Criar projeto da'
// para melhorar e deixar mais intuitivo"). O projeto deixou de ser um campo e
// quatro fichas espremidos no meio do navegador de pastas: e' uma sequencia
// que se le de cima para baixo — a LINGUAGEM em cartoes, o NOME com o caminho
// que vai nascer, o LOCAL numa linha (o navegador so' abre se pedir) e a
// PREVIA dos arquivos. O botao "Criar projeto" mora no pe' do cartao.
Item {
    id: root

    property var controller

    readonly property var catalog: root.controller.templateCatalog
    readonly property bool projectMode: root.controller.creatingProject
    readonly property var chosenEcosystem: catalog.ecosystem(root.controller.createTemplate)
    readonly property string projectName: root.controller.createName.trim()

    width: parent.width
    height: visible ? content.implicitHeight : 0
    visible: root.projectMode

    function focusName() {
        nameField.forceActiveFocus();
    }

    Column {
        id: content

        anchors.fill: parent
        spacing: Theme.spacingMedium

        // ---- 1. a linguagem, em cartoes ------------------------------------
        Text {
            text: qsTr("LINGUAGEM")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        Row {
            id: tiles

            width: parent.width
            spacing: Theme.spacingSmall

            Repeater {
                model: root.catalog.languages

                delegate: FolderPickerLanguageTile {
                    required property var modelData

                    width: (tiles.width - 3 * tiles.spacing) / 4
                    language: modelData
                    selected: root.controller.createLanguage === modelData.key
                    onChosen: {
                        root.controller.chooseLanguage(modelData.key);
                        nameField.forceActiveFocus();
                    }
                }
            }
        }

        // Mais de um ecossistema para a linguagem: a escolha aparece; um so':
        // a linha de baixo so' diz o que vai ser criado.
        Row {
            visible: root.catalog.ecosystems(root.controller.createLanguage).length > 1
            spacing: Theme.spacingSmall

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
        }

        Text {
            width: parent.width
            text: root.chosenEcosystem !== null
                  ? root.chosenEcosystem.label + "  —  " + root.chosenEcosystem.detail
                  : qsTr("Escolha a linguagem para ver o que vai ser criado.")
            color: root.chosenEcosystem !== null ? Theme.textSecondary : Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
            elide: Text.ElideRight
        }

        // ---- 2. o nome -------------------------------------------------------
        Text {
            text: qsTr("NOME")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.8
        }

        Item {
            width: parent.width
            height: 34

            Rectangle {
                width: parent.width
                height: parent.height
                radius: Theme.radius
                color: Theme.background0
                border.color: nameField.activeFocus ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: nameField

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    text: root.controller.createName
                    color: Theme.textPrimary
                    selectedTextColor: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    font.pixelSize: Theme.fontSizeLarge
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.controller.createName = text
                    onAccepted: root.controller.submitCreate()
                }

                Text {
                    anchors.fill: nameField
                    verticalAlignment: Text.AlignVCenter
                    visible: nameField.text === ""
                    text: qsTr("meu-projeto")
                    color: Theme.textDisabled
                    font: nameField.font
                }
            }
        }

        // ---- 3. o local: uma linha; o navegador so' se pedir --------------
        Row {
            width: parent.width
            height: 26
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Será criado em")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeSmall
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - x - changeLocation.width - Theme.spacingSmall
                text: root.controller.currentPath
                      + (root.controller.currentPath.endsWith("/") ? "" : "/")
                      + (root.projectName !== "" ? root.projectName : qsTr("…"))
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideMiddle
            }

            FolderPickerButton {
                id: changeLocation

                height: parent.height
                text: qsTr("Alterar local…")
                onClicked: root.controller.createBrowsing = true
            }
        }

        // ---- 4. a previa -----------------------------------------------------
        Rectangle {
            width: parent.width
            height: previewText.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.background0
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: previewText

                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: Theme.spacingSmall
                text: root.catalog.preview(root.controller.createTemplate, root.controller.createName)
                color: Theme.textSecondary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeCaption
                wrapMode: Text.WrapAnywhere
            }
        }
    }
}
