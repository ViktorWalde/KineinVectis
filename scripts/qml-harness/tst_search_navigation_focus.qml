import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property int focuses: 0
    property int queries: 0
    property int commands: 0
    property string opened: ""
    property int phase: 0

    SearchEverywhereController {
        id: search
        workspaceRoot: "/project"
        onFocusEditorRequested: root.focuses++
        onFindFilesRequested: root.queries++
        onReadFileRequested: path => root.opened = path
        onCommandAccepted: root.commands++
    }

    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }

    Component.onCompleted: {
        search.openSearchEverywhere();
        search.handleFileSearchResults([{ path: "main.cpp", name: "main.cpp" }], false);
        search.scheduleSearchEverywhere("pending");
        search.acceptSearchEverywhere();
        check(!search.everywhereVisible && opened === "/project/main.cpp", "aceite abre arquivo e fecha busca");
        search.handleRequestFailed("fs.findFiles", "late failure");
        check(!search.everywhereVisible, "erro tardio não reabre busca");
        settle.start();
    }

    Timer {
        id: settle
        interval: 300
        onTriggered: {
            if (root.phase === 0) {
                root.check(root.focuses === 1, "navegação solicita foco ao editor");
                root.check(root.queries === 0, "aceite cancela consulta do debounce");
                search.openSearchEverywhere();
                search.appendItem("command", "Setup", "", "", "setup.list", 0, 0);
                search.acceptSearchEverywhere();
                root.check(root.commands === 1, "comando aceito");
                root.phase = 1;
                restart();
            } else {
                root.check(root.focuses === 1, "comando conserva foco de seu próprio dono");
                Qt.exit(root.failures > 0 ? 1 : 0);
            }
        }
    }
}
