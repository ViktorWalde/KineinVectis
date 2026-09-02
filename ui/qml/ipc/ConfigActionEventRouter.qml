import QtQuick

// O que o core devolve das Configuration Actions -> ConfigActionController.
//
// Inclui o `requestFailed`, porque neste dominio a RECUSA e informacao de
// produto: "o preset ja existe", "o target nao esta declarado", "o arquivo
// mudou desde o preview". Jogar isso so no log deixaria o usuario com um
// dialogo que nao reage.
Item {
    id: root

    property var coreClient: null
    property var configActionController: null

    visible: false

    Connections {
        target: root.coreClient

        function onConfigActionsListed(actions, activeBuildSystems) {
            root.configActionController.handleListed(actions, activeBuildSystems);
        }

        function onConfigActionPreviewed(preview) {
            root.configActionController.handlePreviewed(preview);
        }

        function onConfigActionApplied(id, message, files, jobId) {
            root.configActionController.handleApplied(id, message, files, jobId);
        }

        function onRequestFailed(method, message) {
            root.configActionController.handleFailed(method, message);
        }
    }
}
