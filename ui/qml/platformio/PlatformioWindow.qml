pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A JANELA DO PLATFORMIO (0.191.0, roadmaps/52 §14.45): o PIO Home, a
// interface do proprio PlatformIO (projeto novo, placas, plataformas,
// bibliotecas), no slot do lado do icone, como o Grafana.
//
//   PlatformIO  pronto em 127.0.0.1:41357         ↗  ■  ×
//   ┌──────────────────────────────────────────────────┐
//   │ o PIO Home dentro da IDE (KvLocalWebPage), ou o  │
//   │ estado, a mensagem e "Abrir no navegador"        │
//   └──────────────────────────────────────────────────┘
//
// Abrir a janela sobe (ou reusa) o servidor; fechar destroi a view e devolve
// a largura, mas o servidor fica 5 minutos para a proxima abertura (depois o
// core o desliga, por `platformio.homeRelease`). A view so' navega na pagina
// da sessao.
Rectangle {
    id: root

    property var controller: null
    // O SettingsController: a opcao `platformioWebView` e o "Ligar".
    property var settings: null

    signal closeRequested()
    signal widenRequested(real width)
    signal restoreWidthRequested(real width)

    radius: Theme.radiusLarge
    color: Theme.background1

    readonly property real minimumWidth: 300
    readonly property bool webEnabled: root.settings !== null && root.settings.platformioWebView
    readonly property bool ready: root.controller !== null && root.controller.ready
    // A largura de antes da pagina: fechar a janela devolve o espaco ao codigo.
    property real widthBeforeWeb: 0

    function widenForWeb() {
        if (!root.webEnabled || !root.ready || root.widthBeforeWeb > 0) return;
        root.widthBeforeWeb = root.width;
        root.widenRequested(900);
    }

    function giveBackWidth() {
        if (root.widthBeforeWeb <= 0) return;
        root.restoreWidthRequested(root.widthBeforeWeb);
        root.widthBeforeWeb = 0;
    }

    onVisibleChanged: {
        if (root.visible) {
            Qt.callLater(root.widenForWeb);
        } else {
            web.release();
            root.giveBackWidth();
        }
    }
    onReadyChanged: if (root.ready && root.visible) Qt.callLater(root.widenForWeb)

    // ---- cabecalho: titulo, estado, abrir fora, parar, fechar --------------

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
            text: qsTr("PlatformIO")
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
                iconName: "external"
                enabled: root.ready
                tooltip: qsTr("Abrir o PIO Home no navegador")
                onClicked: root.controller.openOutside()
            }

            KvIconButton {
                compact: true
                iconName: "stop"
                enabled: root.controller !== null && (root.controller.ready || root.controller.busy)
                tooltip: qsTr("Desligar o PIO Home")
                onClicked: root.controller.stop()
            }

            KvIconButton {
                compact: true
                iconName: "close"
                tooltip: qsTr("Fechar a janela do PlatformIO")
                onClicked: root.closeRequested()
            }
        }
    }

    // ---- pronto: o PIO Home (dentro da IDE ou o convite para ligar) --------

    KvLocalWebPage {
        id: web

        anchors.top: headerRow.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingSmall
        visible: root.visible && root.ready
        webEnabled: root.webEnabled
        contextShared: WebSupport.contextShared
        baseUrl: root.ready ? root.controller.url : ""
        scopeUrl: root.ready ? root.controller.url : ""
        iconName: "platformio"
        viewName: "PlatformioHomeView"
        disabledTitle: qsTr("O PIO Home dentro da IDE está desligado")
        disabledText: qsTr("Ligado, o PIO Home (projeto novo, placas, plataformas, bibliotecas) abre nesta janela, ao lado do código. Desligado, o navegador embutido nem é carregado; a opção também fica em Configurações → Interface.")
        enableLabel: qsTr("Ligar o PIO Home dentro da IDE")
        remoteText: qsTr("O PIO Home roda só em 127.0.0.1. %1 abre no navegador.").arg(web.baseUrl)
        onEnableRequested: {
            root.settings.setPlatformioWebView(true);
            Qt.callLater(root.widenForWeb);
        }
    }

    // ---- sem servidor: subindo, desligado ou com erro ------------------------

    Column {
        anchors.centerIn: parent
        width: Math.min(parent.width - 2 * Theme.spacingMedium, 420)
        spacing: Theme.spacingSmall
        visible: !root.ready

        KvIcon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: "platformio"
            size: 28
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: root.controller === null ? ""
                  : root.controller.busy ? qsTr("Subindo o PIO Home…")
                  : root.controller.failed ? qsTr("O PIO Home não subiu")
                  : qsTr("O PIO Home está desligado")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: root.controller === null ? ""
                  : root.controller.busy ? qsTr("Na primeira vez o PlatformIO baixa a interface; leva alguns segundos.")
                  : root.controller.message
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvButton {
            width: parent.width
            primary: true
            visible: root.controller !== null && !root.controller.busy
            text: root.controller !== null && root.controller.failed
                  ? qsTr("Tentar de novo") : qsTr("Ligar o PIO Home")
            onClicked: root.controller.start()
        }
    }
}
