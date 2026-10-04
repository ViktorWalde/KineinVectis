import QtQuick
import KineinVectis

// A janela do Grafana (0.154.0, roadmaps/59 §6.1): a aba Web so' abre
// endereco local, e desligada nunca cria a view (o modulo web nem carrega); a
// engrenagem abre e fecha os ajustes por cima da regra (o "configurar…"
// antigo so' sabia abrir).
Item {
    id: root

    width: 600
    height: 400

    GrafanaWebPage {
        id: page

        anchors.fill: parent
        webEnabled: false
        baseUrl: "http://localhost:3000"
    }

    GrafanaPanel {
        id: panel

        width: 400
        height: 300
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        let failures = 0;

        // So' localhost, 127.0.0.1 e ::1, com ou sem porta e caminho.
        failures += check(page.isLocal("http://localhost:3000"), "localhost com porta");
        failures += check(page.isLocal("http://localhost"), "localhost sem porta");
        failures += check(page.isLocal("https://127.0.0.1:3000/d/abc/painel?orgId=1"), "127.0.0.1 com caminho");
        failures += check(page.isLocal("http://[::1]:3000/"), "::1");
        failures += check(!page.isLocal("http://localhost.evil.com:3000"), "subdominio que comeca com localhost");
        failures += check(!page.isLocal("https://grafana.example.com"), "host remoto");
        failures += check(!page.isLocal("http://127.0.0.10:3000"), "outro IP");
        failures += check(!page.isLocal("file:///etc/passwd"), "arquivo");
        // Tentativas classicas de burlar o "so' local" (59 §7, seguranca).
        failures += check(!page.isLocal("http://localhost:3000@evil.com/"), "userinfo depois da porta");
        failures += check(!page.isLocal("http://localhost@evil.com/"), "userinfo sem porta");
        failures += check(!page.isLocal("http://127.0.0.1.evil.com/"), "IP como prefixo de dominio");
        failures += check(!page.isLocal("http://localhost\\@evil.com/"), "barra invertida");
        failures += check(!page.isLocal("http://localhost\n.evil.com/"), "quebra de linha");
        failures += check(!page.isLocal("javascript:alert(1)//http://localhost"), "javascript:");
        failures += check(!page.isLocal(" http://localhost:3000"), "espaco antes");
        failures += check(!page.isLocal("http://localhost:3000x"), "porta com lixo");

        // Desligada: nenhuma view, nenhum erro — o modulo web nao foi tocado.
        page.ensureView();
        failures += check(page.view === null && page.loadError === "", "desligada nao cria view");

        // Ligada com endereco remoto: tambem nao.
        page.baseUrl = "https://grafana.example.com";
        page.webEnabled = true;
        page.ensureView();
        failures += check(page.view === null, "endereco remoto nao cria view");

        // Ligada e local: cria a view, ou diz o que instalar (sem o modulo).
        page.baseUrl = "http://localhost:3000";
        page.ensureView();
        failures += check(page.view !== null || page.loadError.indexOf("qml6-module-qtwebengine") >= 0,
                          "ligada: view ou o pacote a instalar");
        page.release();
        failures += check(page.view === null, "release destroi a view");

        // A engrenagem vence a regra nos dois sentidos.
        const before = panel.setupShown;
        panel.toggleSetup();
        failures += check(panel.setupShown === !before, "engrenagem inverte");
        panel.toggleSetup();
        failures += check(panel.setupShown === before, "engrenagem volta");

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
