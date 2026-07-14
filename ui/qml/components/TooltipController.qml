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
    property var owner: null

    function showFor(item, message, preferredPlacement) {
        if (!item || message === "") {
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

    function hideFor(item) {
        if (item && owner && owner !== item) {
            return;
        }
        shown = false;
        text = "";
        owner = null;
    }
}
