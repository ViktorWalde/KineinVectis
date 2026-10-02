import QtQuick

// O RETRATO DO LAYOUT (53 §4.4), montado e lido por funcoes puras. Saiu do
// ShellController quando ele chegou a 393/400: gravar e ler o `layout` e' uma
// responsabilidade, e o controller so' aplica o que este arquivo devolve.
//
// LISTAS: o que chega do settings atravessa o C++ (QVariantMap) e a lista vem
// como QVariantList, que no QML NAO e' um Array do JS — `Array.isArray` da'
// falso. `listOf` copia por `length`, e e' o unico jeito de ler lista aqui
// (o trilho mostrou o defeito em 2026-10-02, 40.7 §7.160).
QtObject {
    id: root

    function listOf(value) {
        const out = [];
        if (value !== undefined && value !== null && typeof value.length === "number"
                && typeof value !== "string") {
            for (let i = 0; i < value.length; i++) {
                out.push(value[i]);
            }
        }
        return out;
    }

    function numberOr(value, fallback) {
        return typeof value === "number" && isFinite(value) && value > 0 ? value : fallback;
    }

    function clamp(value, minimum, maximum) {
        return Math.max(minimum, Math.min(maximum, value));
    }

    function emptyRail() {
        return { pinned: [], unpinned: [], hidden: [] };
    }

    // O retrato gravado (schema 1). `rail`, `bottom.pinned` e `order` sao
    // campos opcionais e aditivos do mesmo schema.
    function snapshot(shell) {
        return {
            schemaVersion: 1,
            leftWindow: shell.leftWindow,
            leftVisible: shell.showExplorer,
            sizes: {
                explorer: Math.round(shell.explorerPreferredWidth),
                outline: Math.round(shell.outlineWidth),
                bottom: Math.round(shell.bottomPreferredHeight)
            },
            outlineCollapsed: shell.outlineCollapsed,
            bottom: { visible: shell.showBottomPanel, tab: shell.bottomTab,
                      pinned: shell.bottomPinned },
            rail: shell.railState,
            order: shell.barOrders
        };
    }

    // Os valores a aplicar, partindo dos atuais: campo ausente ou estranho
    // fica no que ja' estava. Devolve {propriedade: valor} do ShellController.
    function decode(layout, shell) {
        const sizes = layout.sizes !== undefined && layout.sizes !== null ? layout.sizes : {};
        const bottom = layout.bottom !== undefined && layout.bottom !== null ? layout.bottom : {};
        const rail = layout.rail !== undefined && layout.rail !== null ? layout.rail : {};
        const values = {
            explorerPreferredWidth: clamp(numberOr(sizes.explorer, shell.explorerPreferredWidth), 160, 1200),
            outlineWidth: clamp(numberOr(sizes.outline, shell.outlineWidth), 160, 420),
            bottomPreferredHeight: clamp(numberOr(sizes.bottom, shell.bottomPreferredHeight), 120, 1200),
            railState: { pinned: listOf(rail.pinned), unpinned: listOf(rail.unpinned),
                         hidden: listOf(rail.hidden) },
            bottomPinned: listOf(bottom.pinned),
            barOrders: decodeOrders(layout.order)
        };
        if (listOf(shell.leftWindows).indexOf(layout.leftWindow) >= 0) {
            values.leftWindow = layout.leftWindow;
        }
        if (typeof layout.leftVisible === "boolean") {
            values.showExplorer = layout.leftVisible;
        }
        if (typeof layout.outlineCollapsed === "boolean") {
            values.outlineCollapsed = layout.outlineCollapsed;
        }
        if (typeof bottom.tab === "string" && bottom.tab !== "" && bottom.tab !== "git") {
            values.bottomTab = bottom.tab;
        }
        if (typeof bottom.visible === "boolean") {
            values.showBottomPanel = bottom.visible;
        }
        return values;
    }

    // Fixar tira de "desfixada" e de "oculta"; ocultar tira de "fixada";
    // desfixar so' marca. Cada lista sem repeticao.
    function withMembership(state, id, pinned, unpinned, hidden) {
        const without = function(list) {
            return listOf(list).filter(function(x) { return x !== id; });
        };
        const next = { pinned: without(state.pinned), unpinned: without(state.unpinned),
                       hidden: without(state.hidden) };
        if (pinned) next.pinned.push(id);
        if (unpinned) next.unpinned.push(id);
        if (hidden) next.hidden.push(id);
        return next;
    }

    // A ORDEM que o usuario arrastou em cada barra (0.3.9, pedido do autor:
    // "arrastar os icones dentro do rodape, para organiza-los"): {barra:
    // [chaves]}. So' listas de texto sobrevivem; o resto e' ignorado.
    function decodeOrders(value) {
        const orders = {};
        if (value === undefined || value === null || typeof value !== "object") return orders;
        for (const bar in value) {
            orders[bar] = listOf(value[bar]).filter(function(key) { return typeof key === "string"; });
        }
        return orders;
    }

    function withOrder(orders, bar, order) {
        const next = {};
        for (const key in orders) next[key] = orders[key];
        next[bar] = listOf(order);
        return next;
    }

    // As chaves de `keys` na ordem salva; as que a ordem nao conhece (uma
    // area nova, um item que apareceu agora) vem depois, na ordem de fabrica.
    function ordered(keys, saved) {
        const known = listOf(saved).filter(function(key) { return listOf(keys).indexOf(key) >= 0; });
        return known.concat(listOf(keys).filter(function(key) { return known.indexOf(key) < 0; }));
    }

    // Os ITENS (objetos com a chave em `field`) na ordem salva.
    function orderItems(items, field, saved) {
        const list = listOf(items);
        const byKey = {};
        for (const item of list) byKey[item[field]] = item;
        return ordered(list.map(function(item) { return item[field]; }), saved)
            .map(function(key) { return byKey[key]; });
    }

    // Soltar `key` no vao `dropIndex` da lista VISIVEL (0 = antes do
    // primeiro, n = depois do ultimo). As chaves salvas que nao estao a vista
    // agora guardam o lugar delas no fim, para voltarem quando aparecerem.
    function reordered(visible, saved, key, dropIndex) {
        const list = listOf(visible);
        const from = list.indexOf(key);
        if (from < 0) return listOf(saved);
        list.splice(from, 1);
        list.splice(dropIndex > from ? dropIndex - 1 : dropIndex, 0, key);
        return list.concat(listOf(saved).filter(function(other) { return list.indexOf(other) < 0; }));
    }

    // Uma lista com `id` presente (on) ou ausente (off), sem repeticao.
    function withItem(list, id, on) {
        const next = listOf(list).filter(function(x) { return x !== id; });
        if (on) next.push(id);
        return next;
    }
}
