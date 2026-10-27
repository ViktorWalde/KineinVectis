import QtQuick

// Selection is only observed data; the core owns UF2/ELF and USB validation.
Item {
    id: root
    visible: false
    property var discovery: null
    property string image: ""
    property var destination: ({})
    signal changed()
    onImageChanged: changed()
    onDestinationChanged: changed()
    function clear() { image = ""; forgetDestination(); }
    function setImage(value) { image = value; }
    function forgetDestination() { if (Object.keys(destination).length > 0) destination = ({}); }
    function key(entry) { return JSON.stringify([entry.chip, entry.serialSelector, entry.flashSizeBytes]); }
    function selectable(entry) { return entry.chip === "rp2040" && !!entry.serialSelector && entry.flashSizeBytes > 0; }
    function select(entry) {
        if (!discovery || !discovery.complete || !selectable(entry)
            || !discovery.devices.some(current => key(current) === key(entry))) return;
        destination = key(destination) === key(entry) ? ({}) : {
            chip: entry.chip, serialSelector: entry.serialSelector, flashSizeBytes: entry.flashSizeBytes
        };
    }
    function payload() { return image !== "" && destination.serialSelector ? {image: image, destination: destination} : ({}); }
    Connections {
        target: root.discovery
        function onDevicesChanged() { root.forgetDestination(); }
    }
}
