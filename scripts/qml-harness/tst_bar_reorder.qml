import QtQuick
import KineinVectis

// Arrastar para reordenar dentro de uma barra (0.3.9, pedido do autor). Prova
// as funcoes puras do codec (ordem salva, item novo no fim, soltar num vao,
// chave escondida guarda o lugar), o ReorderController (vao pelos centros,
// `moved` com as chaves visiveis, cancelar) e a faixa de abas de baixo
// desenhando na ordem do usuario.
Item {
    id: root

    width: 800
    height: 200

    property var moves: []

    ShellLayoutCodec {
        id: codec
    }

    Row {
        id: row

        y: 10
        spacing: 4

        Repeater {
            model: ["a", "b", "c"]

            delegate: Rectangle {
                required property string modelData
                readonly property string reorderKey: modelData

                width: 40
                height: 20
            }
        }
    }

    Column {
        id: otherColumn

        x: 300
        y: 10
        width: 40

        Repeater {
            model: ["x"]

            delegate: Rectangle {
                required property string modelData
                readonly property string reorderKey: modelData

                width: 40
                height: 20
            }
        }
    }

    property var transfers: []

    // A barra parceira: a coluna de cima e' a "outra" barra (os trilhos).
    ReorderController {
        id: otherReorder

        container: otherColumn
        vertical: true
    }

    ReorderController {
        id: reorder

        partner: otherReorder
        onTransferred: function(key, dropIndex, targetKeys) {
            root.transfers.push(key + "@" + dropIndex + ":" + targetKeys.join(""));
        }
        container: row
        onMoved: function(key, dropIndex, visibleKeys) {
            root.moves.push(key + "@" + dropIndex + ":" + visibleKeys.join(""));
        }
    }

    BottomTabBar {
        id: tabs

        width: 800
        activeTab: "terminal"
        tabOrder: ["problems", "terminal"]
    }

    WorkspaceStatusBar {
        id: status

        y: 100
        width: 800
        coreConnected: true
        cursorSummary: "1:1"
        lspSummary: "LSP ● 2"
        rightOrder: ["core", "ide"]
    }

    // As chaves visiveis de uma faixa da barra de status, na ordem desenhada.
    function stripKeys(strip) {
        const keys = [];
        for (let i = 0; i < strip.children.length; i++) {
            const child = strip.children[i];
            if (child.visible && child.reorderKey !== undefined) keys.push(child.reorderKey);
        }
        return keys.join(",");
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    Component.onCompleted: {
        let failures = 0;

        // Ordem salva; o que ela nao conhece vem depois, na ordem de fabrica.
        failures += check(codec.ordered(["a", "b", "c", "d"], ["c", "a"]).join("") === "cabd", "ordered");
        failures += check(codec.ordered(["a", "b"], ["z", "b"]).join("") === "ba", "chave sumida nao entra");
        // Soltar num vao: para frente e para tras.
        failures += check(codec.reordered(["a", "b", "c"], [], "a", 3).join("") === "bca", "a para o fim");
        failures += check(codec.reordered(["a", "b", "c"], [], "c", 0).join("") === "cab", "c para o comeco");
        failures += check(codec.reordered(["a", "b", "c"], [], "b", 1).join("") === "abc", "no proprio lugar");
        // O que esta' salvo e nao a vista guarda o lugar no fim.
        failures += check(codec.reordered(["a", "b"], ["x", "b", "a"], "a", 0).join("") === "abx",
                          "escondida guardada: " + codec.reordered(["a", "b"], ["x", "b", "a"], "a", 0));
        // Decodificar: so' listas de texto.
        const orders = codec.decodeOrders({ bottom: ["jobs", 3, "terminal"], rail: "x" });
        failures += check(orders.bottom.join(",") === "jobs,terminal" && orders.rail.length === 0, "decode");
        failures += check(Object.keys(codec.decodeOrders(null)).length === 0, "decode nulo");
        failures += check(codec.withOrder({ rail: ["a"] }, "bottom", ["b"]).rail.join("") === "a", "withOrder preserva");

        // O controlador: vao pelos centros (a=0..40, b=44..84, c=88..128).
        reorder.begin("a");
        reorder.update(100, 5);
        failures += check(reorder.dropIndex === 2, "vao depois de b: " + reorder.dropIndex);
        reorder.update(130, 5);
        failures += check(reorder.dropIndex === 3, "vao no fim: " + reorder.dropIndex);
        reorder.finish();
        failures += check(root.moves.join() === "a@3:abc", "moved: " + root.moves.join());
        failures += check(!reorder.active, "terminou");
        reorder.begin("b");
        reorder.update(0, 0);
        reorder.cancel();
        failures += check(root.moves.length === 1 && !reorder.active, "cancelar nao move");

        // Passar para a barra parceira: sobre ela a linha muda de barra, e
        // soltar transfere com o vao e as chaves de la' (0.3.9, os trilhos).
        reorder.begin("b");
        reorder.update(305, 15);
        failures += check(reorder.overPartner && otherReorder.active && otherReorder.dropIndex === 1,
                          "linha no parceiro: " + otherReorder.dropIndex);
        reorder.finish();
        failures += check(root.transfers.join() === "b@1:x", "transferido: " + root.transfers.join());
        failures += check(!otherReorder.active && !reorder.active, "os dois terminaram");
        failures += check(codec.inserted(["x"], [], "b", 1).join("") === "xb", "inserted no fim");
        failures += check(codec.inserted(["x", "y"], ["z"], "b", 0).join("") === "bxyz", "inserted no comeco");
        const sided = codec.withSide(codec.emptyRail(), "outline", "left");
        failures += check(sided.sides.outline === "left", "withSide");
        failures += check(codec.withMembership(sided, "git", true, false, false).sides.outline === "left",
                          "fixar preserva os lados");
        failures += check(codec.decodeSides({ a: "right", b: "cima" }).b === undefined, "lado estranho fora");

        // A faixa de abas na ordem do usuario.
        failures += check(tabs.orderedTabs[0].key === "problems" && tabs.orderedTabs[1].key === "terminal",
                          "abas na ordem salva");
        failures += check(tabs.orderedTabs.length === tabs.allTabs.length, "ordem parcial nao some com abas");

        // A barra de status: a faixa da direita na ordem salva, o resto depois.
        // `children` nao e' Array do JS no Qt 6.4: procura por indice.
        let rightStrip = null;
        for (let i = 0; i < status.children.length; i++) {
            if (status.children[i].anchors.right === status.right) rightStrip = status.children[i];
        }
        failures += check(stripKeys(rightStrip) === "core,ide,cursor,lsp", "status: " + stripKeys(rightStrip));

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
