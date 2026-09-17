pragma ComponentBehavior: Bound
import QtQuick

// Os ARQUIVOS NA PLACA (serial.files, C2 do roadmaps/41 bloco C,
// 2026-09-17): o que o `mpremote fs` ve no sistema de arquivos de um
// MicroPython — listar, baixar, enviar, apagar, criar pasta. Referencia de
// UX: o "Files on device" do Thonny. Filho do EmbeddedController
// (`embeddedController.files`), pela mesma razao de identity/flash/access.
//
// Tres decisoes de produto moram aqui:
//   1. Tudo e' GESTO EXPLICITO: qualquer `fs` interrompe o programa da
//      placa (raw REPL + soft reset ao sair). Nada roda ao abrir o painel.
//   2. O que ESCREVE na placa (enviar, apagar, criar pasta) pede confirmacao
//      antes de sair daqui — `pending` guarda o gesto ate' o segundo clique.
//   3. Baixar vai para `placa/<caminho>` sob o workspace: o espelho da
//      placa, que nunca colide com o codigo do projeto e pode ser baixado
//      de novo. O core cria a pasta; ao chegar, `openLocalRequested` abre
//      o arquivo no editor (fiacao no AppDomains).
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    // A porta perguntada; vazio = nunca perguntou (ou workspace trocou).
    property string device: ""
    // A pasta na placa que a lista mostra ("" = a raiz).
    property string path: ""
    property bool busy: false
    property string jobId: ""
    property string command: ""
    property string lastAction: ""
    property var entries: []
    property bool listed: false
    property string errorText: ""
    property string rawOutput: ""
    // O gesto que ESCREVE, esperando o segundo clique: { action, path, local }.
    property var pending: ({})
    readonly property bool hasPending: pending.action !== undefined

    signal filesRequested(string device, string action, string path, string local)
    // O arquivo baixado chegou: quem abre e' o editor (AppDomains).
    signal openLocalRequested(string local)
    // "Enviar o arquivo aberto": quem sabe qual e' o editor (AppDomains).
    signal uploadCurrentRequested()

    visible: false

    function clear() {
        device = "";
        path = "";
        busy = false;
        jobId = "";
        command = "";
        lastAction = "";
        entries = [];
        listed = false;
        errorText = "";
        rawOutput = "";
        pending = ({});
    }

    // O caminho de `nome` dentro da pasta atual da placa.
    function joined(nome) {
        return path === "" ? nome : path + "/" + nome;
    }

    // Um pedido por vez: o mpremote prende a porta.
    function request(action, boardPath, local) {
        if (busy || device === "") {
            return false;
        }
        busy = true;
        errorText = "";
        lastAction = action;
        filesRequested(device, action, boardPath, local === undefined ? "" : local);
        return true;
    }

    // Listar `pasta` da porta `porta` (a raiz quando omitida). Trocar de
    // porta esquece a lista da outra.
    function list(porta, pasta) {
        if (busy || porta === undefined || porta === "") {
            return;
        }
        if (porta !== device) {
            clear();
            device = porta;
        }
        path = pasta === undefined ? "" : pasta;
        pending = ({});
        request("list", path, "");
    }

    function refresh() {
        list(device, path);
    }

    function enterDir(nome) {
        list(device, joined(nome));
    }

    function up() {
        const i = path.lastIndexOf("/");
        list(device, i < 0 ? "" : path.substring(0, i));
    }

    // Baixar para o espelho `placa/<caminho>` do workspace (relativo: o core
    // resolve sob o workspace e cria a pasta).
    function download(nome) {
        const alvo = joined(nome);
        request("get", alvo, "placa/" + alvo);
    }

    // Os gestos que ESCREVEM ficam pendentes ate' confirmPending().
    function upload(local, nome) {
        if (local === undefined || local === "") {
            errorText = qsTr("nenhum arquivo aberto no editor para enviar");
            return;
        }
        const base = local.substring(local.lastIndexOf("/") + 1);
        pending = ({ action: "put", path: joined(nome === undefined || nome === "" ? base : nome), local: local });
    }

    function remove(nome) {
        pending = ({ action: "rm", path: joined(nome), local: "" });
    }

    function makeDir(nome) {
        if (nome === undefined || nome.trim() === "") {
            return;
        }
        pending = ({ action: "mkdir", path: joined(nome.trim()), local: "" });
    }

    function confirmPending() {
        if (!hasPending) {
            return;
        }
        const p = pending;
        pending = ({});
        request(p.action, p.path, p.local);
    }

    function cancelPending() {
        pending = ({});
    }

    // Uma frase para a confirmacao, sem "undefined".
    function pendingText() {
        if (!hasPending) return "";
        if (pending.action === "put") return qsTr("Enviar %1 para :%2 na placa? (sobrescreve)").arg(pending.local).arg(pending.path);
        if (pending.action === "rm") return qsTr("Apagar :%1 da placa?").arg(pending.path);
        return qsTr("Criar a pasta :%1 na placa?").arg(pending.path);
    }

    function handleStarted(newJobId, newCommand) {
        jobId = newJobId;
        command = newCommand;
    }

    // O desfecho. Um desfecho de OUTRA porta (pedido antigo) nao sobrescreve.
    function handleOutcome(outcome) {
        if (outcome.device !== device) {
            return;
        }
        busy = false;
        if (outcome.command !== undefined && outcome.command !== "") {
            command = outcome.command;
        }
        rawOutput = outcome.raw !== undefined ? outcome.raw : "";
        if (!outcome.success) {
            errorText = outcome.error !== undefined && outcome.error !== "" ? outcome.error : qsTr("o mpremote falhou");
            return;
        }
        errorText = "";
        if (outcome.action === "list") {
            entries = outcome.entries !== undefined && outcome.entries !== null ? outcome.entries : [];
            listed = true;
            return;
        }
        if (outcome.action === "get") {
            if (outcome.local !== undefined && outcome.local !== "") {
                openLocalRequested(outcome.local);
            }
            return;
        }
        // Escreveu na placa: a lista que a tela mostra envelheceu.
        refresh();
    }

    // A recusa ANTES de tocar a porta (sem mpremote, sem permissao, pedido
    // invalido): e' a resposta, no lugar do resultado.
    function handleFailed(method, message) {
        if (method !== "serial.files") {
            return;
        }
        busy = false;
        errorText = message;
    }

    // Uma linha por entrada, sem "undefined".
    function entrySummary(e) {
        if (e.directory === true) return e.name + "/";
        return e.name + (e.size !== undefined ? "  " + e.size + " B" : "");
    }
}
