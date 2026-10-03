import QtQuick
import KineinVectis

// Limites dos paineis e layout versionado (0.3.6, roadmap 53 §4.4; regra
// decidida pelo autor em 2026-10-01). O que se prova:
//   - o exibido e' o preferido entre o minimo DECLARADO pelo conteudo e o
//     maximo que deixa o editor com 480 px; se os dois brigam, o minimo vence;
//   - a janela menor so' limita o exibido: o preferido fica intacto e volta
//     quando a janela cresce;
//   - arrastar para nos limites de agora;
//   - o layout vai e volta (retrato schema 1), o eco do que se gravou nao e'
//     reaplicado, e trocar de workspace cancela a gravacao pendente.
Item {
    id: root

    property var saves: []

    ShellController {
        id: shell

        onLayoutSaveRequested: function(scope, values) {
            root.saves.push({ scope: scope, values: values });
        }
    }

    // O SettingsController minimo que o applySettings le.
    QtObject {
        id: settings

        property bool railExpanded: false
        property var layout: null
        property int explorerWidth: 280
        property int bottomPanelHeight: 260
        property int outlineWidth: 220
        property bool outlineCollapsed: false
        property bool legacy: false

        function hasPersistedLayout() { return legacy; }
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    Component.onCompleted: {
        let failures = 0;
        const gap = Theme.panelGap;

        // 1366 de largura, trilho de 48: teto = 1366 - 48 - 2*gap - 480.
        shell.updateViewport(1366, 768);
        shell.updatePanelLimits(48, 220);
        const ceiling = 1366 - 48 - 2 * gap - 480;
        failures += check(shell.leftMaximumWidth === ceiling, "teto " + shell.leftMaximumWidth);

        shell.explorerPreferredWidth = 300;
        failures += check(shell.explorerWidth === 300, "dentro dos limites, exibe o preferido");

        // O conteudo declara um minimo maior (o rodape do Git): o exibido sobe.
        shell.updatePanelLimits(48, 330);
        failures += check(shell.explorerWidth === 330, "minimo declarado " + shell.explorerWidth);
        failures += check(shell.explorerPreferredWidth === 300, "o minimo nao reescreve o preferido");
        shell.updatePanelLimits(48, 220);

        // Preferido grande, janela pequena: limita o exibido, guarda o preferido.
        shell.explorerPreferredWidth = 600;
        shell.updateViewport(1024, 700);
        const smallCeiling = 1024 - 48 - 2 * gap - 480;
        failures += check(shell.explorerWidth === smallCeiling, "limitado pela janela " + shell.explorerWidth);
        failures += check(shell.explorerPreferredWidth === 600, "preferido intacto");
        shell.updateViewport(1920, 1080);
        failures += check(shell.explorerWidth === 600, "maximizar devolve o escolhido");

        // Janela estreita demais: o minimo vence o teto.
        shell.updateViewport(800, 600);
        shell.updatePanelLimits(48, 330);
        failures += check(shell.explorerWidth === 330, "minimo vence o teto " + shell.explorerWidth);
        shell.updatePanelLimits(48, 220);

        // Arrastar para nos limites de agora; o preferido vira o que se ve.
        shell.updateViewport(1366, 768);
        shell.explorerPreferredWidth = 300;
        shell.resizeExplorer(10000);
        failures += check(shell.explorerWidth === ceiling && shell.explorerPreferredWidth === ceiling,
                          "arrastar alem do teto " + shell.explorerPreferredWidth);
        shell.resizeExplorer(-10000);
        failures += check(shell.explorerPreferredWidth === 220, "arrastar abaixo do minimo");
        shell.resizeBottomPanel(10000);
        failures += check(shell.bottomPanelHeight === 768 - gap - 160, "altura do painel de baixo");

        // Layout schema 1: aplicar, e o retrato devolve o mesmo.
        const layout = {
            schemaVersion: 1, leftWindow: "git", leftVisible: true,
            sizes: { explorer: 340, outline: 240, bottom: 300 },
            outlineCollapsed: false, bottom: { visible: true, tab: "problems", pinned: ["build"] },
            rail: { pinned: ["database"], unpinned: [], hidden: ["tools"], sides: { outline: "left" } },
            // A ordem arrastada de cada barra (0.3.9) vai e volta igual.
            order: { bottom: ["tools", "terminal"], rail: ["git"] }
        };
        settings.layout = layout;
        shell.applySettings(settings);
        failures += check(shell.leftWindow === "git" && shell.explorerPreferredWidth === 340
                          && shell.bottomTab === "problems" && shell.showBottomPanel,
                          "layout aplicado");
        failures += check(JSON.stringify(shell.layoutSnapshot()) === JSON.stringify(layout),
                          "retrato " + JSON.stringify(shell.layoutSnapshot()));
        failures += check(root.saves.length === 0, "aplicar nao grava");
        failures += check(shell.bottomPinned.join() === "build", "abas fixadas aplicadas");
        failures += check(shell.railState.pinned[0] === "database" && shell.railState.hidden[0] === "tools",
                          "trilho aplicado " + JSON.stringify(shell.railState));

        // O eco do que se gravou nao puxa o painel de volta.
        shell.explorerPreferredWidth = 360;
        shell.savedLayoutText = JSON.stringify(layout);
        shell.applySettings(settings);
        failures += check(shell.explorerPreferredWidth === 360, "eco reaplicado");

        // Campo estranho fica no que estava; aba "git" nao e' aba de baixo.
        settings.layout = { schemaVersion: 1, sizes: { explorer: "x" }, bottom: { tab: "git" } };
        shell.applySettings(settings);
        failures += check(shell.explorerPreferredWidth === 360 && shell.bottomTab === "problems",
                          "campo estranho mudou algo");

        // Sem layout, os campos antigos valem (um ciclo de migracao).
        settings.layout = null;
        settings.legacy = true;
        settings.explorerWidth = 250;
        shell.applySettings(settings);
        failures += check(shell.explorerPreferredWidth === 250, "fallback dos campos antigos");

        // Escopo: com workspace grava no workspace; a troca cancela o pendente.
        shell.workspaceRoot = "/tmp/a";
        shell.applySettings(settings);
        shell.persistLayoutSoon();
        shell.workspaceRoot = "/tmp/b";
        failures += check(!shell.layoutLoaded, "a troca nao bloqueou a gravacao");
        shell.toggleRail();
        failures += check(root.saves.length === 1 && root.saves[0].scope === "global"
                          && root.saves[0].values.railExpanded === true, "trilho vai ao global");

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
