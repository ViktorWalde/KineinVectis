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

    // O ESTADO de uma area em palavras, com o tom em que aparece no painel de
    // areas (retorno do autor, 2026-10-02: "tudo numa cor so', em lista, fica
    // confuso"). Tom: "accent" = fixada pelo usuario; "normal" = no trilho;
    // "muted" = fora dele.
    function statusOf(entry, state, onRail) {
        if (contains(state.hidden, entry.id)) {
            return { text: qsTr("Oculta neste projeto"), tone: "muted" };
        }
        if (contains(state.pinned, entry.id)) {
            return { text: qsTr("Fixada por você"), tone: "accent" };
        }
        if (entry.defaultPolicy === "pinned") {
            return contains(state.unpinned, entry.id)
                ? { text: qsTr("Desafixada"), tone: "muted" }
                : { text: qsTr("Sempre no trilho"), tone: "normal" };
        }
        return onRail ? { text: qsTr("No trilho porque está em uso"), tone: "normal" }
                      : { text: qsTr("Aparece quando houver uso"), tone: "muted" };
    }

    // Fixa agora? (explicita ou pelo padrao, sem ter sido desafixada.)
    function isPinned(entry, state) {
        return contains(state.pinned, entry.id)
            || (entry.defaultPolicy === "pinned" && !contains(state.unpinned, entry.id)
                && !contains(state.hidden, entry.id));
    }
}
