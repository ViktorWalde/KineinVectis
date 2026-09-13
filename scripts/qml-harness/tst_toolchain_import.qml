// O gerenciador de toolchain que LE o disco, na UI (42 §8 itens b e d,
// 2026-09-13): o arquivo de toolchain do kit chega por sinal proprio e e'
// PRESERVADO quando o painel aplica so' chip/alvo/sysroot; a proposta de kit
// lida do SDK vira linhas legiveis e so' vai ao core no "Aplicar proposta",
// num setKit so'; a recusa do importKit e' informacao de produto, nao ruido;
// trocar de workspace esquece tudo.
//
// Por que existe: um applyKit com o toolchainFile como "" LIMPARIA o arquivo
// que o autor importou toda vez que ele ajustasse o chip — e nenhum
// compilador reclama de um argumento posicional a menos.
import QtQuick
import "../../ui/qml/toolchain"

Item {
    id: root

    property var kits: []
    property var inspecoes: []
    property var importacoes: []

    ToolchainController {
        id: tc

        onSetKitRequested: function(preset, sysroot, targetTriple, chip, toolchainFile) {
            root.kits.push({ preset, sysroot, targetTriple, chip, toolchainFile });
        }
        onInspectSysrootRequested: function(path) { root.inspecoes.push(path); }
        onImportKitRequested: function(path) { root.importacoes.push(path); }
    }

    Component.onCompleted: {
        let failures = 0;
        tc.workspaceRoot = "/tmp/proj";

        // O arquivo do kit chega por sinal proprio; ausente = vazio.
        tc.handleKitFile("/opt/br/host/share/buildroot/toolchainfile.cmake");
        if (tc.toolchainFile !== "/opt/br/host/share/buildroot/toolchainfile.cmake") failures += 1;
        // O painel aplica chip/alvo/sysroot SEM falar do arquivo: preservado.
        tc.applyKit("/sr", "aarch64-linux-gnu", "");
        if (root.kits.length !== 1 || root.kits[0].toolchainFile !== "/opt/br/host/share/buildroot/toolchainfile.cmake") failures += 2;
        // Passar "" LIMPA, como nos outros campos.
        tc.applyKit("/sr", "aarch64-linux-gnu", "", "");
        if (root.kits[1].toolchainFile !== "") failures += 4;
        tc.handleKitFile(undefined);
        if (tc.toolchainFile !== "") failures += 8;

        // Ler sysroot / importar: aparado, vazio nao pede.
        tc.inspectSysroot("  /sysroots/pi  ");
        tc.inspectSysroot("   ");
        if (root.inspecoes.length !== 1 || root.inspecoes[0] !== "/sysroots/pi") failures += 16;
        tc.handleSysrootReport({ path: "/sysroots/pi", exists: true, verdict: "utilizavel: headers, bibliotecas (glibc 2.36) e 2 .pc para o pkg-config" });
        if (tc.sysrootSummary().indexOf("utilizavel") !== 0) failures += 32;

        tc.importKit("/opt/br/output");
        if (root.importacoes.length !== 1 || tc.hasKitProposal) failures += 64;
        tc.handleKitProposal({ kind: "buildroot", path: "/opt/br/output/host",
                               cCompiler: "/opt/br/output/host/bin/aarch64-buildroot-linux-gnu-gcc",
                               sysroot: "/opt/br/output/host/aarch64-buildroot-linux-gnu/sysroot",
                               targetTriple: "aarch64-buildroot-linux-gnu",
                               toolchainFile: "/opt/br/output/host/share/buildroot/toolchainfile.cmake",
                               hint: "o toolchainfile.cmake do Buildroot ja' fixa compiladores e sysroot" });
        if (!tc.hasKitProposal) failures += 128;
        const linhas = tc.proposalLines();
        if (linhas.length !== 6 || linhas[0].indexOf("buildroot") !== 0 || linhas[1].indexOf("CC:") !== 0) failures += 256;

        // Aplicar a proposta: UM setKit com sysroot, alvo e arquivo; o chip fica.
        tc.handleResolved([], [], "", "", "", "STM32F401CC", "");
        tc.applyProposal();
        const k = root.kits[root.kits.length - 1];
        if (root.kits.length !== 3 || k.sysroot !== "/opt/br/output/host/aarch64-buildroot-linux-gnu/sysroot"
                || k.targetTriple !== "aarch64-buildroot-linux-gnu"
                || k.toolchainFile !== "/opt/br/output/host/share/buildroot/toolchainfile.cmake"
                || k.chip !== "STM32F401CC") failures += 512;

        // A recusa do importKit e' o erro DESTE painel; a de outro metodo nao.
        tc.handleFailed("toolchain.importKit", "/x nao e' um SDK que eu reconheca");
        if (tc.hasKitProposal || tc.importError.indexOf("nao e' um SDK") < 0) failures += 1024;
        tc.handleFailed("git.status", "nada a ver");
        if (tc.importError.indexOf("nao e' um SDK") < 0) failures += 2048;

        // Sem proposta, aplicar nao pede nada.
        tc.applyProposal();
        if (root.kits.length !== 3) failures += 4096;

        // Trocar de workspace esquece tudo.
        tc.handleKitProposal({ kind: "yocto", path: "/opt/poky/environment-setup-x" });
        tc.workspaceRoot = "/tmp/outro";
        if (tc.hasKitProposal || tc.toolchainFile !== "" || tc.sysrootSummary() !== "" || tc.importError !== "") failures += 8192;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
