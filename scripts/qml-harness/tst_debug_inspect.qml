import QtQuick
import "../../ui/qml/debug"

// O que o depurador mostra (P3, 2026-09-17): o DebugInspectController REAL
// como filho do DebugController real, com o roteador falso.
//
// O que se prova: os escopos so' sao pedidos com frame e uma vez por vez;
// trocar de frame esquece escopos e variaveis; o desfecho de outro frame e'
// ignorado; escolher um escopo pede as variaveis pelo ref dele e so' as
// desse ref entram; a memoria vira linhas hex+ascii com o endereco certo e
// os bytes ilegiveis ditos; o disassembly vira uma linha por instrucao sem
// "undefined"; a recusa do core vira erro; o fim da sessao limpa tudo.
Item {
    id: root

    property var pedidos: []

    DebugController {
        id: controller
    }

    Connections {
        target: controller.inspect

        function onScopesRequested(frameId) { root.pedidos.push("scopes " + frameId); }
        function onScopeVariablesRequested(ref) { root.pedidos.push("vars " + ref); }
        function onReadMemoryRequested(memoryReference, count, offset) { root.pedidos.push("mem " + memoryReference + " " + count + " " + offset); }
        function onDisassembleRequested(memoryReference, instructionCount, instructionOffset) { root.pedidos.push("dis " + memoryReference + " " + instructionCount + " " + instructionOffset); }
    }

    Component.onCompleted: {
        let failures = 0;
        const i = controller.inspect;

        // Sem frame: nao pede.
        i.requestScopes();
        if (root.pedidos.length !== 0 || i.frameId !== -1) failures += 1;

        // Um frame parado: os escopos sao pedidos uma vez por vez.
        controller.currentFrameId = 7;
        i.requestScopes();
        i.requestScopes();
        if (root.pedidos.join("|") !== "scopes 7" || !i.scopesBusy) failures += 2;
        // Desfecho de OUTRO frame (pedido antigo) e' ignorado.
        i.handleScopes(3, [{ name: "Locals", ref: 1, expensive: false }]);
        if (i.scopes.length !== 0 || !i.scopesBusy) failures += 4;
        i.handleScopes(7, [{ name: "Locals", ref: 11, expensive: false }, { name: "Registers", ref: 12, expensive: false }, { name: "Peripherals", ref: 13, expensive: true }]);
        if (i.scopesBusy || i.scopes.length !== 3) failures += 8;

        // Escolher um escopo pede as variaveis pelo ref; so' as dele entram.
        i.selectScope("Peripherals");
        if (root.pedidos[1] !== "vars 13" || i.selectedRef !== 13 || !i.variablesBusy) failures += 16;
        i.selectScope("nao existe");
        if (i.selectedScope !== "Peripherals" || root.pedidos.length !== 2) failures += 32;
        i.handleVariables(12, [{ name: "r0", value: "1", ref: 0 }]);
        if (i.scopeVariables.length !== 0) failures += 64;
        i.handleVariables(13, [{ name: "GPIO", value: "{…}", ref: 40 }, { name: "UART0", value: "{…}", ref: 41 }]);
        if (i.variablesBusy || i.scopeVariables.length !== 2 || i.scopeVariables[1].name !== "UART0") failures += 128;

        // Trocar de frame esquece escopos e variaveis.
        controller.currentFrameId = 8;
        if (i.scopes.length !== 0 || i.selectedScope !== "" || i.scopeVariables.length !== 0) failures += 256;

        // Memoria: o endereco aparado, a contagem padrao, e o hex por linha.
        i.readMemory("  0x3ff00000 ");
        if (root.pedidos[2] !== "mem 0x3ff00000 64 0" || !i.memoryBusy || i.memoryReference !== "0x3ff00000") failures += 512;
        i.readMemory("0x1");
        if (root.pedidos.length !== 3) failures += 1024;
        // 20 bytes: "ABCDEFGHIJKLMNOPQRST" -> duas linhas, 16 + 4.
        i.handleMemory({ address: "0x3ff00000", data: Qt.btoa("ABCDEFGHIJKLMNOPQRST"), unreadableBytes: 2 });
        if (i.memoryBusy || i.memoryLines.length !== 3) failures += 2048;
        if (i.memoryLines[0] !== "0x3ff00000  41 42 43 44 45 46 47 48 49 4a 4b 4c 4d 4e 4f 50  ABCDEFGHIJKLMNOP") failures += 4096;
        if (i.memoryLines[1] !== "0x3ff00010  51 52 53 54  QRST") failures += 8192;
        if (i.memoryLines[2].indexOf("2 bytes") < 0) failures += 16384;
        // Bytes fora do ASCII viram ".".
        i.handleMemory({ address: "0x10", data: Qt.btoa(String.fromCharCode(1) + String.fromCharCode(255)) });
        if (i.memoryLines[0] !== "0x10  01 ff  ..") failures += 32768;

        // Disassembly: uma linha por instrucao, sem "undefined".
        i.disassemble("0x40080000", 2);
        if (root.pedidos[3] !== "dis 0x40080000 2 0" || !i.disassemblyBusy) failures += 65536;
        i.handleDisassembly([{ address: "0x40080000", instruction: "entry a1, 32", instructionBytes: "36 41 00", symbol: "main" }, { address: "0x40080003", instruction: "movi a2, 1" }]);
        if (i.disassemblyBusy || i.instructions.length !== 2) failures += 131072;
        if (i.instructionLine(i.instructions[0]) !== "0x40080000  36 41 00  entry a1, 32   <main>") failures += 262144;
        if (i.instructionLine(i.instructions[1]) !== "0x40080003  movi a2, 1") failures += 524288;

        // A recusa do core: erro, e a de outro metodo nao mexe.
        i.readMemory("0x0");
        i.handleFailed("debug.evaluate", "outro");
        if (!i.memoryBusy) failures += 1048576;
        i.handleFailed("debug.readMemory", "sem sessao");
        if (i.memoryBusy || i.errorText !== "sem sessao") failures += 2097152;

        // O fim da sessao limpa tudo (o pai limpa o filho).
        controller.handleFinished(0);
        if (i.frameId !== -1 || i.memoryLines.length !== 0 || i.instructions.length !== 0 || i.errorText !== "" || i.memoryReference !== "") failures += 4194304;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
