pragma Singleton
import QtQuick

QtObject {
    id: root

    property bool shown: false
    property string text: ""
    property real anchorX: 0
    property real anchorY: 0
    property real anchorWidth: 0
    property real anchorHeight: 0
    property string placement: "bottom"
    property Item owner: null
    onOwnerChanged: {
        if (!owner) {
            shown = false;
            text = "";
        }
    }
    property Connections ownerVisibility: Connections {
        target: root.owner
        function onVisibleChanged() {
            if (root.owner && !root.owner.visible) root.hideFor(root.owner);
        }
    }

    function showFor(item, message, preferredPlacement) {
        if (!item || !item.visible || message === "") {
            hideFor(item);
            return;
        }
        const point = item.mapToItem(null, 0, 0);
        owner = item;
        text = message;
        anchorX = point.x;
        anchorY = point.y;
        anchorWidth = item.width;
        anchorHeight = item.height;
        placement = preferredPlacement === "right" ? "right" : "bottom";
        shown = true;
    }

    // O texto do dono mudou com o tooltip aberto (o "Atualizar" que era
    // "testa o endereco…", 16.5b): troca no lugar; vazio, esconde.
    function updateFor(item, message) {
        if (!root.shown || root.owner !== item) return;
        if (message === "") hideFor(item);
        else text = message;
    }

    function hideFor(item) {
        if (!item || owner !== item) {
            return;
        }
        shown = false;
        text = "";
        owner = null;
    }
}
