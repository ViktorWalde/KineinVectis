pragma ComponentBehavior: Bound
import QtQuick

// Estado do PIO HOME (0.191.0, roadmaps/52 §14.45): o servidor do PlatformIO
// que o core sobe so' em 127.0.0.1, com sessao aleatoria, e a pagina dessa
// sessao. NAO decide nada: quem acha o `pio`, escolhe a porta e desliga o
// servidor e' o core. Nao fala com o CoreClient: pede por sinal e recebe do
// roteador.
//
// O servidor nao e' do projeto: ele vive entre workspaces, e o proprio PIO
// Home se desliga depois de 5 minutos sem pagina conectada (o core avisa por
// `event.platformio.homeExited`). Abrir a janela de novo sobe outro.
Item {
    id: root

    visible: false

    // "idle" | "starting" | "ready" | "off" (parado ou encerrado; a mensagem
    // diz qual) | "error". Cada derivacao abaixo mora so' aqui: a janela le
    // `ready`, `busy` e `failed`.
    property string status: "idle"
    // A pagina da sessao; vazia fora de "ready".
    property string url: ""
    property int port: 0
    // O que aconteceu por ultimo, para a janela dizer.
    property string message: ""
    property bool windowVisible: false

    readonly property bool ready: root.status === "ready" && root.url !== ""
    readonly property bool busy: root.status === "starting"
    readonly property bool failed: root.status === "error"
    readonly property bool off: root.status === "off"
    readonly property string statusPhrase:
        root.busy ? qsTr("subindo o PIO Home…")
        : root.ready ? qsTr("pronto em 127.0.0.1:%1").arg(root.port)
        : root.failed ? qsTr("não subiu")
        : root.off ? qsTr("desligado")
        : ""

    signal windowRequested()
    signal homeStartRequested()
    signal homeStopRequested()

    function open() {
        windowRequested();
    }

    // A janela apareceu: sobe (ou reusa) o servidor.
    function prepare() {
        windowVisible = true;
        if (!root.ready && !root.busy) root.start();
    }

    function close() {
        windowVisible = false;
    }

    function start() {
        status = "starting";
        message = "";
        homeStartRequested();
    }

    function stop() {
        if (root.ready || root.busy) homeStopRequested();
    }

    // A URL da sessao no navegador do sistema: so' http local.
    function openOutside() {
        if (root.ready) Qt.openUrlExternally(root.url);
    }

    function handleStarted(newUrl, newPort) {
        url = newUrl;
        port = newPort;
        status = "ready";
        message = "";
    }

    function handleStopped(stopped) {
        url = "";
        port = 0;
        status = "off";
        message = stopped ? qsTr("O PIO Home foi desligado. Abra de novo para religar.")
                          : qsTr("Nenhum PIO Home estava ligado.");
    }

    function handleExited(code) {
        url = "";
        port = 0;
        status = "off";
        message = code === null || code === undefined
                  ? qsTr("O PIO Home foi encerrado. Abra de novo para religar.")
                  : qsTr("O PIO Home terminou (código %1): sem página aberta por 5 minutos ou por falha. Abra de novo para religar.").arg(code);
    }

    function handleFailed(method, text, code) {
        if (String(method).indexOf("platformio.home") !== 0) return;
        url = "";
        port = 0;
        status = "error";
        message = code === "TOOL_NOT_FOUND"
                  ? qsTr("O PlatformIO Core não está nesta máquina. Instale com: pipx install platformio")
                  : text;
    }
}
