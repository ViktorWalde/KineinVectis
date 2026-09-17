pragma ComponentBehavior: Bound
import QtQuick

// "Gravar" como CONFIGURACAO DE EXECUCAO (E4 do integracoes/38 §6; decisao
// do autor em 2026-09-11: nao e' dominio novo). O core compoe a linha do
// motor (runConfig.flashProposal, PURO — nada roda, nada e' salvo) e este
// controller guarda a PREVIA: nome, comando, motor, evidencias e avisos.
//
// Dois gestos saem daqui, e nenhum grava por conta propria:
//   runRequested(command)          -> RuntimeController.startRun(command):
//                                     a linha roda AGORA na aba de execucao,
//                                     como qualquer comando digitado
//   saveRequested(name, command)   -> RunConfigController: vira a
//                                     configuracao ativa, e o botao Executar
//                                     passa a gravar ate' o usuario trocar
// A fiacao dos dois mora no AppDomains (donos diferentes). Filho do
// EmbeddedController (`embeddedController.flash`), pela mesma razao do
// `identity`: nenhuma propriedade de repasse nova.
Item {
    id: root

    property bool busy: false
    // O motor que o usuario ESCOLHEU; vazio = o que o modelo do projeto
    // sugere (esptool/probe-rs/picotool/dfu-util decidido pelo core).
    property string engine: ""
    property var proposal: ({})
    property string errorText: ""
    // O firmware BAIXADO do catalogo (C5) que a linha grava no lugar dos
    // artefatos do build; vazio = os artefatos do build. O id vem da lista
    // de instalaveis do ToolchainController (kind "firmware", installed).
    property string firmware: ""
    readonly property bool found: proposal.command !== undefined && proposal.command !== ""
    readonly property var engines: ["esptool", "probe-rs", "picotool", "dfu-util"]
    // Os frameworks do projeto (project.model.frameworks, do pai): o wrapper
    // de cada um entra como motor (bloco E do roadmaps/41): idf.py, west, pio.
    property var frameworks: []
    readonly property var frameworkEngines: engineOfFrameworks(frameworks)
    readonly property var allEngines: engines.concat(frameworkEngines)

    signal proposalRequested(string device, string engine, double flashSizeBytes, string firmware)
    signal runRequested(string command)
    signal saveRequested(string name, string command)

    visible: false

    function clear() {
        busy = false;
        engine = "";
        firmware = "";
        proposal = ({});
        errorText = "";
    }

    // Escolher o firmware e' toggle; trocar descarta a previa. O motor
    // e' o da pagina do firmware (o core recusa outro), por isso solta.
    function selectFirmware(id) {
        firmware = id === firmware ? "" : id;
        engine = "";
        proposal = ({});
        errorText = "";
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
        proposal = ({});
        errorText = "";
    }

    // Pede a previa. `device` e' a porta escolhida (vazia = o core decide se
    // o motor a exige); `flashSizeBytes` e' o que a identidade leu (0 = nao
    // comparar). Um pedido por vez.
    function propose(device, flashSizeBytes) {
        if (busy) {
            return;
        }
        busy = true;
        errorText = "";
        proposalRequested(device === undefined ? "" : device, engine,
                          flashSizeBytes === undefined ? 0 : flashSizeBytes, firmware);
    }

    function handleProposal(nova) {
        busy = false;
        proposal = nova === undefined || nova === null ? ({}) : nova;
        errorText = "";
    }

    // A recusa do core (sem build, sem porta, sem motor, sem ferramenta) e'
    // a resposta, no lugar da previa.
    function handleFailed(method, message) {
        if (method !== "runConfig.flashProposal") {
            return;
        }
        busy = false;
        proposal = ({});
        errorText = message;
    }

    function run() {
        if (found) {
            runRequested(proposal.command);
        }
    }

    function save() {
        if (found) {
            saveRequested(proposal.name !== undefined ? proposal.name : "Gravar", proposal.command);
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
