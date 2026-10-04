pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A JANELA DO GRAFANA (2026-10-04, roadmaps/59 §6.1): a observabilidade
// acoplada ao layout como o Banco, os Containers e o Remoto — abre no slot do
// lado do icone e lembra a largura por projeto. Antes era um pop-up no meio da
// tela, com um "configurar…" que nao mudava nada.
//
//   Grafana  respondeu agora · 11.2.0          ⚙  ↗  ×
//   Painel  Web
//   ━━━━━━
//   a aba aberta: o painel (endereco, token, achados) ou o Grafana local
//   dentro da IDE (GrafanaWebPage, opcional)
//
// Com a aba Web a vista a janela pede largura (um dashboard nao cabe em
// 300 px); fechar a janela destroi a view.
Rectangle {
    id: root

    property var controller: null
    // O SettingsController: a opcao `grafanaWebView` e o "Ligar".
    property var settings: null

    signal closeRequested()
    signal widenRequested(real width)
    signal restoreWidthRequested(real width)

    radius: Theme.radiusLarge
    color: Theme.background1

    readonly property real minimumWidth: 300
    readonly property bool webEnabled: root.settings !== null && root.settings.grafanaWebView
    readonly property string baseUrl: root.controller ? root.controller.profile.url : ""
    property string tab: "painel"
    // A largura de antes da aba Web. Sair dela, ou fechar a janela, devolve o
    // espaco ao codigo: antes a janela ficava com 830 px tambem no Painel
    // (achado na tela, 2026-10-04).
    property real widthBeforeWeb: 0

    function widenForWeb() {
        if (!root.webEnabled || root.widthBeforeWeb > 0) return;
        root.widthBeforeWeb = root.width;
        root.widenRequested(900);
    }

    function giveBackWidth() {
        if (root.widthBeforeWeb <= 0) return;
        root.restoreWidthRequested(root.widthBeforeWeb);
        root.widthBeforeWeb = 0;
    }

    function showTab(id) {
        root.tab = id;
        if (id === "web") root.widenForWeb();
        else root.giveBackWidth();
    }

    // Um dashboard dos achados abre AQUI quando a aba Web esta' ligada; senao,
    // no navegador do sistema, como antes.
    function openDashboard(path) {
        const url = root.controller.dashboardUrl(path);
        if (root.webEnabled && web.isLocal(url)) {
            root.showTab("web");
            web.load(url);
        } else {
            Qt.openUrlExternally(url);
        }
    }

    onVisibleChanged: {
        if (root.visible) {
            if (root.tab === "web") root.widenForWeb();
        } else {
            web.release();
            root.giveBackWidth();
        }
    }

    // ---- cabecalho: titulo, estado, ajustes, abrir fora, fechar -------------

    Item {
        id: headerRow

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        height: 24

        Text {
            id: title

            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Grafana")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        Text {
            anchors.left: title.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: headerActions.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: root.controller ? root.controller.statusPhrase : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }

        Row {
            id: headerActions

            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            KvIconButton {
                compact: true
                iconName: "settings"
                active: panel.setupShown
                tooltip: panel.setupShown ? qsTr("Esconder o endereço e o token")
                                          : qsTr("Mostrar o endereço e o token")
                onClicked: {
                    root.showTab("painel");
                    panel.toggleSetup();
                }
            }

            KvIconButton {
                compact: true
                iconName: "external"
                enabled: root.baseUrl !== ""
                tooltip: qsTr("Abrir o Grafana no navegador")
                onClicked: Qt.openUrlExternally(root.baseUrl)
            }

            KvIconButton {
                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar a janela do Grafana")
                onClicked: root.closeRequested()
            }
        }
    }

    RemoteSections {
        id: tabs

        anchors.top: headerRow.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        current: root.tab
        sections: [
            { "id": "painel", "label": qsTr("Painel") },
            { "id": "web", "label": qsTr("Web") }
        ]
        onSelected: id => root.showTab(id)
    }

    GrafanaPanel {
        id: panel

        anchors.top: tabs.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        visible: root.tab === "painel"
        controller: root.controller
        onDashboardActivated: path => root.openDashboard(path)
    }

    GrafanaWebPage {
        id: web

        anchors.top: tabs.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        visible: root.visible && root.tab === "web"
        webEnabled: root.webEnabled
        baseUrl: root.baseUrl
        onEnableRequested: {
            root.settings.setGrafanaWebView(true);
            Qt.callLater(root.widenForWeb);
        }
    }
}
