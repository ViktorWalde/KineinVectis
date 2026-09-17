pragma ComponentBehavior: Bound
import QtQuick

// Permissao POR CANAL (E2 do integracoes/38 §6, 2026-09-17): o que o core
// MEDIU em cada canal — o no' serial (grupo ou ACL do udev), o ModemManager
// e a regra udev das sondas — e o passo OFICIAL, com fonte e data, para o
// que falta. Filho do EmbeddedController (`embeddedController.access`),
// como `identity` e `flash`: nenhuma propriedade de repasse nova.
//
// A IDE nunca roda sudo. `runStep(command)` emite o comando para ser
// ESCRITO no terminal da IDE (o mesmo caminho do painel de instalacao,
// ShellEnvironmentOverlays -> RuntimeController.submitShellInput); o prompt
// de senha aparece la', a vista.
Item {
    id: root

    property bool busy: false
    property var channels: []
    property string errorText: ""
    readonly property bool diagnosed: channels.length > 0
    readonly property int problems: countProblems(channels)

    signal diagnoseRequested(string device)
    signal commandRequested(string command)

    visible: false

    function clear() {
        busy = false;
        channels = [];
        errorText = "";
    }

    // Um pedido por vez. `device` vazio = todas as portas.
    function diagnose(device) {
        if (busy) {
            return;
        }
        busy = true;
        errorText = "";
        diagnoseRequested(device === undefined ? "" : device);
    }

    function handleChannels(novos) {
        busy = false;
        channels = novos === undefined || novos === null ? [] : novos;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method !== "serial.access") {
            return;
        }
        busy = false;
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

    // O passo vai para o terminal da IDE: escrito, visivel, com o prompt de
    // senha do sudo la'. Nada roda escondido.
    function runStep(command) {
        if (command === undefined || command === "") {
            return;
        }
        commandRequested(command);
    }
}
