import QtQuick
import KineinVectis

WorkspaceStatusBar {
    id: root

    property var coreClient: null
    property var shellController: null
    property var indexController: null
    property var activeJobController: null
    property var lspStatusController: null
    property var editorController: null
    property var remoteController: null

    workspaceRoot: coreClient.workspaceRoot
    workspaceName: coreClient.workspaceName
    breadcrumb: editorController !== null && editorController.currentTab >= 0
                && !editorController.currentReadOnly
                ? shellController.relativeToRoot(editorController.currentFilePath()) : ""
    logsActive: shellController.tabActive("logs")
    running: coreClient.running
    coreConnected: coreClient.connected
    coreProtocolVersion: coreClient.protocolVersion
    coreStatus: coreClient.status
    indexSummary: indexController !== null ? indexController.summary() : ""
    contextSummary: indexController !== null ? indexController.contextSummary() : ""
    contextDetail: indexController !== null ? indexController.contextDetail() : ""
    jobTitle: activeJobController !== null ? activeJobController.title : ""
    jobProgress: activeJobController !== null ? activeJobController.progress : -1
    jobMessage: activeJobController !== null ? activeJobController.message : ""
    jobCanCancel: activeJobController !== null ? activeJobController.canCancel : false
    jobCount: activeJobController !== null ? activeJobController.runningCount : 0
    // So' com arquivo aberto: a tela inicial mostrava "1:1" sem editor (F0).
    cursorSummary: editorController !== null && editorController.currentTab >= 0
                   ? editorController.cursorSummary : ""
    lspSummary: lspStatusController !== null ? lspStatusController.summary() : ""
    lspDetail: lspStatusController !== null ? lspStatusController.detail() : ""
    lspFailed: lspStatusController !== null ? lspStatusController.hasFailure() : false
    remoteIsMirror: remoteController !== null && remoteController.workspace.isMirror
    remoteTarget: remoteController !== null && remoteController.workspace.mirror
                  ? remoteController.workspace.mirror.name : ""
    remoteSyncing: remoteController !== null && remoteController.workspace.syncing
    remoteSyncDirection: remoteController !== null ? remoteController.workspace.syncDirection : ""
    remoteSyncFailed: remoteController !== null && remoteController.workspace.syncFailed
    remoteSyncMessage: remoteController !== null ? remoteController.workspace.syncMessage : ""
    remoteDeploying: remoteController !== null && remoteController.deploying
    remoteProbed: remoteController !== null && remoteController.probedAt > 0
    remoteProbeOk: remoteController !== null && remoteController.probeOk
    remoteProbedAt: remoteController !== null ? remoteController.probedAt : 0
    leftOrder: shellController.savedOrder("statusLeft")
    rightOrder: shellController.savedOrder("statusRight")
    onItemMoved: (strip, key, dropIndex, visibleKeys) => shellController.moveInBar(
                     strip, visibleKeys, key, dropIndex)
    onRemotePanelRequested: remoteController.open()
    onLogsRequested: shellController.toggleBottomTab("logs")
    onJobsRequested: shellController.showTab("jobs")
    onCancelJobRequested: activeJobController.cancel()
}
