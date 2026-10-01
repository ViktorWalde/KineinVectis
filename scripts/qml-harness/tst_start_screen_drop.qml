import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property string opened: ""

    QtObject {
        id: recents
        property var visibleWorkspaces: []
        property var workspaces: []
        property int hiddenMissingCount: 0
        property string errorText: ""
        property int highlightedIndex: -1
        function moveHighlight() {}
        function openHighlighted() {}
        function clearAll() {}
    }

    StartScreen {
        id: start
        width: 720
        height: 600
        recentWorkspacesController: recents
        urlDecoder: ({localDirectoryPathFromUrls: function(urls) {
            return urls.length === 1 && urls[0] === "file:///tmp/projeto%20novo"
                   ? "/tmp/projeto novo" : "";
        }})
        onWorkspacePathDropped: path => root.opened = path
    }

    function check(condition, message) {
        if (!condition) { console.error("FALHOU: " + message); failures += 1; }
    }

    Component.onCompleted: {
        let action = Qt.IgnoreAction;
        const local = {hasUrls: true, urls: ["file:///tmp/projeto%20novo"],
            accept: function(value) { action = value; }};
        start.acceptFolderDrop(local);
        check(opened === "/tmp/projeto novo" && action === Qt.CopyAction,
              "pasta local não encaminhada para abertura do workspace");
        opened = "";
        const remote = {hasUrls: true, urls: ["https://site.test/projeto"],
            accepted: true, accept: function() { failures += 1; }};
        start.acceptFolderDrop(remote);
        check(opened === "" && !remote.accepted, "URL remota abriu workspace");
        const file = {hasUrls: true, urls: ["file:///tmp/arquivo.txt"], accepted: true,
            accept: function() { failures += 1; }};
        start.acceptFolderDrop(file);
        check(opened === "" && !file.accepted, "arquivo avulso abriu workspace");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
