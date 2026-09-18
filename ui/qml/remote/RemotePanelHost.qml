pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel do alvo remoto dentro de um dialogo, como os outros overlays de
// ambiente. Separa o CONTEUDO (RemotePanel, burro) do CHROME (fundo, moldura,
// dispensar por clique fora). Mesmo desenho do DataSourcePanelHost.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520

    signal dismissRequested()

    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: moldura

        anchors.centerIn: parent
        width: Math.min(680, root.maxAvailableWidth)
        height: Math.min(520, root.maxAvailableHeight)
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        RemotePanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium

            targets: root.controller ? root.controller.targets : []
            selectedName: root.controller ? root.controller.selectedName : ""
            selectedSaved: root.controller ? root.controller.selectedSaved : false
            draft: root.controller ? root.controller.draft : null
            program: root.controller ? root.controller.program : ""
            deploySource: root.controller ? root.controller.deploySource : ""
            errorText: root.controller ? root.controller.errorText : ""
            probing: root.controller ? root.controller.probing : false
            probeOk: root.controller ? root.controller.probeOk : false
            probeArch: root.controller ? root.controller.probeArch : ""
            probeKernel: root.controller ? root.controller.probeKernel : ""
            probeTools: root.controller ? root.controller.probeTools : []
            probeMessage: root.controller ? root.controller.probeMessage : ""
            deploying: root.controller ? root.controller.deploying : false
            deployMessage: root.controller ? root.controller.deployMessage : ""
            lastCommand: root.controller ? root.controller.lastCommand : ""
            lastOutcome: root.controller ? root.controller.lastOutcome : ""
            openPath: root.controller ? root.controller.openPath : ""
            mirror: root.controller ? root.controller.mirror : null
            isMirror: root.controller ? root.controller.isMirror : false
            syncing: root.controller ? root.controller.syncing : false
            syncMessage: root.controller ? root.controller.syncMessage : ""

            onTargetSelected: name => root.controller.select(name)
            onNewRequested: root.controller.startNew()
            onFieldEdited: (field, value) => root.controller.editDraft(field, value)
            onProgramEdited: text => root.controller.program = text
            onDeploySourceEdited: text => root.controller.deploySource = text
            onSaveRequested: root.controller.save()
            onRemoveRequested: root.controller.remove()
            onProbeRequested: root.controller.probe()
            onDeployRequested: root.controller.deploy()
            onCommandRequested: kind => root.controller.requestCommand(kind)
            onOpenPathEdited: text => root.controller.openPath = text
            onOpenFolderRequested: root.controller.openFolder()
            onSyncRequested: direction => root.controller.sync(direction)
            onCloseRequested: root.dismissRequested()
        }
    }
}
