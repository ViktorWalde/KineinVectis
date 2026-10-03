import QtQuick
import KineinVectis

// O ciclo de foco (0.3.9 F4): Ctrl+F6 vai para a proxima area VISIVEL,
// Ctrl+Shift+F6 para a anterior, o fim volta ao comeco; sem area com foco, o
// ciclo comeca no editor; area escondida e' pulada.
Item {
    id: root

    property var visits: []

    component Area: Item {
        id: area

        property string key: ""

        function focusArea() { root.visits.push(area.key); }

        Item { id: inner; objectName: area.key + "-inner" }
        readonly property Item innerItem: inner
    }

    Area { id: left; key: "esquerda" }
    Area { id: editor; key: "editor" }
    Area { id: bottom; key: "baixo" }

    FocusCycle {
        id: cycle

        areas: [left, editor, bottom]
        editorArea: editor
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;
        cycle.cycle(null, 1);
        failures += check(root.visits.join() === "editor", "sem foco comeca no editor: " + root.visits);
        cycle.cycle(editor.innerItem, 1);
        cycle.cycle(bottom.innerItem, 1);
        failures += check(root.visits.join() === "editor,baixo,esquerda", "para frente e volta: " + root.visits);
        cycle.cycle(left.innerItem, -1);
        failures += check(root.visits[3] === "baixo", "para tras do comeco vai ao fim: " + root.visits);
        bottom.visible = false;
        cycle.cycle(editor.innerItem, 1);
        failures += check(root.visits[4] === "esquerda", "area escondida e' pulada: " + root.visits);
        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
