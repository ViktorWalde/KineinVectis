import QtQuick
import KineinVectis

// Os paineis de ambiente sao criados UMA vez (0.3.9, 40.7 §7.195). A lista
// `overlayEntries` do ToolWindows se refaz a cada mudanca de estado; com ela
// como modelo, o Repeater recriava os cinco paineis a cada vez e o painel
// aberto perdia o que tinha. Aqui a lista e' trocada por outra igual, com um
// `active` diferente: o painel tem de ser o MESMO objeto e so' a visibilidade
// muda.
Item {
    id: root

    width: 800
    height: 600

    property int created: 0
    property string requestedKit: "unrequested"
    property int kitRequests: 0
    property int editorFocuses: 0

    QtObject {
        id: editor
        function focusEditor() { root.editorFocuses++; }
    }

    IndexController { id: index }
    ToolchainController {
        id: toolchain
        onGetRequested: preset => { root.requestedKit = preset; root.kitRequests++; }
    }

    Component {
        id: fakePanel

        Item {
            property string typed: ""

            Component.onCompleted: root.created += 1
        }
    }

    QtObject {
        id: fakeToolWindows

        property var overlayEntries: [
            { "id": "remote", "active": false, "panel": fakePanel },
            { "id": "database", "active": false, "panel": fakePanel }
        ]
    }

    QtObject {
        id: fakeOwner

        property bool panelVisible: false
        // O que os dois hosts fixos (bibliotecas e instalacao) leem.
        property var libraries: []
        property string selectedId: ""
        property string target: ""
        property var targets: []
        property string targetsOrigin: ""
        property var plan: null
        property string errorText: ""
        property string distroName: ""
        property var tools: []
        property string expandedId: ""

        function close() {}
    }

    ShellEnvironmentOverlays {
        id: overlays

        anchors.fill: parent
        hostWidth: 800
        hostHeight: 600
        libraryController: fakeOwner
        setupController: fakeOwner
        toolWindows: fakeToolWindows
        indexController: index
        toolchainController: toolchain
        editorController: editor
    }

    // A vaga de cada painel, na ordem da lista (o Repeater fica entre os filhos).
    function slots() {
        const found = [];
        for (let i = 0; i < overlays.children.length; i++) {
            const child = overlays.children[i];
            if (child.entry !== undefined || child.modelData !== undefined) found.push(child);
        }
        return found;
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        // Nada aberto, nada criado: o painel nasce na primeira abertura
        // (40.7 §7.203; criar os cinco na abertura pesava no primeiro quadro).
        failures += check(root.created === 0, "nenhum painel criado fechado: " + root.created);
        const before = slots();
        failures += check(before.length === 2 && !before[0].visible, "duas vagas, fechadas");

        // A lista refeita, como faz o ToolWindows quando um painel abre.
        fakeToolWindows.overlayEntries = [
            { "id": "remote", "active": true, "panel": fakePanel },
            { "id": "database", "active": false, "panel": fakePanel }
        ];
        const opened = slots();
        failures += check(root.created === 1, "abrir cria so' o aberto: " + root.created);
        failures += check(opened[0] === before[0] && opened[0].visible && !opened[1].visible,
                          "a mesma vaga, agora visivel");
        opened[0].panel.typed = "host.example";

        // Fecha e reabre (a lista refeita outra vez): o mesmo painel volta.
        fakeToolWindows.overlayEntries = [
            { "id": "remote", "active": false, "panel": fakePanel },
            { "id": "database", "active": false, "panel": fakePanel }
        ];
        fakeToolWindows.overlayEntries = [
            { "id": "remote", "active": true, "panel": fakePanel },
            { "id": "database", "active": false, "panel": fakePanel }
        ];
        const again = slots();
        failures += check(root.created === 1, "reabrir nao recria: " + root.created);
        failures += check(again[0].panel.typed === "host.example", "o painel guardou o digitado");

        const contextSlot = overlays.children.find(child => child.panel !== undefined && child.entry === undefined);
        failures += check(contextSlot.panel === null, "inspector is lazy");
        index.workspaceRoot = "/project";
        index.openInspector();
        const inspector = contextSlot.panel;
        failures += check(inspector !== null && contextSlot.visible, "first open creates the real inspector");
        index.closeInspector();
        failures += check(!contextSlot.visible, "close hides the inspector");
        index.openInspector();
        failures += check(contextSlot.panel === inspector, "reopen reuses the inspector");
        const previousFocuses = root.editorFocuses;
        inspector.dismissFromKeyboard();
        failures += check(root.editorFocuses === previousFocuses + 1 && !contextSlot.visible,
                          "keyboard dismiss returns focus to the editor");
        index.openInspector();
        index.setActivePath("/project/main.cpp");
        index.handleContext({path: "/project/main.cpp", language: "cpp", effective: {kit: {preset: "cross"}}});
        toolchain.preset = "old";
        index.openContextKit();
        failures += check(root.requestedKit === "cross" && toolchain.menuVisible && root.kitRequests === 1
                          && !contextSlot.visible, "kit navigation reaches the existing toolchain owner");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
