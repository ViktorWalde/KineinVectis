pragma ComponentBehavior: Bound
import QtQuick

// Estado do painel de embarcados (roadmaps/35 §5.7, fatia 1 — o FIO).
//
// Guarda o que o core respondeu ao `probe.list`: a sonda reconhecida, a saida
// crua da ferramenta e a dica quando nao ha' nada para mostrar. NAO decide
// nada: quem roda o probe-rs, quem le a saida e quem escolhe a dica e' o core.
// O chip, o alvo e o depurador moram no `ToolchainController` — sao KIT, nao
// sonda — e o painel os edita por ele.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    property string workspaceRoot: ""
    property bool panelVisible: false
    property var probes: []
    // `true` ate' o core dizer o contrario: a tela nao acusa ferramenta ausente
    // antes de perguntar.
    property bool toolAvailable: true
    property string rawOutput: ""
    property string hint: ""
    property bool busy: false
    property string errorText: ""

    readonly property bool probeFound: probes.length > 0

    signal listRequested()

    visible: false

    onWorkspaceRootChanged: {
        probes = [];
        toolAvailable = true;
        rawOutput = "";
        hint = "";
        busy = false;
        errorText = "";
        panelVisible = false;
    }

    function open() {
        panelVisible = true;
        refresh();
    }

    function close() {
        panelVisible = false;
    }

    // Toda abertura PERGUNTA de novo: sonda e' coisa que se pluga e despluga,
    // e a lista da ultima vez e' exatamente o que nao se pode mostrar.
    function refresh() {
        busy = true;
        errorText = "";
        listRequested();
    }

    function handleProbes(newProbes, newToolAvailable, newRawOutput, newHint) {
        probes = newProbes === undefined ? [] : newProbes;
        toolAvailable = newToolAvailable === undefined ? true : newToolAvailable;
        rawOutput = newRawOutput === undefined ? "" : newRawOutput;
        hint = newHint === undefined ? "" : newHint;
        busy = false;
        errorText = "";
    }

    function handleFailed(method, message) {
        if (method !== "probe.list") {
            return;
        }
        busy = false;
        errorText = message;
    }

    // Uma linha por sonda: nome, familia, VID:PID e serial — o que o probe-rs
    // reportou, na ordem em que se procura no `lsusb`. Campo ausente nao vira
    // "undefined" na tela.
    function probeSummary(probe) {
        const partes = [probe.name];
        if (probe.kind !== undefined && probe.kind !== "") {
            partes.push(probe.kind);
        }
        partes.push(probe.vid + ":" + probe.pid);
        if (probe.serial !== undefined && probe.serial !== "") {
            partes.push(probe.serial);
        }
        return partes.join(" · ");
    }
}
