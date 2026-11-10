import QtQuick
import KineinVectis

// O cabecalho na ordem do usuario (0.3.9): os widgets fixos se reordenam
// pela ordem salva, o "⋯" fica sempre no fim, e reordenar nao esconde chip.
// O chip da configuracao do CMake (C1-23) entra antes da toolchain, ganha a
// largura antes dela e, sem lugar, vai para o "⋯" com o aviso.
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
        onConfigurationMenuRequested: (menuX, menuY) => root.menuPoint = { x: menuX, y: menuY }
    }

    property var initialOrder: []
    property var menuPoint: null

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    // A ordem desenhada: os widgets visiveis do grupo, pelo x de destino
    // (o x de verdade anima em motionFast).
    function group() {
        // `children` nao e' Array do JS no Qt 6.4 (o do AppImage): copia por indice.
        let found = null;
        for (let i = 0; i < bar.children.length; i++) {
            const child = bar.children[i];
            if (child.spacing !== undefined && child.children.length >= 5) found = child;
        }
        return found;
    }

    function chip(key) {
        const owner = root.group();
        for (let i = 0; i < owner.children.length; i++) {
            if (owner.children[i].reorderKey === key) return owner.children[i];
        }
        return null;
    }

    function keysInRow() {
        const group = root.group();
        const items = [];
        for (let i = 0; i < group.children.length; i++) items.push(group.children[i]);
        return items.filter(child => child.visible && child.reorderKey !== undefined
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

        // C1-23: o chip da configuracao, antes da toolchain na ordem de fabrica
        // e no fim de uma ordem salva antes dele.
        bar.configurationSummary = "release";
        failures += check(keysInRow().join(",") === "python,project,git,configuration,toolchain",
                          "ordem salva antiga: " + keysInRow());
        bar.order = [];
        failures += check(keysInRow().join(",") === "project,git,configuration,toolchain,python",
                          "fabrica com a configuracao: " + keysInRow());
        const configuration = root.chip("configuration");
        configuration.children.find(child => child.tapped !== undefined).tapped({ button: Qt.LeftButton });
        const expectedX = root.group().x + bar.xOf("configuration");
        failures += check(root.menuPoint !== null && Math.abs(root.menuPoint.x - expectedX) < 0.5
                          && root.menuPoint.y > root.group().y + configuration.height,
                          "o menu abre embaixo do chip, no espaco da barra: " + JSON.stringify(root.menuPoint)
                          + " x esperado " + expectedX);
        // A configuracao pega a largura antes da toolchain: encolhendo, a
        // toolchain sai primeiro, e sem lugar a configuracao vai para o ⋯.
        let configurationOnly = false;
        let inverted = false;
        for (let width = 1400; width >= 300; width -= 10) {
            bar.width = width;
            if (configuration.visible && bar.toolchainHidden) configurationOnly = true;
            if (!configuration.visible && !bar.toolchainHidden) inverted = true;
        }
        failures += check(configurationOnly, "ha' largura com a configuracao e sem a toolchain");
        failures += check(!inverted, "a toolchain nunca fica com a configuracao escondida");
        failures += check(bar.configurationHidden && bar.toolchainHidden, "estreita: os dois no ⋯");
        // Sem o ⚠ do Python, so' a configuracao decide o ponto do ⋯.
        bar.pythonSummary = "";
        const overflow = root.group().children.find(child => child.iconName === "more");
        const dot = overflow.children.find(child => child.color !== undefined && child.radius !== undefined
                                                    && child.width === 7);
        failures += check(overflow.visible && dot !== undefined && !dot.visible,
                          "a configuracao escondida sem aviso nao acende o ⋯");
        bar.configurationSummary = "⚠ sem configuração";
        failures += check(overflow.visible && dot !== undefined && dot.visible,
                          "a configuracao escondida com ⚠ acende o ⋯");
        // Sem toolchain nem Python, o ⋯ existe so' pela configuracao.
        const toolchain = bar.toolchainSummary;
        bar.toolchainSummary = "";
        failures += check(bar.configurationHidden && overflow.visible,
                          "a configuracao escondida sozinha abre o ⋯");
        bar.toolchainSummary = toolchain;
        bar.configurationSummary = "";
        bar.width = 1400;
        failures += check(!bar.configurationHidden && !configuration.visible,
                          "sem resumo: sem chip e sem item no ⋯");
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
