import QtQuick
import KineinVectis

// O WorkspaceEventRouter REAL com um bridge falso:
// - 16.1, F4 (roadmaps/59 §7.1.1): trocar de projeto pede as ferramentas de
//   novo, porque o core mede as versoes na raiz do projeto aberto;
// - 16.4b (§7.1.4): salvar o CMakeLists.txt so' reconfigura (roda o CMake do
//   projeto) com a confianca, e a confianca chega a' faixa de saude;
// - C1-23 (0.210.0): o `cmake.status` chega a' faixa com a configuracao ativa
//   e o menu dela, o nono argumento da ponte.
Item {
    id: root

    QtObject {
        id: bridge
        signal workspaceChanged()
        // Os outros sinais que o roteador ouve (sem eles, o Connections avisa).
        signal statusChanged()
        signal cmakeStatusResolved(bool configured, bool hasCompileCommands, bool cdbStale,
                                   string cdbStaleBecause, string preset, var presetChoices,
                                   string presetChoicesFile, var compileDatabase,
                                   var configuration)
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
        signal projectEnvironmentsResolved(var environments)
        signal workspaceFolderCreated(string path)
        signal recentWorkspacesResolved(var workspaces)
        signal toolsListed(var tools)
        signal environmentTool(var tool)
        signal requestFailed(string method, string message)
        property bool workspaceTrusted: false
        property int detects: 0
        property int configures: 0
        property var workspaceBuildSystems: ["cmake"]
        function detectTools() { bridge.detects++; }
        function cmakeConfigure() { bridge.configures++; }
    }
    QtObject {
        id: picker
        property int closed: 0
        property var environments: []
        property string environmentError: ""
        function close() { picker.closed++; }
        function setProjectEnvironments(entries) { environments = entries; }
        function projectEnvironmentsFailed(message) { environmentError = message; }
    }
    QtObject {
        id: workspace
        property int changed: 0
        function handleWorkspaceChanged() { workspace.changed++; }
    }
    QtObject {
        id: health
        property var trust: []
        property var cmakeStatus: []
        function handleTrust(trusted) { health.trust = health.trust.concat([trusted]); }
        function handleCmakeStatus(configured, stale, staleBecause, preset, choices, choicesFile,
                                   database, configuration) {
            health.cmakeStatus = [configured, stale, staleBecause, preset, choices, choicesFile,
                                  database, configuration];
        }
    }
    WorkspaceEventRouter {
        coreClient: bridge
        folderPicker: picker
        workspaceController: workspace
        projectHealthController: health
        recentWorkspacesController: QtObject {
            function handleRequestFailed(method, message) {}
        }
    }
    Component.onCompleted: {
        bridge.workspaceChanged();
        const ok = bridge.detects === 1 && picker.closed === 1 && workspace.changed === 1;
        if (!ok) console.error("FALHOU: trocar de projeto pede tools.detect (" + bridge.detects + ")");
        bridge.fileSaved("/p/CMakeLists.txt");
        const withoutTrust = bridge.configures === 0;
        if (!withoutTrust) console.error("FALHOU: sem confiança, salvar o CMakeLists.txt reconfigurou");
        bridge.workspaceTrusted = true;
        bridge.fileSaved("/p/CMakeLists.txt");
        const trusted = bridge.configures === 1 && JSON.stringify(health.trust) === "[true]";
        if (!trusted) console.error("FALHOU: com confiança: " + bridge.configures + " " + JSON.stringify(health.trust));
        bridge.projectEnvironmentsResolved([{language: "c", executable: "/old/gcc", version: "GCC 10"},
                                            {language: "c", executable: "/new/gcc", version: "GCC 16"}]);
        bridge.requestFailed("workspace.projectEnvironments", "scan failed");
        const environments = picker.environments.length === 2 && picker.environmentError === "scan failed";
        if (!environments) console.error("FALHOU: resultado/erro de ambientes não chegou ao seletor");
        const menu = { preset: "debug", source: "configuration", refused: "", directory: "build/debug",
                       choices: [{ preset: "debug", buildDir: "build/debug", configured: true }] };
        bridge.cmakeStatusResolved(true, true, false, "", "debug", [], "",
                                   { selectedConfiguration: "debug" }, menu);
        const status = health.cmakeStatus;
        const configuration = status.length === 8 && status[0] === true && status[3] === "debug"
                && status[6].selectedConfiguration === "debug"
                && status[7].preset === "debug" && status[7].source === "configuration"
                && status[7].directory === "build/debug" && status[7].choices.length === 1
                && status[7].choices[0].buildDir === "build/debug";
        if (!configuration) console.error("FALHOU: o status não levou a configuração: " + JSON.stringify(status));
        Qt.exit(ok && withoutTrust && trusted && environments && configuration ? 0 : 1);
    }
}
