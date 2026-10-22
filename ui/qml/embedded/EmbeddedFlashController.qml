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

    signal proposalRequested(string device, string engine, double flashSizeBytes, string firmware, bool nativeMode, string buildDirectory)
    signal runRequested(string command, var flash)
    signal saveRequested(string name, string command, var flash)

    visible: false

    function clear() {
        busy = false;
        engine = "";
        firmware = "";
        nativeMode = true;
        buildDirectory = "";
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
        busy = false;
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

    // Pede a previa. `device` e' a porta escolhida (vazia = o core decide se
    // o motor a exige); `flashSizeBytes` e' o que a identidade leu (0 = nao
    // comparar). Um pedido por vez.
    function propose(device, flashSizeBytes) {
        if (busy) {
            return;
        }
        busy = true;
        requestedRevision = contextRevision;
        proposalRequestId = -1;
        proposal = ({});
        errorText = "";
        proposalRequested(device === undefined ? "" : device, engine,
                          flashSizeBytes === undefined ? 0 : flashSizeBytes, firmware,
                          nativeMode, nativeMode ? buildDirectory : "");
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
