// O PROVEDOR DE INSTALACAO de toolchain na UI (integracoes/39 §5, 2026-09-13):
// o catalogo chega do core e a tela so' ordena (recomendadas primeiro);
// "Instalar" pede UMA vez, so' para o que nao esta' instalado, e nunca duas
// em paralelo (sao centenas de MB); o desfecho zera o "instalando" e pede o
// catalogo e os candidatos de novo — a toolchain nova ja' e' candidato.
//
// Por que existe: a guarda a menos e o duplo clique baixa 300 MB duas vezes;
// a ordem errada esconde a recomendada; e o "instalando" que nao volta trava
// o botao para sempre. Nenhum compilador acusa isso.
import QtQuick
import "../../ui/qml/toolchain"

Item {
    id: root

    property var pedidosInstalar: []
    property int pedidosCatalogo: 0
    property int consultasKit: 0

    ToolchainController {
        id: tc

        onInstallRequested: function(id) { root.pedidosInstalar.push(id); }
        onInstallableRequested: root.pedidosCatalogo += 1
        onGetRequested: function(preset) { root.consultasKit += 1; }
    }

    Component.onCompleted: {
        let failures = 0;

        // Abrir o workspace pede o kit E o catalogo.
        tc.workspaceRoot = "/tmp/proj";
        if (root.pedidosCatalogo !== 1 || root.consultasKit !== 1) failures += 1;

        // O catalogo como o core manda: nada recomendado primeiro na lista.
        tc.handleInstallable([
            { id: "bootlin-aarch64", label: "Bootlin aarch64", version: "2026.08-1", sizeBytes: 99347372, sha256: "0213", url: "https://x/a.tar.xz", license: "GPL", installed: false, recommended: false },
            { id: "arm-gnu", label: "Arm GNU", version: "15.2.rel1", sizeBytes: 155499480, sha256: "5978", url: "https://x/b.tar.xz", license: "GPL", installed: false, recommended: true },
            { id: "xpack", label: "xPack", version: "15.2.1-1.1", sizeBytes: 307090703, sha256: "da6a", url: "https://x/c.tar.gz", license: "MIT", installed: true, recommended: true }
        ], "/home/x/.local/share/kinein-vectis/toolchains", "stm32");
        if (tc.installRoot !== "/home/x/.local/share/kinein-vectis/toolchains" || tc.projectFamily !== "stm32") failures += 2;
        const ordem = tc.installableSorted().map(t => t.id);
        if (ordem.join(",") !== "arm-gnu,xpack,bootlin-aarch64") failures += 4;

        // Instalar: uma vez; a segunda espera; instalada nao pede; id estranho nao pede.
        tc.install("arm-gnu");
        if (root.pedidosInstalar.length !== 1 || root.pedidosInstalar[0] !== "arm-gnu" || tc.installing !== "arm-gnu") failures += 8;
        tc.install("bootlin-aarch64");
        if (root.pedidosInstalar.length !== 1) failures += 16;
        tc.handleInstalled({ id: "arm-gnu", version: "15.2.rel1", path: "/home/x/.local/share/kinein-vectis/toolchains/arm-gnu/15.2.rel1", success: true });
        if (tc.installing !== "" || tc.lastInstallOutcome.indexOf("instalada em") < 0) failures += 32;
        if (root.pedidosCatalogo !== 2 || root.consultasKit !== 2) failures += 64;
        tc.install("xpack");
        if (root.pedidosInstalar.length !== 1) failures += 128;
        tc.install("nao-existe");
        if (root.pedidosInstalar.length !== 1) failures += 256;

        // Falha: o motivo do core vira o desfecho, e o botao volta.
        tc.install("bootlin-aarch64");
        tc.handleInstalled({ id: "bootlin-aarch64", version: "2026.08-1", path: "/x", success: false, error: "o SHA-256 do arquivo baixado NAO e' o publicado" });
        if (tc.installing !== "" || tc.lastInstallOutcome.indexOf("falhou: o SHA-256") !== 0) failures += 512;

        // Sem workspace nao se instala; trocar de workspace esquece o catalogo.
        tc.workspaceRoot = "";
        if (tc.installable.length !== 0 || tc.installing !== "" || tc.lastInstallOutcome !== "") failures += 1024;
        tc.install("arm-gnu");
        if (root.pedidosInstalar.length !== 2) failures += 2048;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
