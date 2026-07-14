import QtQuick

Item {
    id: root

    property var coreClient: null
    property var assistantController: null

    visible: false

    Connections {
        target: root.coreClient

        function onAiProfilesResolved(profiles, defaultProfile) {
            root.assistantController.handleProfiles(profiles, defaultProfile);
        }

        function onAiTerminalOpened(id, profileId, name, command) {
            root.assistantController.handleTerminalOpened(id, profileId, name, command);
        }

        function onTerminalRender(render) {
            root.assistantController.handleTerminalRender(render);
        }

        function onTerminalClosed(id, exitCode) {
            root.assistantController.handleTerminalClosed(id);
        }

        function onRequestFailed(method, message) {
            root.assistantController.handleRequestFailed(method, message);
        }
    }
}
