pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Configuracoes > Build: o perfil de rigor, com o que cada um FAZ escrito
// na propria opcao (antes eram tres palavras e um paragrafo em cima; o
// detalhe por linguagem esta' no manual, §7).
Column {
    id: root

    property var settings: null

    readonly property var options: [
        { key: "strict", label: qsTr("Estrito"), hint: qsTr("Aviso do compilador para o build; lint completo (clippy pedantic, ruff ampliado).") },
        { key: "balanced", label: qsTr("Equilibrado"), hint: qsTr("O projeto decide se aviso para o build; lint padrão.") },
        { key: "relaxed", label: qsTr("Relaxado"), hint: qsTr("Aviso nunca para o build; o lint só aponta erro grave.") }
    ]

    spacing: Theme.spacingSmall

    Row {
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Perfil de rigor")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeMedium
        }

        SettingsProjectBadge {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.settings !== null && root.settings.shownProjectKeys.indexOf("rigorProfile") >= 0
        }
    }

    Text {
        width: parent.width
        text: qsTr("O quanto Verificar e Compilar apertam o SEU projeto (C/C++ com CMake, Rust, Python). "
                   + "Vale no próximo build; a IDE reconfigura o que precisar.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
        wrapMode: Text.WordWrap
    }

    Repeater {
        model: root.options

        delegate: Rectangle {
            id: option

            required property var modelData

            readonly property bool selected: root.settings !== null
                                             && root.settings.rigorProfile === option.modelData.key

            width: root.width
            height: optionText.implicitHeight + 2 * Theme.spacingMedium
            radius: Theme.radius
            color: option.selected ? Theme.surfaceSelected : (optionArea.containsMouse ? Theme.surface2 : Theme.background1)
            border.width: 1
            border.color: option.selected ? Theme.accent : Theme.borderSoft
            Accessible.role: Accessible.RadioButton
            Accessible.name: option.modelData.label
            Accessible.checked: option.selected

            Behavior on color {
                ColorAnimation { duration: Theme.motionFast }
            }

            // O marcador de escolha unica: o circulo se enche ao escolher.
            Rectangle {
                id: radio

                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                width: 16
                height: 16
                radius: width / 2
                color: "transparent"
                border.width: 1
                border.color: option.selected ? Theme.accent : Theme.borderStrong

                Rectangle {
                    anchors.centerIn: parent
                    width: option.selected ? 8 : 0
                    height: width
                    radius: width / 2
                    color: Theme.accent

                    Behavior on width {
                        NumberAnimation { duration: Theme.motionFast }
                    }
                }
            }

            Column {
                id: optionText

                anchors.left: radio.right
                anchors.leftMargin: Theme.spacingMedium
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingMedium
                anchors.verticalCenter: parent.verticalCenter
                spacing: 2

                Text {
                    text: option.modelData.label + (option.modelData.key === "strict" ? qsTr("  (padrão)") : "")
                    color: option.selected ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: Theme.fontSizeBody
                    font.weight: option.selected ? Font.DemiBold : Font.Normal
                }

                Text {
                    width: parent.width
                    text: option.modelData.hint
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                    wrapMode: Text.WordWrap
                }
            }

            MouseArea {
                id: optionArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: if (!option.selected) root.settings.settingChanged("rigorProfile", option.modelData.key)
            }
        }
    }
}
