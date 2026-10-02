import QtQuick

// A PROJECAO DO TRILHO (0.3.7 F1, roadmap 53 §4.3): quais areas aparecem, a
// partir da lista de dados do `ToolWindows`, do estado do usuario (fixadas,
// desfixadas, ocultas — persistido no `layout`) e dos fatos do core. So'
// funcoes puras: o `SideRail` desenha, o `ShellController` guarda o estado, o
// harness testa sem janela (tst_rail_projection.qml).
//
//   visivel(e) = e.id fora das ocultas
//              E ( e.id nas fixadas
//                  OU (politica "pinned" E e.id fora das desfixadas)
//                  OU (politica "contextual" E fato(e.factKey)) )
//
// Habilitar nao fixa; fixar nao instala; ocultar nao desabilita (53 §4.2): a
// area oculta continua no "Mais", na paleta e no atalho.
QtObject {
    id: root

    // O trilho nunca passa disto (49 §9.3); o resto vai para o "Mais".
    readonly property int maximumVisible: 7

    function contains(list, id) {
        return list !== undefined && list !== null && list.indexOf(id) >= 0;
    }

    function isVisible(entry, state, facts) {
        if (contains(state.hidden, entry.id)) {
            return false;
        }
        if (contains(state.pinned, entry.id)) {
            return true;
        }
        if (entry.defaultPolicy === "pinned") {
            return !contains(state.unpinned, entry.id);
        }
        if (entry.defaultPolicy === "contextual") {
            return facts[entry.factKey] === true;
        }
        return false;
    }

    // As entradas do trilho, na ordem das entradas, no maximo `maximumVisible`.
    function visibleEntries(entries, state, facts) {
        const shown = entries.filter(function(entry) {
            return root.isVisible(entry, state, facts);
        });
        shown.sort(function(a, b) { return a.order - b.order; });
        return shown.slice(0, root.maximumVisible);
    }

    // As que nao estao no trilho: o "Mais" lista estas.
    function overflowEntries(entries, visible) {
        const ids = visible.map(function(entry) { return entry.id; });
        return entries.filter(function(entry) { return ids.indexOf(entry.id) < 0; });
    }

    // Por que uma area esta' fora do trilho, dito no "Mais".
    function absenceReason(entry, state) {
        if (contains(state.hidden, entry.id)) {
            return qsTr("oculta");
        }
        if (entry.defaultPolicy === "contextual") {
            return qsTr("aparece quando houver uso");
        }
        return qsTr("desafixada");
    }

    // O menu do botao direito num icone do trilho.
    function contextItems(entry, state) {
        const pinnedNow = contains(state.pinned, entry.id)
                          || (entry.defaultPolicy === "pinned" && !contains(state.unpinned, entry.id));
        return [
            { label: qsTr("Abrir"), action: entry.commandId, enabled: entry.available },
            pinnedNow
                ? { label: qsTr("Desafixar do trilho"), action: "rail.unpin:" + entry.id, enabled: true }
                : { label: qsTr("Fixar no trilho"), action: "rail.pin:" + entry.id, enabled: true },
            { label: qsTr("Ocultar do trilho"), action: "rail.hide:" + entry.id, enabled: true },
            { label: qsTr("Restaurar trilho padrão"), action: "rail.restore", enabled: true }
        ];
    }

    // O "Mais": cada area fora do trilho, com o motivo e o atalho, e fixar.
    function overflowItems(entries, state, visible) {
        const items = [];
        const outside = overflowEntries(entries, visible);
        for (let i = 0; i < outside.length; i++) {
            const entry = outside[i];
            const shortcut = entry.shortcut !== undefined && entry.shortcut !== ""
                             ? "  ·  " + entry.shortcut : "";
            items.push({
                label: entry.title + "  —  " + absenceReason(entry, state) + shortcut,
                action: entry.commandId,
                enabled: entry.available
            });
            items.push({
                label: "    " + qsTr("Fixar %1 no trilho").arg(entry.title),
                action: "rail.pin:" + entry.id,
                enabled: true
            });
        }
        items.push({ label: qsTr("Restaurar trilho padrão"), action: "rail.restore", enabled: true });
        return items;
    }
}
