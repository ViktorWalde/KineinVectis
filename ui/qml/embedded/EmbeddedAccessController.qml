pragma ComponentBehavior: Bound
import QtQuick

// Permissao POR CANAL (E2 do integracoes/38 §6, 2026-09-17): o que o core
// MEDIU em cada canal — o no' serial (grupo ou ACL do udev), o ModemManager
// e a regra udev das sondas — e o passo OFICIAL, com fonte e data, para o
// que falta. Filho do EmbeddedController (`embeddedController.access`),
// como `identity` e `flash`: nenhuma propriedade de repasse nova.
//
// A IDE nunca eleva privilégios automaticamente. `runStep(command)` emite o comando para ser
// EXECUTADO no terminal da IDE por clique explícito (o mesmo caminho do painel de instalacao,
// ShellEnvironmentOverlays -> RuntimeController.runInNewTerminal); o prompt
// de senha aparece la', a vista.
Item {
    id: root

    property bool busy: false
    property bool discardReply: false
    property var channels: []
    property string focusedDevice: ""
    readonly property var visibleChannels: focusedDevice === "" ? channels
        : channels.filter(channel => channel.device === focusedDevice)
    property string errorText: ""
    readonly property bool diagnosed: channels.length > 0
    readonly property int problems: countProblems(visibleChannels)

    signal diagnoseRequested(string device)
    signal commandRequested(string command)
    signal refreshRequested()

    visible: false

    function clear(disconnected) {
        // Keep a single request in flight; its reply must not revive cleared data.
        if (disconnected === true) busy = false;
        discardReply = busy;
        channels = [];
        focusedDevice = "";
        errorText = "";
    }

    // Um pedido por vez. `device` vazio = todas as portas.
    function diagnose(device) {
        if (busy) {
            return;
        }
        busy = true;
        errorText = "";
        channels = [];
        focusedDevice = device === undefined ? "" : device;
        diagnoseRequested(focusedDevice);
    }

    function recheck() {
        if (busy) return;
        refreshRequested();
        diagnose(focusedDevice);
    }

    function handleChannels(novos) {
        if (!busy) return;
        busy = false;
        if (discardReply) { discardReply = false; return; }
        channels = novos === undefined || novos === null ? [] : novos;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method !== "serial.access") {
            return;
        }
        if (!busy) return;
        busy = false;
        if (discardReply) { discardReply = false; return; }
        errorText = message;
    }

    function countProblems(lista) {
        let n = 0;
        for (let i = 0; i < lista.length; i++) {
            if (!lista[i].ok) n += 1;
        }
        return n;
    }

    // O rotulo do canal: o que ele e', e a porta quando e' por porta.
    function channelTitle(canal) {
        const nomes = { serial: qsTr("porta serial"), probe: qsTr("sondas (regra udev)"),
                        modemManager: qsTr("ModemManager") };
        const nome = nomes[canal.kind] !== undefined ? nomes[canal.kind] : String(canal.kind);
        return canal.device !== undefined && canal.device !== null && canal.device !== ""
               ? nome + " · " + canal.device : nome;
    }

    function stepsOf(canal) {
        return canal.fix !== undefined && canal.fix !== null && canal.fix.steps !== undefined
               ? canal.fix.steps : [];
    }

    // O passo vai para o terminal da IDE com Enter, visivel, com o prompt de
    // senha do sudo la'. Nada roda escondido.
    function runStep(command) {
        if (command === undefined || command === "") {
            return;
        }
        commandRequested(command);
    }
}
