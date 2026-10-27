import QtQuick

// Explicit image and alternative; the core owns format, memory and identity checks.
Item {
    id: root
    visible: false
    property var discovery: null
    property string image: ""
    property bool binaryMode: false
    property var destination: ({})
    signal changed()
    onImageChanged: changed()
    onBinaryModeChanged: changed()
    onDestinationChanged: changed()

    function clear() { image = ""; binaryMode = false; forgetDestination(); }
    function forgetDestination() { if (Object.keys(destination).length > 0) destination = ({}); }
    function setImage(value) { image = value; }
    function selectBinary(value) { binaryMode = value; }

    function key(entry) {
        return JSON.stringify([entry.vid, entry.pid, entry.serial || "", entry.path,
            entry.configuration, entry.interface, entry.alternate]);
    }

    function selectable(entry) {
        return discovery && discovery.isDfu(entry) && entry.vid === "0483" && entry.pid === "df11"
            && entry.configuration === 1 && entry.interface === 0 && !!entry.serial && entry.serial !== "UNKNOWN";
    }

    function select(entry) {
        if (!discovery || !discovery.complete || !selectable(entry)
            || !discovery.interfaces.some(current => key(current) === key(entry))) return;
        destination = key(destination) === key(entry) ? ({}) : {
            vid: entry.vid, pid: entry.pid, serial: entry.serial, path: entry.path,
            configuration: entry.configuration, interface: entry.interface, alternate: entry.alternate
        };
    }

    function payload(target) {
        if (image === "" || !destination.serial) return ({});
        const result = {image: image, destination: destination};
        if (binaryMode) result.binaryTarget = target || "";
        return result;
    }

    Connections {
        target: root.discovery
        function onInterfacesChanged() { root.forgetDestination(); }
    }
}
