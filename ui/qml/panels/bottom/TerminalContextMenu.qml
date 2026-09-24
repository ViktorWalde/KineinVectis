pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Ações ergonômicas do terminal. Os ids estáveis permitem que teclado e menu
// percorram o mesmo caminho, sem duplicar a semântica na superfície visual.
Item {
    id: root

    property bool open: false
    property bool available: true
    property string sessionId: ""
    property real popupX: 0
    property real popupY: 0
    property bool canCopy: false
    property bool canPaste: false
    property bool canSelectVisible: false
    property bool canSelectAll: false
    property bool canUseSession: false
    property bool canClearScrollback: false
    property bool canCreateSession: false
    property bool canCloseSession: false

    signal actionRequested(string action)
    signal dismissed()

    visible: open
    anchors.fill: parent
    onAvailableChanged: if (!available) open = false
    onSessionIdChanged: open = false

    function showAt(x, y) {
        if (!available) return;
        popupX = x;
        popupY = y;
        open = true;
    }

    function showForItem(item, x, y) {
        const point = item.mapToItem(root, x, y);
        showAt(point.x, point.y);
    }

    function entries() {
        return [
            { "label": qsTr("Copiar"), "shortcut": qsTr("Ctrl+Shift+C"),
              "action": "terminal.copy", "enabled": canCopy },
            { "label": qsTr("Colar"), "shortcut": qsTr("Ctrl+V"),
              "action": "terminal.paste", "enabled": canPaste },
            { "label": qsTr("Selecionar Tudo"), "shortcut": "",
              "action": "terminal.selectAll", "enabled": canSelectAll },
            { "label": qsTr("Selecionar área visível"), "shortcut": "",
              "action": "terminal.selectVisible", "enabled": canSelectVisible },
            { "label": qsTr("Limpar tela"), "shortcut": qsTr("Ctrl+L"),
              "action": "terminal.clearScreen", "enabled": canUseSession },
            { "label": qsTr("Limpar histórico"), "shortcut": "",
              "action": "terminal.clearScrollback", "enabled": canClearScrollback },
            { "label": qsTr("Novo terminal"), "shortcut": "",
              "action": "terminal.new", "enabled": canCreateSession },
            { "label": qsTr("Fechar terminal"), "shortcut": "",
              "action": "terminal.close", "enabled": canCloseSession }
        ];
    }

    AppMenuPopup {
        anchors.fill: parent
        menuX: root.popupX
        menuY: root.popupY
        menuWidth: 320
        items: root.entries()
        onDismissRequested: function(restoreFocus) {
            root.open = false;
            if (restoreFocus) root.dismissed();
        }
        onActionRequested: function(action) {
            root.open = false;
            root.actionRequested(action);
        }
    }
}
