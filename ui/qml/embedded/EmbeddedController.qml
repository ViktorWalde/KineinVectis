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

    // Tamanho do ELF (build.size): regioes do linker script com uso, e as
    // secoes cruas. `sizeMeasured` separa "ainda nao medi" de "medi e deu zero".
    property var sizeRegions: []
    property var sizeSections: []
    property string sizeTool: ""
    property bool sizeToolAvailable: true
    property bool sizeMeasured: false
    property bool sizeBusy: false

    readonly property bool probeFound: probes.length > 0

    signal listRequested()
    signal sizeRequested(string program)

    visible: false

    onWorkspaceRootChanged: {
        probes = [];
        toolAvailable = true;
        rawOutput = "";
        hint = "";
        busy = false;
        errorText = "";
        sizeRegions = [];
        sizeSections = [];
        sizeTool = "";
        sizeToolAvailable = true;
        sizeMeasured = false;
        sizeBusy = false;
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
        if (method === "build.size") {
            sizeBusy = false;
            errorText = message;
            return;
        }
        if (method !== "probe.list") {
            return;
        }
        busy = false;
        errorText = message;
    }

    // O core resolve o ELF sozinho (como o debug.start); por isso o programa
    // vai vazio. Sobrescrever nao e' desta fatia.
    function measureSize() {
        sizeBusy = true;
        errorText = "";
        sizeRequested("");
    }

    function handleBuildSize(sections, regions, toolAvailable, tool, rawOutput) {
        sizeSections = sections === undefined ? [] : sections;
        sizeRegions = regions === undefined ? [] : regions;
        sizeToolAvailable = toolAvailable === undefined ? true : toolAvailable;
        sizeTool = tool === undefined ? "" : tool;
        sizeMeasured = true;
        sizeBusy = false;
    }

    // Fracao usada de uma regiao, 0..1. Regiao sem capacidade (nao deveria
    // ocorrer) nao divide por zero.
    function fracaoUsada(region) {
        return region.size > 0 ? region.used / region.size : 0;
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
