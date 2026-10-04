pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Dropdown do seletor de run configs da Main Toolbar. Vive nos overlays
// (nao dentro do header) porque o header tem 44px e os paineis desenhados
// depois dele cobririam o menu — mesmo racional do ProjectEntryContextMenu.
//
// Desde 2026-10-03 e' o AppMenuPopup de toda a IDE: a configuracao ativa leva
// o ✓, as acoes ficam num grupo proprio, e o teclado (setas, Enter, Esc) vem
// junto. Antes eram linhas desenhadas a mao, sem teclado.
Item {
    id: root

    property real menuX: 0
    property real menuY: 0
    property var configsModel
    property string activeConfigId: ""

    signal dismissRequested()
    signal configChosen(string id)
    signal newRequested()
    signal editRequested()
    signal deleteRequested()

    readonly property string configPrefix: "config:"

    function entries() {
        const items = [{ label: qsTr("Automático"), action: root.configPrefix,
                         icon: root.activeConfigId === "" ? "check" : "", enabled: true, shortcut: "" }];
        const count = root.configsModel ? root.configsModel.count : 0;
        for (let index = 0; index < count; index++) {
            const config = root.configsModel.get(index);
            items.push({ label: config.name, action: root.configPrefix + config.id,
                         icon: root.activeConfigId === config.id ? "check" : "", enabled: true, shortcut: "" });
        }
        items.push({ separator: true, label: "", action: "", enabled: false });
        items.push({ label: qsTr("Nova configuração…"), action: "new", icon: "add", enabled: true, shortcut: "" });
        if (root.activeConfigId !== "") {
            items.push({ label: qsTr("Editar atual…"), action: "edit", icon: "configure", enabled: true, shortcut: "" });
            items.push({ label: qsTr("Excluir atual"), action: "delete", icon: "trash", enabled: true, shortcut: "" });
        }
        return items;
    }

    function dispatch(action) {
        if (action.indexOf(root.configPrefix) === 0) root.configChosen(action.slice(root.configPrefix.length));
        else if (action === "new") root.newRequested();
        else if (action === "edit") root.editRequested();
        else if (action === "delete") root.deleteRequested();
    }

    AppMenuPopup {
        anchors.fill: parent
        menuX: root.menuX
        menuY: root.menuY
        menuWidth: 260
        items: root.entries()
        onDismissRequested: root.dismissRequested()
        onActionRequested: function(action) { root.dispatch(action); }
    }
}
