pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel do alvo remoto na moldura comum (KvPanelFrame, F8): o CONTEUDO
// (RemotePanel, burro) separado do CHROME.
KvPanelFrame {
    id: root

    property var controller: null

    panelWidth: 680
    panelHeight: 520

    RemotePanel {
        anchors.fill: parent

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
        discovery: root.controller ? root.controller.discovery : "idle"
        discovering: root.controller ? root.controller.discovering : false
        aliases: root.controller ? root.controller.aliases : []
        aliasSources: root.controller ? root.controller.aliasSources : []
        resolving: root.controller ? root.controller.resolving : ""
        resolved: root.controller ? root.controller.resolved : null

        onDiscoverRequested: root.controller.discover()
        // Escolher um alias faz as DUAS coisas que a fatia promete: cria o
        // alvo sem redigitar nada e pergunta ao ssh o que ele faria.
        onAliasChosen: name => { root.controller.useAlias(name); root.controller.resolve(name); }
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
