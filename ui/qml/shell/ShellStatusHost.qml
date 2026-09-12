import QtQuick
import KineinVectis

WorkspaceStatusBar {
    id: root

    property var coreClient: null
    property var shellController: null
    property var gitController: null
    property var toolchainController: null
    property var indexController: null

    workspaceRoot: coreClient.workspaceRoot
    workspaceKindLabel: shellController.kindLabel(
                            coreClient.workspaceKind,
                            coreClient.workspaceBuildSystems)
    logsActive: shellController.showBottomPanel
                && shellController.bottomTab === "logs"
    building: coreClient.building
    testing: coreClient.testing
    analyzing: coreClient.analyzing
    scanningEnvironment: coreClient.scanningEnvironment
    running: coreClient.running
    coreConnected: coreClient.connected
    coreProtocolVersion: coreClient.protocolVersion
    coreStatus: coreClient.status
    gitBranchLabel: gitController.branchLabel
    gitAheadCount: gitController.aheadCount
    gitBehindCount: gitController.behindCount
    gitChangeCount: gitController.changeCount
    toolchainVisible: coreClient.workspaceRoot !== ""
                      && coreClient.workspaceBuildSystems.length > 0
    toolchainSummary: toolchainController.summary()
    indexSummary: indexController !== null ? indexController.summary() : ""
    onLogsRequested: shellController.toggleBottomTab("logs")
    onCancelBuildRequested: coreClient.cancelBuild()
    onCancelTestsRequested: coreClient.cancelTests()
    onCancelQualityRequested: coreClient.cancelQuality()
    onCancelEnvironmentScanRequested: coreClient.cancelEnvironmentScan()
}
