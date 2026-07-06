import QtQuick
import KineinVectis

TopHeaderBar {
    id: root

    property var coreClient: null
    property var shellController: null
    property var jobsController: null
    property var runtimeController: null

    workspaceOpen: coreClient.workspaceRoot !== ""
    coreConnected: coreClient.connected
    building: coreClient.building
    testing: coreClient.testing
    analyzing: coreClient.analyzing
    running: coreClient.running
    onOpenWorkspaceRequested: shellController.requestOpenFolder()
    onBuildRequested: jobsController.startBuild()
    onTestsRequested: jobsController.startTests()
    onQualityRequested: jobsController.startQuality()
    onRunRequested: runtimeController.startRun("")
    onStopRunRequested: runtimeController.stopRun()
}
