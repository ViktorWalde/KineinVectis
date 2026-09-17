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
    readonly property bool found: proposal.command !== undefined && proposal.command !== ""
    readonly property var engines: ["esptool", "probe-rs", "picotool", "dfu-util"]

    signal proposalRequested(string device, string engine, double flashSizeBytes)
    signal runRequested(string command)
    signal saveRequested(string name, string command)

    visible: false

    function clear() {
        busy = false;
        engine = "";
        proposal = ({});
        errorText = "";
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
                          flashSizeBytes === undefined ? 0 : flashSizeBytes);
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
