import QtQuick
import KineinVectis

// Configuracoes > Editor: a fonte (com a previa ao vivo), salvar, formatar e
// fechar pares. `settings` e' o SettingsDialog: os valores e o sinal de troca.
Column {
    id: root

    property var settings: null

    readonly property int minFontSize: 8
    readonly property int maxFontSize: 40

    function fromProject(key) {
        return root.settings !== null && root.settings.shownProjectKeys.indexOf(key) >= 0;
    }

    function stepFont(delta) {
        const next = Math.max(root.minFontSize, Math.min(root.maxFontSize, root.settings.editorFontSize + delta));
        if (next !== root.settings.editorFontSize) root.settings.settingChanged("editorFontSize", next);
    }

    spacing: Theme.spacingSmall

    // A fonte: o passo e, embaixo, como o codigo fica.
    Rectangle {
        width: parent.width
        height: fontColumn.implicitHeight + 2 * Theme.spacingMedium
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderSoft

        Column {
            id: fontColumn

            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Item {
                width: parent.width
                height: 30

                Column {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2

                    Row {
                        spacing: Theme.spacingSmall

                        Text {
                            text: qsTr("Tamanho da fonte do editor")
                            color: Theme.textPrimary
                            font.pixelSize: Theme.fontSizeMedium
                        }

                        SettingsProjectBadge {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: root.fromProject("editorFontSize")
                        }
                    }

                    Text {
                        text: qsTr("Vale na hora, em todos os arquivos abertos")
                        color: Theme.textMuted
                        font.pixelSize: Theme.fontSizeCaption
                    }
                }

                Row {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.spacingXSmall

                    KvButton {
                        width: 30
                        compact: true
                        text: "−"
                        enabled: root.settings !== null && root.settings.editorFontSize > root.minFontSize
                        tooltip: qsTr("Diminuir")
                        onClicked: root.stepFont(-1)
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 34
                        horizontalAlignment: Text.AlignHCenter
                        text: root.settings ? root.settings.editorFontSize : ""
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeLarge
                    }

                    KvButton {
                        width: 30
                        compact: true
                        text: "+"
                        enabled: root.settings !== null && root.settings.editorFontSize < root.maxFontSize
                        tooltip: qsTr("Aumentar")
                        onClicked: root.stepFont(1)
                    }
                }
            }

            // A previa: o mesmo tamanho que o editor vai usar.
            Rectangle {
                width: parent.width
                height: Math.max(36, preview.implicitHeight + 2 * Theme.spacingSmall)
                radius: Theme.radiusXSmall
                color: Theme.background0
                clip: true

                Text {
                    id: preview

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingMedium
                    anchors.verticalCenter: parent.verticalCenter
                    textFormat: Text.StyledText
                    text: "<font color='" + Theme.accent + "'>fn</font> main() { println!(<font color='"
                          + Theme.successSoft + "'>\"olá, placa\"</font>); }"
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: root.settings ? root.settings.editorFontSize : 14
                }
            }
        }
    }

    Rectangle {
        width: parent.width
        height: toggles.implicitHeight + 2 * Theme.spacingXSmall
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderSoft

        Column {
            id: toggles

            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall

            SettingsToggleRow {
                width: parent.width
                label: qsTr("Salvar automaticamente")
                hint: qsTr("Após uma pausa na digitação, ao trocar de aba e ao sair do editor; "
                           + "Ctrl+S continua valendo. O rascunho de segurança fica ligado.")
                checked: root.settings !== null && root.settings.autoSave
                fromProject: root.fromProject("autoSave")
                onToggled: root.settings.settingChanged("autoSave", !root.settings.autoSave)
            }

            SettingsToggleRow {
                width: parent.width
                label: qsTr("Formatar ao salvar")
                hint: qsTr("Ctrl+S formata (rustfmt, clang-format, ruff) e então salva")
                checked: root.settings !== null && root.settings.formatOnSave
                fromProject: root.fromProject("formatOnSave")
                onToggled: root.settings.settingChanged("formatOnSave", !root.settings.formatOnSave)
            }

            SettingsToggleRow {
                width: parent.width
                label: qsTr("Fechar pares automaticamente")
                hint: qsTr("( [ { \" ' fecham sozinhos ao digitar")
                checked: root.settings !== null && root.settings.autoClosePairs
                fromProject: root.fromProject("autoClosePairs")
                onToggled: root.settings.settingChanged("autoClosePairs", !root.settings.autoClosePairs)
            }
        }
    }
}
