import QtQuick
import "../../ui/qml/git"

// O historico da HUD do Git (fatia 2b): o log chega com pais e refs e vira
// linhas com raia; o filtro de texto e' local (o grafo nao muda); trocar o
// branch pede de novo ao core com o ref; clear esquece tudo.
Item {
    id: root

    property var pedidos: []

    GitHistoryController {
        id: historico

        onLogRequested: function(ref) { root.pedidos.push(ref); }
    }

    Component.onCompleted: {
        let failures = 0;
        historico.openHistory();
        if (root.pedidos.length !== 1 || root.pedidos[0] !== "" || !historico.historyLoading) failures += 1;

        historico.handleLog(true, [
            { sha: "m".repeat(40), shortSha: "mmmmmmm", author: "Ana", authorTime: 1, summary: "merge feature",
              parents: ["b".repeat(40), "x".repeat(40)], refs: ["HEAD -> main", "origin/main"] },
            { sha: "b".repeat(40), shortSha: "bbbbbbb", author: "Ana", authorTime: 1, summary: "docs: readme", parents: ["a".repeat(40)] },
            { sha: "x".repeat(40), shortSha: "xxxxxxx", author: "Bia", authorTime: 1, summary: "feat: coisa", parents: ["a".repeat(40)] },
            { sha: "a".repeat(40), shortSha: "aaaaaaa", author: "Ana", authorTime: 1, summary: "base", parents: [] }
        ]);
        if (historico.historyLoading || historico.historyModel.count !== 4 || historico.laneCount !== 2) failures += 2;
        const topo = historico.historyModel.get(0);
        if (!topo.merge || topo.lane !== 0 || topo.refsText.indexOf("HEAD -> main") !== 0) failures += 4;
        if (historico.historyModel.get(2).lane !== 1) failures += 8;
        if (historico.entry("x".repeat(40)).summary !== "feat: coisa" || historico.entry("nada") !== null) failures += 16;

        // O filtro e' local: nao pede ao core; a raia do que sobra e' a mesma.
        historico.setFilterText("bia");
        if (root.pedidos.length !== 1 || historico.historyModel.count !== 1
                || historico.historyModel.get(0).lane !== 1) failures += 32;
        historico.setFilterText("xxxx");
        if (historico.historyModel.count !== 1) failures += 64;
        historico.setFilterText("");
        if (historico.historyModel.count !== 4) failures += 128;

        // Trocar o branch pede ao core com o ref.
        historico.setLogRef("feature");
        if (root.pedidos.length !== 2 || root.pedidos[1] !== "feature" || !historico.historyLoading) failures += 256;

        historico.clear();
        if (historico.historyModel.count !== 0 || historico.logRef !== "" || historico.filterText !== ""
                || historico.historyVisible) failures += 512;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
