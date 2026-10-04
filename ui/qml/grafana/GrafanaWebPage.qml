pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A ABA WEB do Grafana (0.154.0, roadmaps/59 §6.1): o Grafana local dentro
// da IDE, pelo QtWebEngine — OPCIONAL e desligado por padrao.
//
// Desligado nao pesa: nao ha' `import QtWebEngine` em lugar nenhum. A view
// nasce por `Qt.createQmlObject` so' quando a opcao esta' ligada E esta aba
// aparece; o modulo web carrega nessa hora, nao antes. Sem o modulo
// instalado, a aba diz o que instalar em vez de quebrar. `release()` destroi a
// view (e o processo do Chromium com ela) quando a janela fecha.
//
// So' endereco local: a view recusa navegar para fora de localhost,
// 127.0.0.1 e ::1, e manda o link para o navegador do sistema.
Item {
    id: root

    // A configuracao `grafanaWebView` (SettingsController).
    property bool webEnabled: false
    // O endereco salvo do Grafana do projeto.
    property string baseUrl: ""

    signal enableRequested()

    property var view: null
    property string loadError: ""
    // O endereco a abrir quando a view nascer (um dashboard clicado antes).
    property string pendingUrl: ""

    readonly property bool baseIsLocal: root.isLocal(root.baseUrl)

    function isLocal(url) {
        return /^https?:\/\/(localhost|127\.0\.0\.1|\[::1\])(:\d+)?(\/|$)/i.test(String(url));
    }

    function openOutside(url) {
        Qt.openUrlExternally(url);
    }

    // Abre `url` aqui (cria a view se preciso).
    function load(url) {
        root.pendingUrl = url;
        if (root.view !== null) root.view.url = url;
        else root.ensureView();
    }

    function ensureView() {
        if (root.view !== null || !root.visible || !root.webEnabled || !root.baseIsLocal) return;
        try {
            // Os manipuladores moram NA string: o enum
            // WebEngineNavigationRequest so' existe no escopo do import.
            // `reject()` e' do Qt 6.8+; o `action` serve ao 6.4 do AppImage.
            root.view = Qt.createQmlObject(
                "import QtQuick\n"
                + "import QtWebEngine\n"
                + "WebEngineView {\n"
                + "    property var page: null\n"
                + "    anchors.fill: parent\n"
                + "    onNavigationRequested: function(request) {\n"
                + "        if (page.isLocal(request.url)) return;\n"
                + "        if (typeof request.reject === 'function') request.reject();\n"
                + "        else request.action = WebEngineNavigationRequest.IgnoreRequest;\n"
                + "        if (request.isMainFrame === undefined || request.isMainFrame) page.openOutside(request.url);\n"
                + "    }\n"
                + "    onNewWindowRequested: function(request) { page.openOutside(request.requestedUrl); }\n"
                + "}\n", viewSlot, "GrafanaWebView");
            root.view.page = root;
            root.view.url = root.pendingUrl !== "" ? root.pendingUrl : root.baseUrl;
            root.loadError = "";
        } catch (error) {
            root.view = null;
            root.loadError = qsTr("O navegador embutido (QtWebEngine) não está instalado. No Ubuntu/Debian: sudo apt install qml6-module-qtwebengine. Enquanto isso, use Abrir no navegador.");
            console.warn("GrafanaWebPage: " + error);
        }
    }

    function release() {
        if (root.view !== null) {
            root.view.destroy();
            root.view = null;
        }
    }

    onVisibleChanged: if (root.visible) Qt.callLater(root.ensureView)
    onWebEnabledChanged: if (root.webEnabled) Qt.callLater(root.ensureView); else root.release()

    // ---- barra: voltar, recarregar, o endereco, abrir fora -----------------

    Row {
        id: toolbar

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 28
        spacing: 2
        visible: root.view !== null

        KvIconButton {
            compact: true
            iconName: "back"
            tooltip: qsTr("Voltar")
            enabled: root.view !== null && root.view.canGoBack
            onClicked: root.view.goBack()
        }

        KvIconButton {
            compact: true
            iconName: "refresh"
            tooltip: qsTr("Recarregar")
            onClicked: root.view.reload()
        }

        Text {
            width: toolbar.width - 3 * 30
            anchors.verticalCenter: parent.verticalCenter
            text: root.view !== null ? String(root.view.url) : ""
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: Theme.fontSizeMicro
            elide: Text.ElideMiddle
        }

        KvIconButton {
            compact: true
            iconName: "external"
            tooltip: qsTr("Abrir no navegador")
            onClicked: root.openOutside(root.view.url)
        }
    }

    // A linha de carregamento, ambar, sob a barra.
    Rectangle {
        anchors.top: toolbar.bottom
        anchors.left: parent.left
        height: 2
        width: root.view !== null && root.view.loading ? parent.width * root.view.loadProgress / 100 : 0
        color: Theme.accent
        visible: width > 0
    }

    Item {
        id: viewSlot

        anchors.top: toolbar.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        clip: true
    }

    // ---- sem view: desligado, endereco fora, ou modulo faltando ------------

    Column {
        anchors.centerIn: parent
        width: Math.min(parent.width - 2 * Theme.spacingMedium, 420)
        spacing: Theme.spacingSmall
        visible: root.view === null

        KvIcon {
            anchors.horizontalCenter: parent.horizontalCenter
            name: "observability"
            size: 28
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: !root.webEnabled ? qsTr("Grafana dentro da IDE está desligado")
                  : !root.baseIsLocal ? qsTr("Só um Grafana local abre aqui")
                  : qsTr("Não deu para abrir aqui")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: root.loadError !== "" ? root.loadError
                  : !root.webEnabled ? qsTr("Ligado, os dashboards abrem nesta aba, ao lado do código. Desligado, o navegador embutido nem é carregado; a opção também fica em Configurações → Interface.")
                  : qsTr("A aba Web abre só localhost, 127.0.0.1 e ::1 — é para o Grafana que você desenvolve neste projeto. %1 abre no navegador.").arg(root.baseUrl)
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvButton {
            width: parent.width
            primary: true
            visible: !root.webEnabled
            text: qsTr("Ligar o Grafana dentro da IDE")
            onClicked: root.enableRequested()
        }

        KvButton {
            width: parent.width
            text: qsTr("Abrir no navegador")
            enabled: root.baseUrl !== ""
            onClicked: root.openOutside(root.pendingUrl !== "" ? root.pendingUrl : root.baseUrl)
        }
    }
}
