import QtQuick

// Espelho do ContainerEventRouter: leva ao core o que o controller pede. E'
// a UNICA ponte — a UI nunca chama `docker`/`podman` (roadmaps/28 §4).
Item {
    id: root

    property var coreClient: null
    property var containerController: null

    visible: false

    Connections {
        target: root.containerController

        function onStatusRequested() {
            root.coreClient.containerStatus();
        }

        function onListRequested(all) {
            root.coreClient.containerList(all);
        }

        function onImagesRequested() {
            root.coreClient.containerImages();
        }

        function onActionRequested(id, action) {
            root.coreClient.containerAction(id, action);
        }

        function onOpenRequested(id, mode) {
            root.coreClient.containerOpen(id, mode);
        }

        function onComposeRequested(action, file) {
            root.coreClient.containerCompose(action, file);
        }
    }
}
