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
        failures += check(root.created === 2, "dois paineis criados: " + root.created);
        const before = slots();
        failures += check(before.length === 2 && !before[0].visible, "duas vagas, fechadas");
        before[0].children[0].typed = "host.example";

        // A lista refeita, como faz o ToolWindows quando um painel abre.
        fakeToolWindows.overlayEntries = [
            { "id": "remote", "active": true, "panel": fakePanel },
            { "id": "database", "active": false, "panel": fakePanel }
        ];
        const after = slots();
        failures += check(root.created === 2, "nenhum painel recriado: " + root.created);
        failures += check(after[0] === before[0] && after[0].visible && !after[1].visible,
                          "a mesma vaga, agora visivel");
        failures += check(after[0].children[0].typed === "host.example", "o painel guardou o digitado");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
