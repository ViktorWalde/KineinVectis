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
    // Onde se pode soltar: por padrao o proprio container; um trilho inteiro
    // quando o container e' so' a coluna de icones (que pode estar vazia).
    property Item dropZone: container
    // A barra irma para onde um item pode PASSAR (o trilho da direita e o da
    // esquerda, 0.3.9). Sobre a zona dela, a linha aparece la' e soltar
    // emite `transferred` com o vao e as chaves de la'.
    property ReorderController partner: null
    property bool overPartner: false

    property string dragKey: ""
    property int dropIndex: -1
    readonly property bool active: root.dragKey !== ""

    signal moved(string key, int dropIndex, var visibleKeys)
    signal transferred(string key, int dropIndex, var targetKeys)

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
        if (root.partner !== null && root.partner.dropZone !== null && root.container !== null) {
            const zone = root.container.mapToItem(root.partner.dropZone, x, y);
            if (zone.x >= 0 && zone.x <= root.partner.dropZone.width
                    && zone.y >= 0 && zone.y <= root.partner.dropZone.height) {
                const there = root.container.mapToItem(root.partner.container, x, y);
                root.overPartner = true;
                root.dropIndex = -1;
                root.partner.receive(root.dragKey, there.x, there.y);
                return;
            }
            if (root.overPartner) {
                root.overPartner = false;
                root.partner.cancel();
            }
        }
        root.placeAt(x, y);
    }

    // Um item de OUTRA barra pairando aqui: so' a linha, nada se apaga.
    function receive(key, x, y) {
        root.dragKey = key;
        root.placeAt(x, y);
    }

    function placeAt(x, y) {
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
        if (root.overPartner) {
            const key = root.dragKey;
            const index = root.partner.dropIndex;
            const targetKeys = root.partner.items().map(function(item) { return item.reorderKey; });
            root.partner.cancel();
            root.overPartner = false;
            root.dragKey = "";
            root.dropIndex = -1;
            if (key !== "" && index >= 0) root.transferred(key, index, targetKeys);
            return;
        }
        const key = root.dragKey;
        const index = root.dropIndex;
        root.dragKey = "";
        root.dropIndex = -1;
        if (key !== "" && index >= 0) {
            root.moved(key, index, items().map(function(item) { return item.reorderKey; }));
        }
    }

    function cancel() {
        if (root.overPartner && root.partner !== null) root.partner.cancel();
        root.overPartner = false;
        root.dragKey = "";
        root.dropIndex = -1;
    }

    // A linha onde o item vai cair, entre dois itens.
    Rectangle {
        id: indicator

        readonly property bool showing: root.active && root.dropIndex >= 0
        readonly property var list: indicator.showing ? root.items() : []
        // A zona e' o trilho inteiro (o container pode estar vazio e sem
        // largura): a linha atravessa a zona, nao a coluna.
        readonly property bool zoned: root.dropZone !== null && root.dropZone !== root.container
        readonly property real gap: {
            if (indicator.list.length === 0) return 1;
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
        visible: indicator.showing
        z: 50
        color: Theme.accent
        radius: Math.min(width, height) / 2
        x: root.vertical && indicator.zoned ? 2
           : (root.container !== null ? root.container.x : 0) + (root.vertical ? 2 : indicator.gap - 1.5)
        y: (root.container !== null ? root.container.y : 0) + (root.vertical ? indicator.gap - 1.5 : 1)
        width: root.vertical ? (indicator.zoned ? root.dropZone.width - 4
                                                : (root.container !== null ? root.container.width - 4 : 0)) : 3
        height: root.vertical ? 3 : (root.container !== null ? root.container.height - 2 : 0)
    }
}
