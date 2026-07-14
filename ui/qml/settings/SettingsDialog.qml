pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Página de configurações (fatia M4.1), overlay simples no padrão dos
// diálogos. Componente burro: recebe os valores efetivos por property e
// emite settingChanged(key, value) por controle; o pai encaminha para o
// SettingsController (escopo global no v1). A UI não usa QtQuick.Controls,
// então toggles e stepper são próprios.
Item {
    id: root

    property bool formatOnSave: false
    property int editorFontSize: 14
    property bool autoClosePairs: true
    property string rigorProfile: "strict"
    property real maxAvailableWidth: 900
    property real maxAvailableHeight: 600

    // Opções do perfil de rigor (M4.5): chave (serde camelCase) + rótulo.
    readonly property var rigorOptions: [
        { "key": "strict", "label": qsTr("Estrito") },
        { "key": "balanced", "label": qsTr("Equilibrado") },
        { "key": "relaxed", "label": qsTr("Relaxado") }
    ]

    readonly property int minFontSize: 8
    readonly property int maxFontSize: 40

    signal dismissRequested()
    signal settingChanged(string key, var value)

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(480, root.maxAvailableWidth)
        height: Math.min(420, root.maxAvailableHeight)
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        Text {
            id: dialogTitle

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: closeChip.left
            anchors.margins: Theme.spacingMedium
            text: qsTr("Configurações")
            color: Theme.textPrimary
            font.pixelSize: 14
            font.bold: true
        }

        KvIconButton {
            id: closeChip
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            iconName: "close"
            tooltip: qsTr("Fechar")
            onClicked: root.dismissRequested()
        }

        Column {
            anchors.top: dialogTitle.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.margins: Theme.spacingMedium
            anchors.topMargin: Theme.spacingLarge
            spacing: Theme.spacingLarge

            // --- Tamanho da fonte do editor ---
            Row {
                width: parent.width
                spacing: Theme.spacingMedium

                Column {
                    width: parent.width - fontStepper.width - Theme.spacingMedium
                    spacing: 2

                    Text {
                        text: qsTr("Tamanho da fonte do editor")
                        color: Theme.textPrimary
                        font.pixelSize: 13
                    }

                    Text {
                        text: qsTr("Aplica na hora ao editor")
                        color: Theme.textMuted
                        font.pixelSize: 10
                    }
                }

                Row {
                    id: fontStepper

                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.spacingSmall

                    Rectangle {
                        width: 26
                        height: 26
                        radius: Theme.radius
                        color: fontMinusArea.containsMouse ? Theme.surface2 : Theme.surface1
                        border.color: Theme.borderSoft
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: "−"
                            color: Theme.textPrimary
                            font.pixelSize: 14
                        }

                        MouseArea {
                            id: fontMinusArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                const next = Math.max(root.minFontSize,
                                                      root.editorFontSize - 1);
                                if (next !== root.editorFontSize) {
                                    root.settingChanged("editorFontSize", next);
                                }
                            }
                        }
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 28
                        horizontalAlignment: Text.AlignHCenter
                        text: root.editorFontSize
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 14
                    }

                    Rectangle {
                        width: 26
                        height: 26
                        radius: Theme.radius
                        color: fontPlusArea.containsMouse ? Theme.surface2 : Theme.surface1
                        border.color: Theme.borderSoft
                        border.width: 1

                        Text {
                            anchors.centerIn: parent
                            text: "+"
                            color: Theme.textPrimary
                            font.pixelSize: 14
                        }

                        MouseArea {
                            id: fontPlusArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                const next = Math.min(root.maxFontSize,
                                                      root.editorFontSize + 1);
                                if (next !== root.editorFontSize) {
                                    root.settingChanged("editorFontSize", next);
                                }
                            }
                        }
                    }
                }
            }

            // --- Formatar ao salvar ---
            SettingsToggleRow {
                width: parent.width
                label: qsTr("Formatar ao salvar")
                hint: qsTr("Ctrl+S formata (rustfmt/clang-format) e então salva")
                checked: root.formatOnSave
                onToggled: root.settingChanged("formatOnSave", !root.formatOnSave)
            }

            // --- Fechar pares automaticamente ---
            SettingsToggleRow {
                width: parent.width
                label: qsTr("Fechar pares automaticamente")
                hint: qsTr("( [ { \" ' fecham sozinhos ao digitar")
                checked: root.autoClosePairs
                onToggled: root.settingChanged("autoClosePairs", !root.autoClosePairs)
            }

            // --- Perfil de rigor (M4.5) ---
            Column {
                width: parent.width
                spacing: Theme.spacingSmall

                Text {
                    text: qsTr("Perfil de rigor")
                    color: Theme.textPrimary
                    font.pixelSize: 13
                }

                Text {
                    width: parent.width
                    text: qsTr("Regula clippy e warnings do SEU projeto (não afeta o Kinein)")
                    color: Theme.textMuted
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                }

                Row {
                    id: rigorSegments

                    width: parent.width
                    spacing: Theme.spacingSmall

                    Repeater {
                        model: root.rigorOptions

                        delegate: Rectangle {
                            id: segment

                            required property var modelData

                            readonly property bool selected: root.rigorProfile === segment.modelData.key

                            width: (rigorSegments.width - 2 * Theme.spacingSmall) / 3
                            height: 30
                            radius: Theme.radius
                            color: segment.selected
                                    ? Theme.accent
                                    : (segmentArea.containsMouse ? Theme.surface2 : Theme.surface1)
                            border.color: segment.selected ? Theme.accent : Theme.borderSoft
                            border.width: 1

                            Text {
                                anchors.centerIn: parent
                                text: segment.modelData.label
                                color: segment.selected ? Theme.background0 : Theme.textPrimary
                                font.pixelSize: 11
                                font.bold: segment.selected
                            }

                            MouseArea {
                                id: segmentArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (root.rigorProfile !== segment.modelData.key) {
                                        root.settingChanged("rigorProfile", segment.modelData.key);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Text {
                width: parent.width
                text: qsTr("As mudanças valem para todos os projetos (global).")
                color: Theme.textMuted
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }
        }
    }

    Keys.onEscapePressed: root.dismissRequested()
}
