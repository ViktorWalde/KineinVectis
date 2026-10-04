import QtQuick
import KineinVectis

// Configuracoes > Interface: o que e' da IDE e nao do codigo. Por ora, a
// onda da tela de boas-vindas (o mesmo interruptor "Animação" de la').
Column {
    id: root

    property var settings: null

    spacing: Theme.spacingSmall

    Rectangle {
        width: parent.width
        height: rows.implicitHeight + 2 * Theme.spacingXSmall
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderSoft

        Column {
            id: rows

            anchors.fill: parent
            anchors.margins: Theme.spacingXSmall

            SettingsToggleRow {
                width: parent.width
                label: qsTr("Animação da tela de boas-vindas")
                hint: qsTr("A onda no degradê do fundo. Só anda com a tela à vista; ao abrir um projeto ela para.")
                checked: root.settings !== null && root.settings.welcomeAnimation
                onToggled: root.settings.settingChanged("welcomeAnimation", !root.settings.welcomeAnimation)
            }
        }
    }
}
