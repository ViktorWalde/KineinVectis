import QtQuick
import KineinVectis

// O painel de baixo contextual (0.3.7 F2, roadmap 53 §5.5). O que se prova:
// sem atividade so' aparecem as de `alwaysVisible` e a ativa; cada fato traz a
// sua aba; fixada aparece sem fato; a ativa nunca some (o atalho que a abre
// tem de mostra-la); e o menu da aba alterna Fixar/Desafixar.
Item {
    id: root

    width: 1200
    height: 40

    BottomTabBar {
        id: bar

        width: root.width
        activeTab: "terminal"
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    function keys(facts, active, pinned) {
        return bar.visibleTabs(bar.allTabs, active, facts, pinned, bar.alwaysVisible)
            .map(function(t) { return t.key; }).join(",");
    }

    Component.onCompleted: {
        let failures = 0;

        failures += check(keys({}, "terminal", []) === "terminal,problems", "sem atividade");
        failures += check(keys({ build: true }, "terminal", []) === "terminal,build,problems",
                          "build com saida");
        failures += check(keys({ debug: true, search: true }, "terminal", [])
                          === "terminal,problems,debug,search", "debug e busca");
        failures += check(keys({}, "logs", []) === "terminal,problems,logs", "a ativa aparece");
        failures += check(keys({}, "terminal", ["jobs"]) === "terminal,problems,jobs", "fixada");
        failures += check(keys({ tests: false }, "terminal", []) === "terminal,problems",
                          "fato falso nao traz");

        // O componente desenha a projecao.
        bar.facts = { build: true };
        failures += check(bar.visibleTabs(bar.allTabs, bar.activeTab, bar.facts, bar.pinnedTabs,
                                          bar.alwaysVisible).length === 3, "componente");

        // Menu da aba: fixar quem nao esta' fixada; desafixar a fixada; as
        // sempre visiveis nao se fixam (ja' aparecem).
        failures += check(bar.tabMenuItems("build")[0].action === "bottom.pin:build", "menu fixar");
        bar.pinnedTabs = ["build"];
        failures += check(bar.tabMenuItems("build")[0].action === "bottom.unpin:build", "menu desafixar");
        failures += check(!bar.tabMenuItems("terminal")[0].enabled, "sempre visivel nao se fixa");

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
