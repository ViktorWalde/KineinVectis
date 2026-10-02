import QtQuick
import KineinVectis

// O cabecalho na ordem do usuario (0.3.9): os widgets fixos se reordenam
// pela ordem salva, o "⋯" fica sempre no fim, e reordenar nao esconde chip.
Item {
    id: root

    width: 1400
    height: 60

    TopHeaderBar {
        id: bar

        width: 1400
        workspaceOpen: true
        workspaceName: "hibrido"
        gitBranchLabel: "main"
        toolchainSummary: "Clang++ · Ninja"
        pythonSummary: "sistema · 3.14.4 ⚠"
        // Como no app: a ordem chega por binding ja' na criacao.
        order: root.initialOrder
    }

    property var initialOrder: []

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    // A ordem desenhada: os widgets visiveis do grupo, pelo x de destino
    // (o x de verdade anima em motionFast).
    function keysInRow() {
        const group = bar.children.find(child => child.spacing !== undefined && child.children.length >= 5);
        return group.children.filter(child => child.visible && child.reorderKey !== undefined
                                     && child.reorderKey !== "")
            .sort((a, b) => bar.xOf(a.reorderKey) - bar.xOf(b.reorderKey)).map(child => child.reorderKey);
    }

    Component.onCompleted: {
        let failures = 0;
        failures += check(keysInRow().join(",") === "project,git,toolchain,python", "fabrica: " + keysInRow());
        root.initialOrder = [];
        failures += check(keysInRow().join(",") === "project,git,toolchain,python", "ordem vazia de novo: " + keysInRow());
        bar.order = ["python", "project"];
        failures += check(keysInRow().join(",") === "python,project,git,toolchain", "ordem salva: " + keysInRow());
        failures += check(!bar.pythonHidden && !bar.toolchainHidden, "nenhum chip escondido");
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
