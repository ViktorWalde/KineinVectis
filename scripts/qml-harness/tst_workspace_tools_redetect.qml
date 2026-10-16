import QtQuick
import KineinVectis

// 16.1, F4 (roadmaps/59 §7.1.1): trocar de projeto pede as ferramentas de
// novo, porque o core mede as versoes na raiz do projeto aberto.
Item {
    id: root

    QtObject {
        id: bridge
        signal workspaceChanged()
        // Os outros sinais que o roteador ouve (sem eles, o Connections avisa).
        signal statusChanged()
        signal cmakeStatusResolved(bool configured, bool hasCompileCommands, bool cdbStale,
                                   string cdbStaleBecause, string preset)
        signal cmakeConfigureFinished(bool success)
        signal fileSaved(string path)
        signal cargoMetadataResolved(var packages)
        signal dirListed(string path, var entries)
        signal fileCreated(string path)
        signal directoryCreated(string path)
        signal pathRenamed(string from, string to)
        signal pathCopied(string from, string to)
        signal pathBatchTransferred(var result)
        signal pathCopyFailed(string to, string message)
        signal pathDeleted(string path)
        signal filesChanged(var changes)
        signal workspaceBrowseListed(string path, string parent, var entries, var places)
        signal workspaceFolderCreated(string path)
        signal recentWorkspacesResolved(var workspaces)
        signal toolsListed(var tools)
        signal environmentTool(var tool)
        signal requestFailed(string method, string message)
        property int detects: 0
        function detectTools() { bridge.detects++; }
    }
    QtObject {
        id: picker
        property int closed: 0
        function close() { picker.closed++; }
    }
    QtObject {
        id: workspace
        property int changed: 0
        function handleWorkspaceChanged() { workspace.changed++; }
    }
    WorkspaceEventRouter {
        coreClient: bridge
        folderPicker: picker
        workspaceController: workspace
    }
    Component.onCompleted: {
        bridge.workspaceChanged();
        const ok = bridge.detects === 1 && picker.closed === 1 && workspace.changed === 1;
        if (!ok) console.error("FALHOU: trocar de projeto pede tools.detect (" + bridge.detects + ")");
        Qt.exit(ok ? 0 : 1);
    }
}
