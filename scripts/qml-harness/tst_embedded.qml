// Embarcados: o que a UI mostra da sonda e o que ela pede (roadmaps/35 §5.7,
// fatia 1 — o fio).
//
// Por que existe: o `probe.list` foi roteado no core em 2026-09-03 e ate'
// 2026-09-11 nenhuma tela o pedia. O `EmbeddedController` e' o consumidor que
// faltava, e a traducao dele — sonda reconhecida, ferramenta ausente, dica,
// saida crua — e' o que separa "nenhuma sonda" (verdade) de "nenhuma sonda"
// com uma plugada (mentira). Sem harness, essa traducao quebra sem compilador
// que reclame — a classe do `cdbStale`.
import QtQuick
import "../../ui/qml/embedded"

Item {
    id: root

    property int consultas: 0

    EmbeddedController {
        id: controller

        onListRequested: root.consultas += 1
    }

    Component.onCompleted: {
        let failures = 0;

        // Abrir PERGUNTA: sonda e' coisa que se pluga e despluga.
        controller.workspaceRoot = "/tmp/projeto";
        controller.open();
        if (!controller.panelVisible) failures += 1;
        if (root.consultas !== 1) failures += 2;
        if (!controller.busy) failures += 4;

        // Uma sonda reconhecida: a linha diz nome, familia, VID:PID e serial.
        controller.handleProbes(
            [{ name: "STLink V3", vid: "0483", pid: "374e", serial: "0031",
               kind: "ST-LINK" }],
            true, "irrelevante", "");
        if (!controller.probeFound) failures += 8;
        if (controller.busy) failures += 16;
        if (controller.probeSummary(controller.probes[0])
                !== "STLink V3 · ST-LINK · 0483:374e · 0031") failures += 32;

        // Campo ausente NAO vira "undefined" na tela.
        if (controller.probeSummary({ name: "CMSIS-DAP", vid: "0d28", pid: "0204" })
                !== "CMSIS-DAP · 0d28:0204") failures += 64;

        // Nada reconhecido: a dica e a saida crua ficam, e' o que o usuario le.
        controller.refresh();
        if (root.consultas !== 2) failures += 128;
        controller.handleProbes([], true, "No debug probes were found.",
                                "nenhuma sonda conectada.");
        if (controller.probeFound) failures += 256;
        if (controller.hint !== "nenhuma sonda conectada.") failures += 512;
        if (controller.rawOutput !== "No debug probes were found.") failures += 1024;

        // Ferramenta ausente e' outro estado, nao "nenhuma sonda".
        controller.handleProbes([], false, "", "instale o probe-rs");
        if (controller.toolAvailable) failures += 2048;

        // Erro de OUTRO dominio nao acende o erro deste painel.
        controller.refresh();
        controller.handleFailed("git.status", "nada a ver");
        if (controller.errorText !== "") failures += 4096;
        if (!controller.busy) failures += 8192;
        controller.handleFailed("probe.list", "probe-rs saiu com 1");
        if (controller.errorText !== "probe-rs saiu com 1") failures += 16384;
        if (controller.busy) failures += 32768;

        // Trocar de workspace FECHA e esquece: a lista da ultima vez e'
        // exatamente o que nao se pode mostrar.
        controller.handleProbes([{ name: "x", vid: "1", pid: "2" }], true, "", "");
        controller.workspaceRoot = "/tmp/outro";
        if (controller.panelVisible) failures += 65536;
        if (controller.probes.length !== 0) failures += 131072;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
