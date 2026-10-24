pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// UMA PAGINA WEB LOCAL dentro da IDE, pelo QtWebEngine — OPCIONAL e desligada
// por padrao. Nasceu como a aba Web do Grafana (0.154.0, roadmaps/59 §6.1) e
// virou componente quando o PIO Home precisou da mesma coisa (52 §14.45).
//
// Desligada nao pesa: nao ha' `import QtWebEngine` em lugar nenhum. A view
// nasce por `Qt.createQmlObject` so' quando a opcao esta' ligada E a pagina
// aparece; o modulo web carrega nessa hora, nao antes. Sem o modulo
// instalado, a pagina diz o que instalar em vez de quebrar. `release()` destroi
// a view (e o processo do Chromium com ela) quando a janela fecha.
//
// So' endereco local: a view recusa navegar para fora de localhost,
// 127.0.0.1 e ::1 (e, com `scopeUrl`, para fora desse prefixo), e manda o
// link para o navegador do sistema. Quem usa da' os textos e o icone.
Item {
    id: root

    // A opcao de visualizacao web de quem usa (SettingsController).
    property bool webEnabled: false
    // O contexto OpenGL compartilhado desta abertura (WebSupport, 16.6).
    property bool contextShared: true
    // O endereco que a pagina abre.
    property string baseUrl: ""
    // Prefixo opcional: com ele, so' enderecos que comecam assim navegam aqui
    // (o PIO Home fica na pagina da sessao; o resto vai ao navegador).
    property string scopeUrl: ""
    // Os textos e o icone de quem usa (o Grafana, o PIO Home).
    property string iconName: "external"
    property string disabledTitle: ""
    property string disabledText: ""
    property string enableLabel: ""
    property string remoteTitle: qsTr("Só um endereço local abre aqui")
    property string remoteText: ""
    property string viewName: "LocalWebView"

    signal enableRequested()

    property var view: null
    property string loadError: ""
    // O endereco a abrir quando a view nascer (um dashboard clicado antes).
    property string pendingUrl: ""

    readonly property bool baseIsLocal: root.isLocal(root.baseUrl)

    // Local e, com `scopeUrl`, dentro dele.
    function isAllowed(url) {
        return root.isLocal(url) && (root.scopeUrl === "" || String(url).indexOf(root.scopeUrl) === 0);
    }

    function isLocal(url) {
        return /^https?:\/\/(localhost|127\.0\.0\.1|\[::1\])(:\d+)?(\/|$)/i.test(String(url));
    }

    // So' `http`/`https` vao ao navegador (16.4c, roadmaps/59 §7.1.4): um
    // `file://`, `smb://` ou esquema de aplicativo numa pagina chegaria ao
    // `xdg-open`, que abre (ou executa) o que estiver la'. O resto fica.
    function outsideUrl(url) {
        const text = String(url);
        return /^https?:\/\/[^\s]+$/i.test(text) ? text : "";
    }

    function openOutside(url) {
        const outside = root.outsideUrl(url);
        if (outside !== "") Qt.openUrlExternally(outside);
    }

    // Abre `url` aqui (cria a view se preciso).
    function load(url) {
        root.pendingUrl = url;
        if (root.view !== null) root.view.url = url;
        else root.ensureView();
    }

    function ensureView() {
        if (root.view !== null || !root.visible || !root.webEnabled || !root.baseIsLocal) return;
        // O contexto OpenGL compartilhado so' e' ligado na abertura com a
        // opcao ligada (16.6): sem ele o QtWebEngine nao desenha.
        if (!root.contextShared) {
            root.loadError = qsTr("A página dentro da IDE vale na próxima abertura da IDE: a opção foi ligada com ela aberta. Enquanto isso, use Abrir no navegador.");
            return;
        }
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
                // Recusa por padrao (seguranca, 59 §7): a pagina nao abre
                // janela, nao le a area de transferencia e nao roda plugin.
                + "    settings.javascriptCanOpenWindows: false\n"
                + "    settings.javascriptCanAccessClipboard: false\n"
                + "    settings.pluginsEnabled: false\n"
                + "    settings.localContentCanAccessFileUrls: false\n"
                + "    settings.localContentCanAccessRemoteUrls: false\n"
                + "    onNavigationRequested: function(request) {\n"
                + "        if (page.isAllowed(request.url)) return;\n"
                + "        if (typeof request.reject === 'function') request.reject();\n"
                + "        else request.action = WebEngineNavigationRequest.IgnoreRequest;\n"
                + "        if (request.isMainFrame === undefined || request.isMainFrame) page.openOutside(request.url);\n"
                + "    }\n"
                + "    onNewWindowRequested: function(request) { page.openOutside(request.requestedUrl); }\n"
                + "}\n", viewSlot, root.viewName);
            root.view.page = root;
            root.denyPermissions(root.view);
            root.view.url = root.pendingUrl !== "" ? root.pendingUrl : root.baseUrl;
            root.loadError = "";
        } catch (error) {
            root.view = null;
            root.loadError = qsTr("O navegador embutido (QtWebEngine) não está instalado. No Ubuntu/Debian: sudo apt install qml6-module-qtwebengine. Enquanto isso, use Abrir no navegador.");
            console.warn("KvLocalWebPage: " + error);
        }
    }

    // Camera, microfone, localizacao, notificacao, tela: uma pagina local
    // de ferramenta nao precisa de nada disso, e a resposta e' NAO sem perguntar. O sinal mudou
    // de nome no Qt 6.8; um manipulador com o nome errado na string faria a
    // criacao falhar no outro Qt, entao a conexao e' pelo que a view tem.
    // Downloads nao aceitos sao cancelados pelo proprio QtWebEngine.
    function denyPermissions(view) {
        if (typeof view.permissionRequested === "function") {
            view.permissionRequested.connect(permission => permission.deny());
        } else if (typeof view.featurePermissionRequested === "function") {
            view.featurePermissionRequested.connect(
                (origin, feature) => view.grantFeaturePermission(origin, feature, false));
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
            name: root.iconName
            size: 28
        }

        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            text: !root.webEnabled ? root.disabledTitle
                  : !root.baseIsLocal ? root.remoteTitle
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
                  : !root.webEnabled ? root.disabledText
                  : root.remoteText
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        KvButton {
            width: parent.width
            primary: true
            visible: !root.webEnabled
            text: root.enableLabel
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
