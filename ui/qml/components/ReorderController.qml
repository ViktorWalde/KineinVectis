import QtQuick
import KineinVectis

// ARRASTAR PARA REORDENAR dentro de uma barra (0.3.9, pedido do autor: "que
// seja possivel arrastar os icones dentro do rodape, para organiza-los da
// forma que eu quiser"). Uma peca para as quatro barras: trilho, abas do
// painel de baixo, cabecalho e status. Quem arrasta e' a ReorderMouseArea de
// cada item; este controlador acha o VAO de soltura pelos centros dos itens
// visiveis do `container` (os que tem `reorderKey`), desenha a linha de
// acento nele e, ao soltar, emite `moved` com o vao — a barra decide onde
// guardar (ShellController.moveInBar). Nada sai da barra: o arrasto so'
// conhece os itens dela.
Item {
    id: root

    property Item container: null
    property bool vertical: false

    property string dragKey: ""
    property int dropIndex: -1
    readonly property bool active: root.dragKey !== ""

    signal moved(string key, int dropIndex, var visibleKeys)

    // O item arrastado fica translucido: o olho ve o que esta' saindo do lugar.
    function opacityFor(key) {
        return root.active && root.dragKey === key ? 0.4 : 1.0;
    }

    // Os itens arrastaveis do container, na ordem em que aparecem.
    function items() {
        const found = [];
        if (root.container === null) return found;
        const children = root.container.children;
        for (let i = 0; i < children.length; i++) {
            const child = children[i];
            if (child.visible && child.reorderKey !== undefined && child.reorderKey !== "") {
                found.push(child);
            }
        }
        found.sort(function(a, b) { return root.vertical ? a.y - b.y : a.x - b.x; });
        return found;
    }

    function begin(key) {
        root.dragKey = key;
        root.dropIndex = -1;
    }

    // `x`,`y` em coordenadas do container.
    function update(x, y) {
        const list = items();
        const position = root.vertical ? y : x;
        let index = 0;
        for (const item of list) {
            const center = root.vertical ? item.y + item.height / 2 : item.x + item.width / 2;
            if (position > center) index += 1;
        }
        root.dropIndex = index;
    }

    function finish() {
        const key = root.dragKey;
        const index = root.dropIndex;
        root.dragKey = "";
        root.dropIndex = -1;
        if (key !== "" && index >= 0) {
            root.moved(key, index, items().map(function(item) { return item.reorderKey; }));
        }
    }

    function cancel() {
        root.dragKey = "";
        root.dropIndex = -1;
    }

    // A linha onde o item vai cair, entre dois itens.
    Rectangle {
        id: indicator

        readonly property var list: root.active && root.dropIndex >= 0 ? root.items() : []
        readonly property real gap: {
            if (indicator.list.length === 0) return 0;
            // Nunca antes do comeco: o container costuma recortar (clip) e a
            // linha do vao 0 sumia.
            if (root.dropIndex < indicator.list.length) {
                const next = indicator.list[root.dropIndex];
                return Math.max(1, (root.vertical ? next.y : next.x) - 1);
            }
            const last = indicator.list[indicator.list.length - 1];
            return (root.vertical ? last.y + last.height : last.x + last.width) + 1;
        }

        // No PAI do container: um Row/Column posicionaria a linha como item.
        parent: root.container !== null ? root.container.parent : null
        visible: indicator.list.length > 0
        z: 50
        color: Theme.accent
        radius: Math.min(width, height) / 2
        x: (root.container !== null ? root.container.x : 0) + (root.vertical ? 2 : indicator.gap - 1.5)
        y: (root.container !== null ? root.container.y : 0) + (root.vertical ? indicator.gap - 1.5 : 1)
        width: root.vertical ? (root.container !== null ? root.container.width - 4 : 0) : 3
        height: root.vertical ? 3 : (root.container !== null ? root.container.height - 2 : 0)
    }
}
