import QtQuick
import "../../ui/qml/editor"

Item {
    width: 320
    height: 240

    EditorOutlineController {
        id: outline
    }

    Component.onCompleted: {
        let failures = 0;
        const method = {
            name: "run", kind: "method", line: 3, column: 5,
            children: []
        };
        const helper = {
            name: "helper", kind: "function", line: 8, column: 1,
            children: []
        };
        const device = {
            name: "Device", kind: "class", line: 1, column: 1,
            children: [method]
        };
        outline.items = [device, helper];
        outline.rebuild();
        if (outline.count !== 3) failures += 1;

        const key = outline.nodeKey("root", device, 0);
        outline.toggle(key);
        if (outline.count !== 2) failures += 2;
        outline.toggle(key);
        if (outline.count !== 3) failures += 4;

        Qt.exit(failures);
    }
}
