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
          commandId: "view.explorer", shortcut: "", available: true },
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

        // O "Mais" lista o que esta' fora, com motivo, atalho e fixar.
        const visible = projection.visibleEntries(entries, none, {});
        const more = projection.overflowItems(entries, none, visible);
        failures += check(more[0].action === "datasource.list"
                          && more[0].label.indexOf("Ctrl+Alt+J") >= 0, "mais: abrir com atalho");
        failures += check(more[1].action === "rail.pin:database", "mais: fixar");
        failures += check(more[more.length - 1].action === "rail.restore", "mais: restaurar");
        // O menu de contexto troca Fixar por Desafixar conforme o estado.
        failures += check(projection.contextItems(entries[0], none)[1].action === "rail.unpin:explorer",
                          "contexto: desafixar o fixo");
        failures += check(projection.contextItems(entries[1], none)[1].action === "rail.pin:database",
                          "contexto: fixar o contextual");

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
