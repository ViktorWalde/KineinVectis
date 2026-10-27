pragma ComponentBehavior: Bound
import QtQuick

// "Gravar" como CONFIGURACAO DE EXECUCAO (E4 do integracoes/38 §6; decisao
// do autor em 2026-09-11: nao e' dominio novo). O core compoe a linha do
// motor (runConfig.flashProposal, sem salvar/gravar; consulta versao) e este
// controller guarda a PREVIA: nome, comando, motor, evidencias e avisos.
//
// Run/save transportam a intenção nativa intacta, ou um comando manual
// explicitamente escolhido. A prévia não executa nem salva por conta própria.
// A fiacao dos dois mora no AppDomains (donos diferentes). Filho do
// EmbeddedController (`embeddedController.flash`), pela mesma razao do
// `identity`: nenhuma propriedade de repasse nova.
Item {
    id: root

    property bool busy: false
    property bool nativeMode: true
    property string buildDirectory: ""
    // O motor que o usuario ESCOLHEU; vazio = o que o modelo do projeto
    // sugere (esptool/probe-rs/picotool/dfu-util decidido pelo core).
    property string engine: ""
    property var proposal: ({})
    property string errorText: ""
    // O firmware BAIXADO do catalogo (C5) que a linha grava no lugar dos
    // artefatos do build; vazio = os artefatos do build. O id vem da lista
    // de instalaveis do ToolchainController (kind "firmware", installed).
    property string firmware: ""
    property int contextRevision: 0
    property int requestedRevision: 0
    property double proposalRequestId: -1
    readonly property bool found: proposal.command !== undefined && proposal.command !== ""
    readonly property var engines: ["esptool", "probe-rs", "picotool", "dfu-util"]
    // Os frameworks do projeto (project.model.frameworks, do pai): o wrapper
    // de cada um entra como motor (bloco E do roadmaps/41): idf.py, west, pio.
    property var frameworks: []
    readonly property var frameworkEngines: engineOfFrameworks(frameworks)
    readonly property var allEngines: engines.concat(frameworkEngines)
    // probe-rs nativo (0.188.0, 52 §14.35): o alvo executavel do codemodel e a
    // sonda listada. O motor sugerido e a raiz vem do pai; o core valida tudo.
    property string suggestedEngine: ""
    property string workspaceRoot: ""
    property string target: ""
    property string probe: ""
    property var targets: []
    property string targetsOrigin: ""
    property string targetsError: ""
    property bool targetsBusy: false
    readonly property alias dfu: dfuSelection
    readonly property bool dfuDestination: nativeMode && firmware === ""
        && (engine === "dfu-util" || (engine === "" && suggestedEngine === "dfu-util"))
    EmbeddedDfuSelectionController { id: dfuSelection; onChanged: { if (root.dfuDestination) root.invalidate(); } }
    readonly property bool probeDestination: nativeMode && firmware === ""
        && (engine === "probe-rs" || (engine === "" && suggestedEngine === "probe-rs"))
    // O ELF unico do alvo escolhido: a depuracao probe-rs usa o mesmo alvo.
    readonly property string targetArtifact: {
        const item = targets.find(entry => entry.name === target);
        return item && (item.artifacts || []).length === 1 ? item.artifacts[0] : "";
    }

    signal proposalRequested(string device, string engine, double flashSizeBytes, string firmware, bool nativeMode, string buildDirectory, string target, string probe, var dfu)
    signal targetsRequested()
    signal runRequested(string command, var flash)
    signal saveRequested(string name, string command, var flash)

    visible: false

    function clear() {
        dfuSelection.clear();
        busy = false;
        engine = "";
        firmware = "";
        nativeMode = true;
        buildDirectory = "";
        target = "";
        probe = "";
        targets = [];
        targetsOrigin = "";
        targetsError = "";
        targetsBusy = false;
        proposal = ({});
        errorText = "";
        contextRevision += 1;
        requestedRevision = -1;
        proposalRequestId = -1;
    }

    // Uma prévia só vale para a porta, firmware e modelo que a produziram.
    // Mantenha busy até receber a resposta em voo, para não cruzar pedidos.
    function invalidate() {
        contextRevision += 1;
        proposal = ({});
        errorText = "";
    }

    function disconnected() {
        dfuSelection.forgetDestination();
        busy = false;
        targetsBusy = false;
        invalidate();
        requestedRevision = -1;
        proposalRequestId = -1;
    }

    // Escolher o firmware e' toggle; trocar descarta a previa. O motor
    // e' o da pagina do firmware (o core recusa outro), por isso solta.
    function selectFirmware(id) {
        firmware = id === firmware ? "" : id;
        engine = "";
        invalidate();
    }

    // O wrapper de cada framework, na ordem do modelo, sem repetir — a
    // palavra e' a do core (`runConfig.flashProposal { engine }`).
    function engineOfFrameworks(lista) {
        const mapa = { espIdf: "idf.py", zephyr: "west", platformIo: "platformio" };
        const saida = [];
        if (lista === undefined || lista === null) return saida;
        for (let i = 0; i < lista.length; i++) {
            const motor = mapa[lista[i].framework];
            if (motor !== undefined && saida.indexOf(motor) < 0) saida.push(motor);
        }
        return saida;
    }

    // Escolher o motor e' toggle; trocar descarta a previa (era de outro motor).
    function selectEngine(nome) {
        engine = nome === engine ? "" : nome;
        invalidate();
    }

    function selectMode(value) {
        if (nativeMode === value) return;
        nativeMode = value;
        invalidate();
    }

    function selectBuildDirectory(value) {
        if (buildDirectory === value) return;
        buildDirectory = value;
        invalidate();
    }

    function selectTarget(name) {
        if (target === name) return;
        target = name;
        invalidate();
    }

    function selectProbe(selector) {
        if (probe === selector) return;
        probe = selector;
        invalidate();
    }

    // O seletor que o probe-rs 0.32 aceita, como a listagem escreve a entrada.
    function selectorOf(entry) {
        const iface = entry.interface === undefined || entry.interface === null ? "" : "-" + entry.interface;
        return entry.vid + ":" + entry.pid + iface + ":" + (entry.serial || "");
    }

    // Sem serial a entrada nao identifica o destino; inacessivel precisa de permissao.
    function probeSelectable(entry) {
        return entry !== undefined && entry !== null && !!entry.serial && entry.inaccessible !== true;
    }

    function requestTargets() {
        if (targetsBusy) return;
        if (dfuDestination && dfu.binaryMode) invalidate();
        targetsBusy = true;
        targetsError = "";
        targetsRequested();
    }

    // So' a resposta que este controller pediu, so' executaveis do file-api e
    // da raiz atual: a lista nao tem id e a Biblioteca tambem a pede.
    function handleTargets(list, origin) {
        if (!targetsBusy) return;
        targetsBusy = false;
        targetsOrigin = origin === undefined ? "" : origin;
        const prefix = workspaceRoot.endsWith("/") ? workspaceRoot : workspaceRoot + "/";
        targets = origin !== "fileApi" || list === undefined || list === null ? []
            : list.filter(item => item.kind === "executable" && (item.sourceDir === workspaceRoot
                || (item.sourceDir || "").startsWith(prefix)));
        if (target !== "" && !targets.some(item => item.name === target)) selectTarget("");
    }

    function handleTargetsFailed(message) {
        if (!targetsBusy) return;
        targetsBusy = false;
        targetsError = message;
    }

    // Pede a previa. `device` e' a porta escolhida (vazia = o core decide se
    // o motor a exige); `flashSizeBytes` e' o que a identidade leu (0 = nao
    // comparar). Com destino na sonda, nem porta nem tamanho nem diretorio:
    // vao alvo e sonda. Um pedido por vez.
    function propose(device, flashSizeBytes) {
        if (busy) {
            return;
        }
        busy = true;
        requestedRevision = contextRevision;
        proposalRequestId = -1;
        proposal = ({});
        errorText = "";
        const toProbe = probeDestination;
        const toUsb = dfuDestination;
        proposalRequested(toProbe || toUsb || device === undefined ? "" : device, engine,
                          toProbe || toUsb || flashSizeBytes === undefined ? 0 : flashSizeBytes, firmware,
                          nativeMode, nativeMode && !toProbe && !toUsb ? buildDirectory : "",
                          toProbe ? target : "", toProbe ? probe : "", toUsb ? dfu.payload(target) : ({}));
    }

    function bindProposal(requestId) {
        if (!busy) return;
        proposalRequestId = requestId;
        if (requestId < 0) handleProposalFailed(requestId,
            qsTr("O serviço da IDE está indisponível. Tente novamente após reconectar."));
    }

    function handleProposal(nova, requestId) {
        if (!busy || requestId !== proposalRequestId) return;
        busy = false;
        proposalRequestId = -1;
        if (requestedRevision !== contextRevision) return;
        proposal = nova === undefined || nova === null ? ({}) : nova;
        const intent = proposal.flash;
        const hasIntent = intent !== undefined && intent !== null && typeof intent === "object"
                          && !Array.isArray(intent) && Object.keys(intent).length > 0;
        if (hasIntent !== nativeMode) {
            proposal = ({});
            errorText = qsTr("A resposta não corresponde ao modo solicitado. Gere outra prévia; nenhuma gravação foi iniciada.");
            return;
        }
        errorText = "";
    }

    // A recusa do core (sem build, sem porta, sem motor, sem ferramenta) e'
    // a resposta, no lugar da previa.
    function handleProposalFailed(requestId, message) {
        if (!busy || requestId !== proposalRequestId) return;
        busy = false;
        proposalRequestId = -1;
        if (requestedRevision !== contextRevision) return;
        proposal = ({});
        errorText = message;
    }

    function run() {
        if (found) {
            runRequested(nativeMode ? "" : proposal.command, nativeMode ? proposal.flash : ({}));
        }
    }

    function save() {
        if (found) {
            saveRequested(proposal.name !== undefined ? proposal.name : "Gravar", proposal.command,
                          nativeMode ? proposal.flash : ({}));
        }
    }

    // Listas que a tela mostra sem "undefined".
    function sourceLines() {
        return proposal.source !== undefined ? proposal.source : [];
    }

    function warningLines() {
        return proposal.warnings !== undefined ? proposal.warnings : [];
    }
}
