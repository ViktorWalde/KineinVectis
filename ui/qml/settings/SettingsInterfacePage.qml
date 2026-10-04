import QtQuick
import KineinVectis

// Configuracoes > Interface: o que e' da IDE e nao do codigo. A onda da tela
// de boas-vindas (o mesmo interruptor "Animação" de la') e o Grafana dentro
// da IDE (0.154.0, desligado por padrao).
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

            SettingsToggleRow {
                width: parent.width
                label: qsTr("Grafana dentro da IDE")
                hint: qsTr("Mostra os dashboards do Grafana local (localhost) na aba Web da janela do Grafana. Desligado, o navegador embutido nem é carregado; ligado, ele só ocupa memória com a aba aberta.")
                checked: root.settings !== null && root.settings.grafanaWebView
                onToggled: root.settings.settingChanged("grafanaWebView", !root.settings.grafanaWebView)
            }
        }
    }
}
