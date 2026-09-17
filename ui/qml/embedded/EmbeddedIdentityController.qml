pragma ComponentBehavior: Bound
import QtQuick

// A identidade PELO CANAL (serial.identify, E5 do integracoes/38 §6,
// 2026-09-17): o que o esptool leu da placa atras de uma porta e o kit que
// isso SUGERE. Dono proprio porque o EmbeddedController estava em 325/400
// linhas e a identidade e' outra responsabilidade que listar portas — a
// mesma regra que fez o EmbeddedSerialView nascer. Filho do
// EmbeddedController (`embeddedController.identity`): nenhuma propriedade de
// repasse nova em AppDomains/Shell/Host.
//
// Duas decisoes de produto moram aqui:
//   1. Identificar e' GESTO EXPLICITO. O esptool abre a porta e puxa DTR/RTS:
//      a placa reseta. Nada aqui roda ao abrir o painel.
//   2. A sugestao NAO vira kit sozinha. `applyToKit` emite o pedido; quem
//      grava e' o ToolchainController, e so' o chip — alvo/sysroot/preset
//      ficam como o usuario deixou.
//
// Nao fala com o CoreClient direto: pede por sinal e recebe do roteador.
Item {
    id: root

    // A porta perguntada; vazio = nunca perguntou (ou workspace trocou).
    property string device: ""
    property bool busy: false
    property string jobId: ""
    property string command: ""
    property var identity: ({})
    property var target: ({})
    property string errorText: ""
    property string rawOutput: ""
    readonly property bool found: identity.chip !== undefined

    signal identifyRequested(string device)
    signal kitChipRequested(string chip)

    visible: false

    function clear() {
        device = "";
        busy = false;
        jobId = "";
        command = "";
        identity = ({});
        target = ({});
        errorText = "";
        rawOutput = "";
    }

    // Um pedido por vez: o esptool prende a porta, e dois de uma vez
    // brigariam por ela.
    function identify(porta) {
        if (busy || porta === "") {
            return;
        }
        clear();
        device = porta;
        busy = true;
        identifyRequested(porta);
    }

    function handleStarted(newJobId, newCommand) {
        jobId = newJobId;
        command = newCommand;
    }

    // O desfecho do job. Um desfecho de OUTRA porta (pedido antigo, ja'
    // trocado) nao sobrescreve o atual.
    function handleIdentified(outcome) {
        if (outcome.device !== device) {
            return;
        }
        busy = false;
        if (outcome.command !== undefined && outcome.command !== "") {
            command = outcome.command;
        }
        identity = outcome.identity !== undefined && outcome.identity !== null ? outcome.identity : ({});
        target = outcome.target !== undefined && outcome.target !== null ? outcome.target : ({});
        errorText = outcome.success ? ""
                    : (outcome.error !== undefined && outcome.error !== "" ? outcome.error : qsTr("a identificação falhou"));
        rawOutput = outcome.raw !== undefined ? outcome.raw : "";
    }

    // A recusa ANTES de abrir a porta (sem esptool, sem permissao): e' a
    // resposta, no lugar do resultado.
    function handleFailed(method, message) {
        if (method !== "serial.identify") {
            return;
        }
        busy = false;
        errorText = message;
    }

    // Uma linha: chip, flash e MAC — o que se confere contra a etiqueta.
    // Campo ausente nao vira "undefined".
    function summary(id) {
        const partes = [];
        if (id.chipDescription !== undefined && id.chipDescription !== "") partes.push(id.chipDescription);
        else if (id.chip !== undefined) partes.push(id.chip);
        if (id.flashSize !== undefined) partes.push(qsTr("flash %1").arg(id.flashSize));
        if (id.mac !== undefined) partes.push("MAC " + id.mac);
        return partes.join(" · ");
    }

    // O kit sugerido em uma linha: chip e os motores, como o targetSummary
    // do EmbeddedController (mesma forma, outro dono).
    function targetSummary(alvo) {
        const partes = [];
        if (alvo.chip !== undefined) partes.push(qsTr("chip %1").arg(alvo.chip));
        if (alvo.family !== undefined) partes.push(alvo.family);
        const motores = [];
        if (alvo.flashEngine !== undefined) motores.push(qsTr("gravar: %1").arg(alvo.flashEngine));
        if (alvo.monitor !== undefined) motores.push(qsTr("monitor: %1").arg(alvo.monitor));
        if (alvo.debugAdapter !== undefined) motores.push(qsTr("debug: %1").arg(alvo.debugAdapter));
        if (motores.length > 0) partes.push(motores.join(", "));
        return partes.join(" · ");
    }

    function applyToKit() {
        if (target.chip === undefined || target.chip === "") {
            return;
        }
        kitChipRequested(target.chip);
    }
}
