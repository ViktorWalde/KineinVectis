import QtQuick
import KineinVectis

// O painel de areas (0.3.7): as SECOES ficam como estavam ao abrir. Visto na
// tela real em 2026-10-02: fixar uma area a puxava de secao, a lista andava
// sob o ponteiro e o clique seguinte caia no alfinete de outra.
Item {
    id: root

    width: 900
    height: 700

    ShellController { id: shell }

    ToolWindows {
        id: windows

        shellController: shell
        workspaceOpen: true
    }

    RailAreasPopup {
        id: popup

        anchors.fill: parent
        toolWindows: windows
        shellController: shell
    }

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
        popup.openAt(60, 60, "");
        const before = ids(popup.onRail);
        failures += check(before === "explorer,terminal,outline,tools", "no trilho ao abrir: " + before);
        failures += check(!popup.railHas("database"), "banco fora");

        // Fixar o banco: o estado muda, a secao nao.
        popup.togglePin(windows.entries.filter(function(e) { return e.id === "database"; })[0]);
        failures += check(popup.railHas("database"), "banco agora no trilho");
        failures += check(ids(popup.onRail) === before, "a secao andou: " + ids(popup.onRail));

        // Reabrir reorganiza.
        popup.close();
        popup.openAt(60, 60, "");
        failures += check(ids(popup.onRail) === "explorer,terminal,outline,database,tools",
                          "reabrir reorganiza: " + ids(popup.onRail));

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
