import QtQuick
import KineinVectis

// O PIO Home (0.191.0, roadmaps/52 §14.45): o estado do PlatformioController,
// a pagina web local presa a' sessao e a janela que a hospeda.
Item {
    id: root

    width: 900
    height: 600

    property int failures: 0
    property int startRequests: 0
    property int stopRequests: 0
    property var widened: []
    property var restored: []

    function check(ok, message) {
        if (!ok) {
            failures++;
            console.error("FALHOU: " + message);
        }
    }

    PlatformioController {
        id: controller

        onHomeStartRequested: root.startRequests++
        onHomeStopRequested: root.stopRequests++
    }

    QtObject {
        id: fakeSettings

        property bool platformioWebView: false
        function setPlatformioWebView(on) { platformioWebView = on; }
    }

    KvLocalWebPage {
        id: page

        width: 400
        height: 300
        webEnabled: false
        baseUrl: "http://127.0.0.1:41357/session/0123456789abcdef0123456789abcdef/"
        scopeUrl: "http://127.0.0.1:41357/session/0123456789abcdef0123456789abcdef/"
    }

    PlatformioWindow {
        id: window

        width: 400
        height: 300
        controller: controller
        settings: fakeSettings
        onWidenRequested: width => root.widened.push(width)
        onRestoreWidthRequested: width => root.restored.push(width)
    }

    Component.onCompleted: {
        // Abrir a janela sobe o servidor UMA vez.
        check(controller.status === "idle" && !controller.ready, "comeca parado");
        controller.prepare();
        check(controller.status === "starting" && controller.busy && root.startRequests === 1,
              "a janela aparecer pede o servidor");
        controller.prepare();
        check(root.startRequests === 1, "subindo, outra abertura nao pede de novo");

        // Pronto: a URL da sessao e a porta.
        controller.handleStarted(page.baseUrl, 41357);
        check(controller.ready && controller.statusPhrase.indexOf("41357") >= 0,
              "pronto mostra a porta: " + controller.statusPhrase);
        controller.prepare();
        check(root.startRequests === 1, "pronto, reabrir nao pede outro servidor");

        // Desligar pelo botao.
        controller.stop();
        check(root.stopRequests === 1, "parar pede ao core");
        controller.handleStopped(true);
        check(controller.off && !controller.ready && controller.url === "",
              "parado esquece a sessao");
        controller.prepare();
        check(root.startRequests === 2, "reabrir depois de parar sobe outro");

        // O servidor que termina sozinho (inatividade ou falha).
        controller.handleStarted(page.baseUrl, 41357);
        controller.handleExited(7);
        check(controller.off && controller.message.indexOf("7") >= 0,
              "saida sozinha diz o codigo: " + controller.message);
        controller.handleExited(null);
        check(controller.message.indexOf("encerrado") >= 0, "saida por sinal: " + controller.message);

        // Recusas: so' as do proprio dominio, e o pio ausente com o passo.
        controller.handleFailed("grafana.get", "outra coisa", "INTERNAL_ERROR");
        check(controller.off && !controller.failed, "erro de outro dominio nao toca o PlatformIO");
        controller.handleFailed("platformio.homeStart", "sem pio", "TOOL_NOT_FOUND");
        check(controller.failed && controller.message.indexOf("pipx install platformio") >= 0,
              "pio ausente diz como instalar: " + controller.message);
        controller.handleFailed("platformio.homeStart", "o PIO Home não respondeu em 120 s", "INTERNAL_ERROR");
        check(controller.message.indexOf("120 s") >= 0, "o motivo do core chega inteiro");
        controller.prepare();
        check(root.startRequests === 3 && controller.busy, "tentar de novo depois da falha");

        // A pagina so' navega dentro da sessao; o resto vai ao navegador.
        check(page.isAllowed(page.baseUrl + "#/projects"), "dentro da sessao");
        check(!page.isAllowed("http://127.0.0.1:41357/"), "a raiz do servidor fica fora");
        check(!page.isAllowed("http://127.0.0.1:41357/session/outra/"), "outra sessao fica fora");
        check(!page.isAllowed("http://127.0.0.1:3000/session/0123456789abcdef0123456789abcdef/"),
              "outra porta fica fora");
        check(!page.isAllowed("https://docs.platformio.org/"), "documentacao vai ao navegador");
        check(page.outsideUrl("https://docs.platformio.org/") === "https://docs.platformio.org/",
              "https sai para o navegador");
        check(page.view === null, "desligada, a pagina nunca cria a view");

        // A janela pede largura so' com a pagina ligada e o servidor pronto.
        controller.handleStarted(page.baseUrl, 41357);
        window.widenForWeb();
        check(root.widened.length === 0, "sem a opcao, a janela nao alarga");
        fakeSettings.platformioWebView = true;
        window.widenForWeb();
        check(root.widened.length === 1 && root.widened[0] === 900, "com a opcao, alarga para a pagina");
        window.widenForWeb();
        check(root.widened.length === 1, "alarga uma vez");
        window.giveBackWidth();
        check(root.restored.length === 1, "devolve a largura");
        controller.handleStopped(true);
        check(!window.ready, "parado, a janela volta ao convite");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
