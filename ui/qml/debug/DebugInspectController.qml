pragma ComponentBehavior: Bound
import QtQuick

// O que o depurador de embarcado MOSTRA (P3 do 40 §4.1, 2026-09-17): os
// escopos inteiros de um frame (Locals, Registers, os perifericos do SVD no
// probe-rs), a memoria e o disassembly — passagem do DAP padrao, que o
// probe-rs 0.32 e o GDB 17 anunciam (readMemory/disassemble). Filho do
// DebugController (`debugController.inspect`), pela mesma razao dos filhos
// do painel de Embarcados: nenhuma propriedade de repasse nova.
//
// O frame vem do pai (`frameId`); trocar de frame esquece os escopos e as
// variaveis do escopo (sao daquele frame). Memoria e disassembly ficam ate'
// a proxima leitura ou o fim da sessao — o endereco e' do usuario.
Item {
    id: root

    // O frame selecionado no pai; -1 = nenhum.
    property real frameId: -1
    property var scopes: []
    property string selectedScope: ""
    property real selectedRef: 0
    property var scopeVariables: []
    property bool scopesBusy: false
    property bool variablesBusy: false
    property string errorText: ""

    // Memoria: o endereco digitado, o que voltou (hex por linha de 16 bytes).
    property string memoryReference: ""
    property int memoryCount: 64
    property string memoryAddress: ""
    property var memoryLines: []
    property bool memoryBusy: false
    // Disassembly: as instrucoes que voltaram.
    property var instructions: []
    property bool disassemblyBusy: false

    signal scopesRequested(real frameId)
    signal scopeVariablesRequested(real ref)
    signal readMemoryRequested(string memoryReference, real count, real offset)
    signal disassembleRequested(string memoryReference, real instructionCount, real instructionOffset)

    visible: false

    onFrameIdChanged: {
        scopes = [];
        selectedScope = "";
        selectedRef = 0;
        scopeVariables = [];
        scopesBusy = false;
        variablesBusy = false;
        errorText = "";
    }

    function clear() {
        frameId = -1;
        memoryReference = "";
        memoryAddress = "";
        memoryLines = [];
        memoryBusy = false;
        instructions = [];
        disassemblyBusy = false;
        errorText = "";
    }

    // Pedir os escopos do frame atual: gesto (o escopo Peripherals e' caro).
    function requestScopes() {
        if (frameId < 0 || scopesBusy) return;
        scopesBusy = true;
        errorText = "";
        scopesRequested(frameId);
    }

    // O desfecho de outro frame (pedido antigo) nao sobrescreve.
    function handleScopes(forFrame, lista) {
        if (forFrame !== frameId) return;
        scopesBusy = false;
        scopes = lista === undefined || lista === null ? [] : lista;
    }

    function selectScope(nome) {
        const escopo = scopes.find(s => s.name === nome);
        if (escopo === undefined) return;
        selectedScope = nome;
        selectedRef = escopo.ref;
        scopeVariables = [];
        variablesBusy = true;
        scopeVariablesRequested(escopo.ref);
    }

    // As variaveis chegam pelo mesmo debug.variables { ref } do pai; so' as
    // do escopo escolhido interessam aqui (o pai trata as expansoes dele).
    function handleVariables(ref, lista) {
        if (ref !== selectedRef || selectedRef === 0) return;
        variablesBusy = false;
        scopeVariables = lista === undefined || lista === null ? [] : lista;
    }

    function readMemory(referencia, count) {
        const ref = referencia === undefined ? memoryReference : String(referencia).trim();
        if (ref === "" || memoryBusy) return;
        memoryReference = ref;
        if (count !== undefined && count > 0) memoryCount = count;
        memoryBusy = true;
        errorText = "";
        readMemoryRequested(ref, memoryCount, 0);
    }

    // O corpo do DAP: `data` em base64 -> linhas "endereco  xx xx …".
    function handleMemory(memory) {
        memoryBusy = false;
        if (memory === undefined || memory === null) return;
        memoryAddress = memory.address !== undefined ? memory.address : "";
        memoryLines = hexLines(memoryAddress, memory.data !== undefined && memory.data !== null ? memory.data : "");
        if (memory.unreadableBytes !== undefined && memory.unreadableBytes > 0) {
            memoryLines = memoryLines.concat([qsTr("(%1 bytes ilegiveis no fim)").arg(memory.unreadableBytes)]);
        }
    }

    function hexLines(address, base64) {
        const bytes = Qt.atob(base64);
        const base = parseInt(address, 16);
        const linhas = [];
        for (let i = 0; i < bytes.length; i += 16) {
            const partes = [];
            let ascii = "";
            for (let j = i; j < Math.min(i + 16, bytes.length); j++) {
                const c = bytes.charCodeAt(j) & 0xff;
                partes.push((c < 16 ? "0" : "") + c.toString(16));
                ascii += c >= 32 && c < 127 ? bytes[j] : ".";
            }
            const rotulo = isNaN(base) ? String(i) : "0x" + Number(base + i).toString(16);
            linhas.push(rotulo + "  " + partes.join(" ") + "  " + ascii);
        }
        return linhas;
    }

    function disassemble(referencia, count) {
        const ref = referencia === undefined ? memoryReference : String(referencia).trim();
        if (ref === "" || disassemblyBusy) return;
        memoryReference = ref;
        disassemblyBusy = true;
        errorText = "";
        disassembleRequested(ref, count === undefined ? 32 : count, 0);
    }

    function handleDisassembly(lista) {
        disassemblyBusy = false;
        instructions = lista === undefined || lista === null ? [] : lista;
    }

    // Uma linha por instrucao, sem "undefined": endereco, bytes, texto, simbolo.
    function instructionLine(i) {
        let linha = i.address + "  ";
        if (i.instructionBytes !== undefined) linha += i.instructionBytes + "  ";
        linha += i.instruction;
        if (i.symbol !== undefined) linha += "   <" + i.symbol + ">";
        return linha;
    }

    // A recusa do core (sem sessao, parametro invalido) e' a resposta.
    function handleFailed(method, message) {
        if (method === "debug.scopes") { scopesBusy = false; errorText = message; }
        else if (method === "debug.readMemory") { memoryBusy = false; errorText = message; }
        else if (method === "debug.disassemble") { disassemblyBusy = false; errorText = message; }
    }
}
