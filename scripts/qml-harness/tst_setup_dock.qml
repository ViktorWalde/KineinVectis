import QtQuick
import KineinVectis

// Catalogue recovery and dock state share the real owners, without a CoreClient.
Item {
    id: root
    width: 1280
    height: 800
    property int requests: 0
    property int failures: 0
    property var catalogue: [
        { id: "arm", name: "ARM GCC", summary: "Bare metal compiler", installed: false },
        { id: "clangd", name: "clangd", summary: "C++ language server", installed: true }
    ]

    ShellController { id: shell }
    SetupController {
        id: setup
        onWindowRequested: shell.showDockWindow("setup")
        onListRequested: root.requests++
    }
    SetupPanelHost {
        id: panel
        width: 360
        height: 600
        visible: shell.docks.showing("setup") !== ""
        controller: setup
        onCloseRequested: shell.docks.close("setup")
    }

    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }

    Component.onCompleted: {
        shell.updateViewport(1280, 800);
        setup.open();
        check(shell.docks.slotVisible("left", false), "Setup abre sem workspace");
        check(panel.visible && setup.panelVisible && setup.busy && requests === 1, "host prepara catálogo uma vez");
        setup.open();
        setup.refresh();
        check(requests === 1, "abertura e refresh durante carga não duplicam pedido");
        setup.handleList("Test Linux", "test", catalogue);
        check(!setup.busy && setup.filteredTools.length === 2, "resposta libera lista");
        setup.query = "  COMPILER  ";
        check(setup.filteredTools.length === 1 && setup.filteredTools[0].id === "arm", "busca ignora caixa e espaços");
        setup.query = "";
        setup.availability = "installed";
        check(setup.filteredTools.length === 1 && setup.filteredTools[0].id === "clangd", "filtro instalado");
        setup.availability = "missing";
        check(setup.filteredTools[0].id === "arm", "filtro ausente");
        setup.toggle("arm");
        shell.docks.close("setup");
        check(!setup.panelVisible, "fechar sincroniza controller");
        setup.open();
        check(requests === 2 && setup.expandedId === "arm" && setup.availability === "missing", "reabrir conserva consulta/detalhes e atualiza detecção");
        setup.handleFailed("setup.list", "offline");
        check(!setup.busy && setup.errorText === "offline" && setup.tools.length === 0, "falha não mostra catálogo antigo como atual");
        setup.refresh();
        check(requests === 3 && setup.busy && setup.errorText === "", "retry limpa falha");
        setup.handleList("Test Linux", "test", catalogue);
        shell.moveRailEntryToSide("setup", "right", 0, []);
        check(shell.docks.showing("setup") === "right" && shell.docks.slotVisible("right", false), "dock acompanha lado sem workspace");
        const snapshot = shell.layoutSnapshot();
        shell.docks.close("setup");
        shell.applySettings({ layout: snapshot });
        check(shell.docks.showing("setup") === "right", "layout restaura Setup no lado escolhido");
        shell.docks.close("setup");
        shell.showDockWindow("database");
        check(!shell.docks.slotVisible("left", false), "exceção não libera os outros docks sem projeto");
        Qt.exit(root.failures > 0 ? 1 : 0);
    }
}
