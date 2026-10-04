pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O conteudo da seccao aberta do Remoto, ligado direto ao RemoteController
// (2026-10-04: a janela acoplada e' o host; o RemotePanel/RemotePanelHost do
// pop-up repassavam 40 propriedades de um para o outro e sairam). As partes
// continuam burras: cada uma recebe por property e pede por sinal.
Flickable {
    id: root

    property var controller: null

    readonly property var c: root.controller
    readonly property string section: root.c ? root.c.section : "visao"
    readonly property bool probed: root.c !== null && root.c.probedName !== ""

    clip: true
    contentWidth: width
    contentHeight: pages.implicitHeight
    boundsBehavior: Flickable.StopAtBounds
    // A rolagem NUNCA fica alem do fim (2026-10-04, achado na tela): o
    // conteudo encolheu (a descoberta do ~/.ssh/config terminou) com a pagina
    // rolada, e fora dos limites o Flickable gastava cada clique em "parar o
    // movimento" — o campo do comando ssh nao recebia o foco. Trocar de
    // seccao volta ao topo.
    onContentHeightChanged: root.returnToBounds()
    onSectionChanged: root.contentY = 0

    Column {
        id: pages

        width: root.width
        spacing: Theme.spacingSmall

        RemoteVerdict {
            width: parent.width
            visible: root.section === "visao"
            hasTarget: root.c !== null && root.c.selectedSaved
            lastContact: root.c !== null && root.c.contacts.state(root.c.selectedName) !== ""
                         ? root.c.contacts.describe(root.c.selectedName, root.c.contacts.nowSeconds) : ""
            probed: root.probed
            probing: root.c !== null && root.c.probing
            probeOk: root.c !== null && root.c.probeOk
            probeArch: root.c ? root.c.probeArch : ""
            probeKernel: root.c ? root.c.probeKernel : ""
            probeMessage: root.c ? root.c.probeMessage : ""
            deploying: root.c !== null && root.c.deploying
            deployMessage: root.c ? root.c.deployMessage : ""
            lastCommand: root.c ? root.c.lastCommand : ""
            lastOutcome: root.c ? root.c.lastOutcome : ""
            errorText: root.c ? root.c.errorText : ""
            canCopyId: root.c !== null && root.c.canCopyId
            armedCommand: root.c ? root.c.armedCommand : ""
            armedName: root.c ? root.c.armedName : ""
            onCopyIdRequested: root.c.copyId()
            onRunArmedRequested: root.c.runArmed()
            onDisarmRequested: root.c.disarm()
        }

        RemoteMirrorView {
            width: parent.width
            visible: root.section === "workspace"
            openPath: root.c ? root.c.workspace.openPath : ""
            mirror: root.c ? root.c.workspace.mirror : null
            isMirror: root.c !== null && root.c.workspace.isMirror
            canOpen: root.c !== null && root.c.selectedSaved && !root.c.workspace.syncing
            syncing: root.c !== null && root.c.workspace.syncing
            syncMessage: root.c ? root.c.workspace.syncMessage : ""
            browseState: root.c ? root.c.workspace : null
            onOpenPathEdited: text => { root.c.workspace.openPath = text; }
            onOpenFolderRequested: root.c.workspace.openFolder()
            onBrowseAction: (kind, path) => {
                switch (kind) {
                case "start": root.c.workspace.startBrowse(); break;
                case "navigate": root.c.workspace.browseTo(path); break;
                case "open": root.c.workspace.openBrowsedFolder(); break;
                }
            }
            onSyncRequested: direction => root.c.workspace.sync(direction)
        }

        RemoteRunActions {
            width: parent.width
            visible: root.section === "executar"
            ready: root.c !== null && root.c.selectedSaved
            deploying: root.c !== null && root.c.deploying
            program: root.c ? root.c.program : ""
            deploySource: root.c ? root.c.deploySource : ""
            probeTools: root.c ? root.c.probeTools : []
            // "Falta no alvo" so' com uma sonda que RESPONDEU: a que falhou
            // nao mediu nada (achado na tela, 2026-10-04).
            probed: root.probed && root.c.probeOk
            deployMessage: root.c ? root.c.deployMessage : ""
            deployOk: root.c !== null && root.c.deployOk
            lastOutcome: root.c ? root.c.lastOutcome : ""
            onProgramEdited: text => { root.c.program = text; }
            onDeploySourceEdited: text => { root.c.deploySource = text; }
            onDeployRequested: root.c.deploy()
            onCommandRequested: kind => root.c.requestCommand(kind)
        }

        RemoteSystemView {
            width: parent.width
            visible: root.section === "sistema"
            probed: root.probed && root.c.probeOk
            failed: root.probed && !root.c.probeOk && !root.c.probing
            probeArch: root.c ? root.c.probeArch : ""
            probeKernel: root.c ? root.c.probeKernel : ""
            probeTools: root.c ? root.c.probeTools : []
        }

        // Em Configurar, os dois caminhos da R0.5 vem ANTES do formulario: se
        // o `ssh <alias>` ja' funciona, o primeiro uso nao deve comecar por
        // campo em branco.
        RemoteDiscovery {
            width: parent.width
            visible: root.c !== null && root.c.configuring
            discovery: root.c ? root.c.setup.discovery : "idle"
            discovering: root.c !== null && root.c.setup.discovering
            aliases: root.c ? root.c.setup.aliases : []
            aliasSources: root.c ? root.c.setup.aliasSources : []
            resolving: root.c ? root.c.setup.resolving : ""
            resolved: root.c ? root.c.setup.resolved : null
            onRefreshRequested: root.c.setup.discover()
            // Escolher um alias cria o alvo sem redigitar nada e pergunta ao
            // ssh o que ele faria.
            onAliasChosen: name => { root.c.useAlias(name); root.c.setup.resolve(name); }
        }

        RemoteNewHost {
            width: parent.width
            visible: root.c !== null && root.c.configuring
            pasted: root.c ? root.c.setup.pasted : ""
            canParse: root.c !== null && root.c.setup.canParse
            proposalSource: root.c ? root.c.setup.proposalSource : []
            errorText: root.c ? root.c.errorText : ""
            onPastedEdited: text => { root.c.setup.pasted = text; }
            onParseRequested: root.c.setup.parse()
        }

        RemoteForm {
            width: parent.width
            visible: root.c !== null && root.c.configuring
            draft: root.c ? root.c.draft : null
            onFieldEdited: (field, value) => root.c.editDraft(field, value)
        }
    }
}
