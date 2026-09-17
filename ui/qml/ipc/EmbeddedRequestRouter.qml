import QtQuick

// Espelho do EmbeddedEventRouter: leva ao core o que o controller pede.
Item {
    id: root

    property var coreClient: null
    property var embeddedController: null

    visible: false

    Connections {
        target: root.embeddedController

        function onListRequested() {
            root.coreClient.probeList();
        }

        function onSizeRequested(program) {
            root.coreClient.buildSize(program);
        }

        function onSerialListRequested() {
            root.coreClient.serialList();
        }

        function onProjectRequested() {
            root.coreClient.projectModel();
        }

        function onMonitorRequested(device, baud) {
            root.coreClient.serialMonitor(device, baud);
        }

        function onIdentifyRequested(device) {
            root.coreClient.serialIdentify(device);
        }
    }
}
