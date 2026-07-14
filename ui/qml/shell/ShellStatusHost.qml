import QtQuick
import KineinVectis

WorkspaceStatusBar {
    id: root

    property var coreClient: null
    property var shellController: null
    property var gitController: null

    workspaceRoot: coreClient.workspaceRoot
    workspaceKindLabel: shellController.kindLabel(coreClient.workspaceKind)
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
    onLogsRequested: shellController.toggleBottomTab("logs")
    onCancelBuildRequested: coreClient.cancelBuild()
    onCancelTestsRequested: coreClient.cancelTests()
    onCancelQualityRequested: coreClient.cancelQuality()
    onCancelEnvironmentScanRequested: coreClient.cancelEnvironmentScan()
}
