import QtQuick

Item {
    id: root

    property var coreClient: null
    property var searchController: null

    visible: false

    Connections {
        target: root.coreClient

        function onSearchResults(matches, truncated) {
            root.searchController.handleSearchResults(matches, truncated);
        }

        function onFileSearchResults(matches, truncated) {
            root.searchController.handleFileSearchResults(matches, truncated);
        }

        function onCommandsListed(commands) {
            root.searchController.handleCommandsListed(commands);
        }

        function onRequestFailed(method, message) {
            if (method === "fs.search" || method === "fs.findFiles") {
                root.searchController.handleRequestFailed(method, message);
            }
        }
    }
}
