import QtQuick
import KineinVectis

WorkspaceStatusBar {
    id: root

    property var coreClient: null
    property var shellController: null
    property var toolchainController: null
    property var indexController: null
    property var pythonController: null
    property var activeJobController: null
    property var lspStatusController: null

    workspaceRoot: coreClient.workspaceRoot
    workspaceKindLabel: shellController.kindLabel(
                            coreClient.workspaceKind,
                            coreClient.workspaceBuildSystems)
    logsActive: shellController.tabActive("logs")
    running: coreClient.running
    coreConnected: coreClient.connected
    coreProtocolVersion: coreClient.protocolVersion
    coreStatus: coreClient.status
    toolchainVisible: coreClient.workspaceRoot !== ""
                      && coreClient.workspaceBuildSystems.length > 0
    toolchainSummary: toolchainController.summary(coreClient.workspaceBuildSystems)
    indexSummary: indexController !== null ? indexController.summary() : ""
    contextSummary: indexController !== null ? indexController.contextSummary() : ""
    contextDetail: indexController !== null ? indexController.contextDetail() : ""
    pythonSummary: pythonController !== null ? pythonController.summary() : ""
    jobTitle: activeJobController !== null ? activeJobController.title : ""
    jobProgress: activeJobController !== null ? activeJobController.progress : -1
    jobMessage: activeJobController !== null ? activeJobController.message : ""
    jobCanCancel: activeJobController !== null ? activeJobController.canCancel : false
    jobCount: activeJobController !== null ? activeJobController.runningCount : 0
    lspSummary: lspStatusController !== null ? lspStatusController.summary() : ""
    lspDetail: lspStatusController !== null ? lspStatusController.detail() : ""
    lspFailed: lspStatusController !== null ? lspStatusController.hasFailure() : false
    onLogsRequested: shellController.toggleBottomTab("logs")
    onJobsRequested: shellController.showTab("jobs")
    onCancelJobRequested: activeJobController.cancel()
}
