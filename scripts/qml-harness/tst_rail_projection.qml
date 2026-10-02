import QtQuick
import KineinVectis

// O trilho por areas (0.3.7 F1, roadmap 53 §4.3): a tabela verdade da
// projecao, os fatos com "desconhecido mantem o ultimo", e fixar/ocultar no
// ShellController. Sem janela: a projecao e' funcao pura.
Item {
    id: root

    RailProjection { id: projection }

    ShellController { id: shell }

    QtObject {
        id: remoteFake

        property var targets: undefined
    }

    RailFacts {
        id: facts

        remoteController: remoteFake
        workspaceOpen: true
    }

    readonly property var entries: [
        { id: "explorer", title: "Projeto", order: 10, defaultPolicy: "pinned", factKey: "",
          commandId: "view.project", shortcut: "", available: true },
        { id: "database", title: "Banco", order: 40, defaultPolicy: "contextual",
          factKey: "datasource.any", commandId: "datasource.list", shortcut: "Ctrl+Alt+J",
          available: true },
        { id: "remote", title: "Remoto", order: 55, defaultPolicy: "contextual",
          factKey: "remote.any", commandId: "remote.list", shortcut: "", available: true }
    ]

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    function ids(list) {
        return list.map(function(e) { return e.id; }).join(",");
    }

    Component.onCompleted: {
        let failures = 0;
        const none = { pinned: [], unpinned: [], hidden: [] };

        // Padrao: o fixo aparece; contextual so' com o fato.
        failures += check(ids(projection.visibleEntries(entries, none, {})) === "explorer",
                          "padrao sem fatos");
        failures += check(ids(projection.visibleEntries(entries, none, { "datasource.any": true }))
                          === "explorer,database", "contextual com fato");
        // Fixada aparece sem fato; oculta vence fixada e fato.
        const pinned = { pinned: ["remote"], unpinned: [], hidden: [] };
        failures += check(ids(projection.visibleEntries(entries, pinned, {})) === "explorer,remote",
                          "fixada sem fato");
        const hidden = { pinned: ["remote"], unpinned: [], hidden: ["remote", "database"] };
        failures += check(ids(projection.visibleEntries(entries, hidden, { "datasource.any": true }))
                          === "explorer", "oculta vence");
        // Desfixar o padrao fixo o tira; a ordem segue `order`.
        const unpinned = { pinned: [], unpinned: ["explorer"], hidden: [] };
        failures += check(ids(projection.visibleEntries(entries, unpinned, { "remote.any": true }))
                          === "remote", "desfixada");
        // Teto de 7.
        const many = [];
        for (let i = 0; i < 9; i++) {
            many.push({ id: "a" + i, order: i, defaultPolicy: "pinned" });
        }
        failures += check(projection.visibleEntries(many, none, {}).length === 7, "teto de 7");

        // O estado em palavras e o tom (painel de areas, retorno do autor de
        // 2026-10-02): fixada pelo usuario em destaque; fora do trilho, apagado.
        const pinnedState = { pinned: ["database"], unpinned: [], hidden: ["remote"] };
        failures += check(projection.statusOf(entries[1], pinnedState, true).tone === "accent",
                          "fixada em destaque");
        failures += check(projection.statusOf(entries[2], pinnedState, false).tone === "muted"
                          && projection.statusOf(entries[2], pinnedState, false).text.indexOf("Oculta") === 0,
                          "oculta apagada");
        failures += check(projection.statusOf(entries[1], none, false).text === "Aparece quando houver uso",
                          "contextual fora");
        failures += check(projection.statusOf(entries[0], none, true).text === "Sempre no trilho",
                          "fixa de fabrica");
        failures += check(projection.isPinned(entries[0], none) && !projection.isPinned(entries[1], none)
                          && projection.isPinned(entries[1], pinnedState), "isPinned");
        failures += check(ids(projection.overflowEntries(entries, projection.visibleEntries(entries, none, {})))
                          === "database,remote", "fora do trilho");

        // Fatos: desconhecido mantem o ultimo conhecido.
        failures += check(facts.knownFacts["remote.any"] === undefined, "sem fato ainda");
        remoteFake.targets = [{ name: "pi" }];
        failures += check(facts.knownFacts["remote.any"] === true, "fato chegou");
        remoteFake.targets = undefined;
        failures += check(facts.knownFacts["remote.any"] === true, "desconhecido apagou o ultimo");
        remoteFake.targets = [];
        failures += check(facts.knownFacts["remote.any"] === false, "fato falso vale");

        // ShellController: fixar tira de oculta; ocultar tira de fixada.
        shell.hideArea("database");
        shell.pinArea("database");
        failures += check(shell.railState.pinned.join() === "database"
                          && shell.railState.hidden.length === 0, "fixar tira de oculta");
        shell.hideArea("database");
        failures += check(shell.railState.pinned.length === 0
                          && shell.railState.hidden.join() === "database", "ocultar tira de fixada");
        shell.restoreRail();
        failures += check(shell.railState.hidden.length === 0, "restaurar");

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
