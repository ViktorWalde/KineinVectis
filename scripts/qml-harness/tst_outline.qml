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
        // O codigo de saida de um processo tem 8 BITS: Qt.exit(256) sai como 0.
        // Enquanto o bitmask ia direto para o exit, todo check com bit >= 256
        // era letra morta: passava verde mesmo quebrado, que e exatamente a
        // doenca que esta suite existe para impedir. O mask agora vai para a
        // SAIDA (onde nao trunca) e o exit so diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
