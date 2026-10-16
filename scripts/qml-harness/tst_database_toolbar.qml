pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// 16.5b (roadmaps/59 §7.1.5): o ↻ do Banco rele o catalogo mesmo sem linha
// selecionada (F2 da 16.1: o `datasources.json` editado fora da IDE), e os
// botoes da barra entram no ciclo de Tab (40.7 §7.254). DatabaseWindow e
// DataSourceController REAIS.
Item {
    id: root
    width: 400
    height: 600
    property int lists: 0
    property int failures: 0

    function check(condition, label) {
        if (!condition) { root.failures++; console.error("FALHOU: " + label); }
    }

    DataSourceController {
        id: bankController
        workspaceRoot: "/projeto"
        onListRequested: root.lists++
    }
    DatabaseWindow {
        id: bank
        width: 300
        height: 580
        controller: bankController
        menuLayer: root
    }

    function buttons(item, found) {
        if (item.iconName !== undefined && item.compact !== undefined) found.push(item);
        for (let index = 0; index < item.children.length; index++) root.buttons(item.children[index], found);
        return found;
    }

    Component.onCompleted: {
        const all = root.buttons(bank, []);
        const refresh = all.find(button => button.iconName === "refresh");
        root.check(!!refresh, "o ↻ existe");
        if (!refresh) { Qt.exit(1); return; }
        root.check(refresh.enabled, "o ↻ vale sem linha selecionada");
        const before = root.lists;
        refresh.clicked();
        root.check(root.lists === before + 1, "o ↻ pede o catálogo: " + (root.lists - before));
        const toolbar = all.filter(button => ["add", "refresh", "terminal", "table", "locate", "collapse"].indexOf(button.iconName) >= 0);
        root.check(toolbar.length === 6 && toolbar.every(button => button.activeFocusOnTab),
            "a barra no ciclo de Tab: " + toolbar.map(button => button.iconName + "=" + button.activeFocusOnTab).join(" "));
        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
