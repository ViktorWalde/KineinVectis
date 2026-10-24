pragma ComponentBehavior: Bound
import QtQuick

// Estado do PIO HOME (0.191.0, roadmaps/52 §14.45): o servidor do PlatformIO
// que o core sobe so' em 127.0.0.1, com sessao aleatoria, e a pagina dessa
// sessao. NAO decide nada: quem acha o `pio`, escolhe a porta e desliga o
// servidor e' o core. Nao fala com o CoreClient: pede por sinal e recebe do
// roteador.
//
// O servidor nao e' do projeto: ele vive entre workspaces. Desde a 0.192.0
// (§14.46) o agente da IDE fica conectado a ele e conta como cliente, entao o
// desligamento por inatividade do proprio PIO Home nao corre mais enquanto o
// core vive. O controller so' relata o fato: a janela fechou
// (`platformio.homeRelease`) ou voltou (`homeStart`, que reusa a sessao); o
// prazo corre no core, no relogio dele, porque o Timer do QML segue o relogio
// de animacao. Abrir a janela depois do desligamento sobe outro.
//
// A pagina tambem pede a' IDE (`event.platformio.homeCommand`): abrir o
// projeto que ela criou, ou um arquivo na linha. O controller so' traduz em
// sinais; quem abre e' o caminho de sempre.
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
    // Quanto o servidor espera com a janela fechada antes de o core o desligar.
    property int releaseSeconds: 300

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
    signal homeReleaseRequested(int afterSeconds)
    signal openProjectRequested(string path)
    signal openFileRequested(string path, int line, int column)

    function open() {
        windowRequested();
    }

    // A janela apareceu: sobe (ou reusa) o servidor.
    // Com o servidor pronto, o homeStart reusa a sessao e cancela o prazo.
    function prepare() {
        windowVisible = true;
        if (root.ready) homeStartRequested();
        else if (!root.busy) root.start();
    }

    // Subindo, o prazo sai quando ficar pronto (handleStarted).
    function close() {
        windowVisible = false;
        if (root.ready) homeReleaseRequested(root.releaseSeconds);
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
        if (!root.windowVisible) homeReleaseRequested(root.releaseSeconds);
    }

    // `false`: o core ja' nao tem servidor (o homeExited vem a caminho).
    function handleReleased(releasing) {
        if (releasing || !root.ready) return;
        url = "";
        port = 0;
        status = "off";
    }

    function handleStopped(stopped) {
        url = "";
        port = 0;
        status = "off";
        message = stopped ? qsTr("O PIO Home foi desligado. Abra de novo para religar.")
                          : qsTr("Nenhum PIO Home estava ligado.");
    }

    // "idle": o prazo do homeRelease venceu. "exited": servidor ou agente
    // terminou sozinho, que e' falha.
    function handleExited(code, reason) {
        url = "";
        port = 0;
        status = "off";
        message = reason === "idle"
                  ? qsTr("O PIO Home foi desligado com a janela fechada. Abra de novo para religar.")
                  : code === null || code === undefined
                  ? qsTr("O PIO Home foi encerrado. Abra de novo para religar.")
                  : qsTr("O PIO Home terminou sozinho (código %1). Abra de novo para religar.").arg(code);
    }

    // O core ja' respondeu a' pagina e so' deixa passar caminho absoluto.
    function handleCommand(method, params) {
        const path = params && params.path ? String(params.path) : "";
        if (path === "") return;
        if (method === "open_project") {
            openProjectRequested(path);
        } else if (method === "open_text_document") {
            openFileRequested(path, params.line > 0 ? params.line : 1,
                              params.column > 0 ? params.column : 1);
        }
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
