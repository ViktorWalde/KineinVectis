pragma ComponentBehavior: Bound
import QtQuick

// Estado do painel de containers (roadmaps/28 §0: Docker e Podman como dominio
// NATIVO; priorizado pelo autor em 2026-09-12).
//
// Guarda o que o core respondeu: o ESTADO do motor (existe? versao? rootless?
// socket? responde? compose?) — que e' a tela de "ativar a ferramenta" —, a
// lista de containers e imagens, e o ultimo job terminado. NAO decide nada e
// NAO chama motor nenhum (invariante do 28 §4): pede por sinal, recebe do
// roteador. Logs e shell viram uma aba de terminal, cuidada pelo
// RuntimeController como qualquer outra.
Item {
    id: root

    property string workspaceRoot: ""
    property bool panelVisible: false

    // container.status — o mapa inteiro do core, e o que a tela le dele.
    property var status: ({})
    readonly property bool engineFound: status.engine !== undefined && status.engine !== null
                                        && status.engine !== ""
    readonly property bool reachable: status.reachable === true
    readonly property string engineLabel: {
        if (!engineFound) return "";
        const nome = status.engine === "podman" ? "Podman" : "Docker";
        const versao = status.version !== undefined && status.version !== "" ? " " + status.version : "";
        const emulado = status.emulated === true ? qsTr(" (o `docker` do PATH é o shim do Podman)") : "";
        return nome + versao + emulado;
    }
    property bool statusBusy: false
    // O compose e' do PROJETO: a ferramenta e' da maquina (`status.compose`),
    // o arquivo e' do workspace aberto (`status.composeFile`, o que o core
    // achou na raiz). Sem os dois, `compose up` so' poderia falhar — e o botao
    // nao promete.
    readonly property string composeTool: status.compose !== undefined && status.compose !== null
                                          ? status.compose : ""
    readonly property string composeFile: status.composeFile !== undefined && status.composeFile !== null
                                          ? status.composeFile : ""
    readonly property bool canCompose: reachable && composeTool !== "" && composeFile !== ""
    // A frase do compose na linha do motor: o que existe e o que falta.
    readonly property string composeSummary: {
        if (composeTool === "") return qsTr("sem compose");
        if (workspaceRoot === "") return qsTr("compose: %1 · abra um projeto").arg(composeTool);
        if (composeFile === "") return qsTr("compose: %1 · o projeto não tem compose.yaml").arg(composeTool);
        return qsTr("compose: %1 · %2").arg(composeTool).arg(composeFile);
    }

    property var containers: []
    property string containersEngine: ""
    property string containersRaw: ""
    property string containersHint: ""
    property bool listBusy: false
    property bool showAll: true

    property var images: []
    property string imagesHint: ""
    property bool imagesBusy: false

    // O ultimo `event.container.finished`: acao, alvo, ok e a mensagem. A tela
    // mostra o que deu errado; o que deu certo so' atualiza a lista.
    property var lastFinished: ({})
    property string errorText: ""

    readonly property bool containerFound: containers.length > 0
    // Logs e shell sao abas de terminal, e a aba pertence ao projeto aberto:
    // sem workspace o core recusa (`container.open` exige root para o cwd).
    // O painel abre sem projeto (o motor e' da maquina) — os botoes nao.
    readonly property bool canOpenTerminals: workspaceRoot !== ""

    signal statusRequested()
    signal listRequested(bool all)
    signal imagesRequested()
    signal actionRequested(string id, string action)
    signal openRequested(string id, string mode)
    signal composeRequested(string action, string file)

    visible: false

    onWorkspaceRootChanged: {
        // O motor e' da MAQUINA, nao do projeto: o status sobrevive a troca de
        // workspace. A lista tambem — mas o painel fecha, como os outros.
        panelVisible = false;
        errorText = "";
    }

    function open() {
        panelVisible = true;
        refresh();
    }

    function close() {
        panelVisible = false;
    }

    // Toda abertura PERGUNTA de novo: container e' coisa que sobe e cai fora
    // da IDE, e a lista da ultima vez e' exatamente o que nao se pode mostrar.
    function refresh() {
        statusBusy = true;
        listBusy = true;
        imagesBusy = true;
        errorText = "";
        statusRequested();
        listRequested(showAll);
        imagesRequested();
    }

    function refreshList() {
        listBusy = true;
        listRequested(showAll);
    }

    function setShowAll(value) {
        showAll = value;
        refreshList();
    }

    function handleStatus(newStatus) {
        status = newStatus === undefined || newStatus === null ? ({}) : newStatus;
        statusBusy = false;
    }

    function handleContainers(newContainers, engine, rawOutput, hint) {
        containers = newContainers === undefined ? [] : newContainers;
        containersEngine = engine === undefined ? "" : engine;
        containersRaw = rawOutput === undefined ? "" : rawOutput;
        containersHint = hint === undefined ? "" : hint;
        listBusy = false;
    }

    function handleImages(newImages, hint) {
        images = newImages === undefined ? [] : newImages;
        imagesHint = hint === undefined ? "" : hint;
        imagesBusy = false;
    }

    // O job acabou: se falhou, o motivo vai para a tela; em todo caso a lista
    // e' perguntada de novo, porque o estado mudou fora daqui.
    function handleFinished(event) {
        lastFinished = event === undefined || event === null ? ({}) : event;
        if (lastFinished.ok === false) {
            errorText = qsTr("%1 %2 falhou: %3").arg(lastFinished.action).arg(lastFinished.target)
                        .arg(lastFinished.message !== undefined ? lastFinished.message : "");
        } else {
            errorText = "";
        }
        refreshList();
    }

    function handleFailed(method, message) {
        if (method === "container.status") {
            statusBusy = false;
        } else if (method === "container.list") {
            listBusy = false;
        } else if (method === "container.images") {
            imagesBusy = false;
        } else if (method !== "container.action" && method !== "container.open"
                   && method !== "container.compose") {
            return;
        }
        errorText = message;
    }

    function act(id, action) {
        errorText = "";
        actionRequested(id, action);
    }

    function openLogs(id) {
        if (!canOpenTerminals) {
            errorText = qsTr("abra um projeto para ver os logs: a aba de terminal é do projeto");
            return;
        }
        openRequested(id, "logs");
    }

    function openShell(id) {
        if (!canOpenTerminals) {
            errorText = qsTr("abra um projeto para o shell: a aba de terminal é do projeto");
            return;
        }
        openRequested(id, "shell");
    }

    function composeUp() {
        errorText = "";
        composeRequested("up", "");
    }

    function composeDown() {
        errorText = "";
        composeRequested("down", "");
    }

    // O que se pode fazer com um container depende do estado que o motor
    // declarou — nunca se oferece "parar" ao que ja' parou.
    function isRunning(container) {
        return container.state === "running";
    }

    // Uma linha por container: nome, imagem, portas e o status humano do
    // motor. Campo ausente nao vira "undefined" na tela.
    function containerSummary(container) {
        const partes = [];
        if (container.names !== undefined && container.names.length > 0) {
            partes.push(container.names.join(", "));
        } else {
            partes.push(String(container.id).substring(0, 12));
        }
        if (container.image !== undefined && container.image !== "") {
            partes.push(container.image);
        }
        if (container.ports !== undefined && container.ports.length > 0) {
            partes.push(container.ports.join(" "));
        }
        return partes.join(" · ");
    }

    function imageSummary(image) {
        const nome = (image.repository !== undefined ? image.repository : "") + ":"
                     + (image.tag !== undefined ? image.tag : "");
        return nome + " · " + formatSize(image.size);
    }

    // Bytes (Podman) viram MB/GB; texto pronto (Docker: "431MB") passa direto.
    function formatSize(size) {
        const n = Number(size);
        if (size === undefined || size === "" || isNaN(n)) {
            return size === undefined ? "" : String(size);
        }
        if (n >= 1e9) return (n / 1e9).toFixed(2) + " GB";
        if (n >= 1e6) return (n / 1e6).toFixed(1) + " MB";
        if (n >= 1e3) return (n / 1e3).toFixed(0) + " kB";
        return n + " B";
    }
}
