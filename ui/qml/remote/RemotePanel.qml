pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O alvo Linux por SSH na tela, em SECOES (fatia R1/V2, 2026-09-24).
//
// Ate' aqui era uma coluna so', rolando: veredito, descoberta, servidor novo,
// seis campos de perfil, espelho e cinco botoes de execucao. A §3 da
// especificacao ja' diagnosticava — "configuracao rara e operacao frequente
// disputam a mesma coluna", "todas as acoes aparecem com peso parecido" — e a
// foto de 2026-09-24 mostrou isso piorando: as duas entradas novas da R0.5
// empurraram Enviar/Rodar/depurar para FORA da tela.
//
// Agora: uma seccao por vez (§5.1 — "nao devem ser cinco cards longos na mesma
// rolagem") e UMA acao primaria por estado no cabecalho, derivada por regra
// pura no `RemoteActionRules`. Componente burro: recebe por property, pede por
// signal.
Item {
    id: root

    property var targets: []
    property string selectedName: ""
    property bool selectedSaved: false
    property var draft: null
    property string program: ""
    property string deploySource: ""
    property string errorText: ""
    property bool probing: false
    property bool probeOk: false
    property string probedName: ""
    property string probeArch: ""
    property string probeKernel: ""
    property var probeTools: []
    property string probeMessage: ""
    property bool deploying: false
    property string deployMessage: ""
    property string lastCommand: ""
    property string lastOutcome: ""
    property string openPath: ""
    property var mirror: null
    property bool isMirror: false
    property bool syncing: false
    property string syncMessage: ""
    property string discovery: "idle"
    property bool discovering: false
    property var aliases: []
    property var aliasSources: []
    property string resolving: ""
    property var resolved: null
    property string probeFailure: ""
    property bool canCopyId: false
    property string armedCommand: ""
    property string armedName: ""
    property string pasted: ""
    property bool canParse: false
    property var proposalSource: []
    // Qual seccao esta' aberta; o dono e' o controller.
    property string section: "visao"

    signal sectionSelected(string id)
    signal primaryRequested(string kind)
    signal pastedEdited(string text)
    signal parseRequested()
    signal discoverRequested()
    signal aliasChosen(string name)
    signal copyIdRequested()
    signal runArmedRequested()
    signal disarmRequested()
    signal targetSelected(string name)
    signal newRequested()
    signal fieldEdited(string field, var value)
    signal programEdited(string text)
    signal deploySourceEdited(string text)
    signal saveRequested()
    signal removeRequested()
    signal deployRequested()
    signal commandRequested(string kind)
    signal openPathEdited(string text)
    signal openFolderRequested()
    signal syncRequested(string direction)
    signal closeRequested()

    readonly property bool draftNamed: root.draft !== null && root.draft.name.trim() !== ""
    readonly property bool probed: root.probedName !== ""

    RemoteActionRules {
        id: regras
    }

    // O proximo gesto, um so', derivado do estado medido.
    readonly property var acao: regras.primaryFor({
        "temAlvoSalvo": root.selectedSaved,
        "rascunhoNomeado": root.draftNamed,
        "sondando": root.probing,
        "sondou": root.probed,
        "sondaOk": root.probeOk,
        "falha": root.probeFailure,
        "eEspelho": root.isMirror,
        "sincronizando": root.syncing,
        "temPastaRemota": root.openPath.trim() !== ""
    })

    KvPanelHeader {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: qsTr("Alvo remoto (Linux por SSH)")
        // O subtitulo diz POR QUE este e' o proximo passo. Acao primaria sem
        // motivo vira adivinhacao.
        subtitle: root.acao.hint
        primaryLabel: root.acao.label
        primaryEnabled: root.acao.enabled
        primaryBusy: root.acao.busy
        onPrimaryRequested: root.primaryRequested(root.acao.kind)
        onCloseRequested: root.closeRequested()
    }

    RemoteList {
        id: lista

        anchors.top: cabecalho.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.bottom: acoes.top
        anchors.bottomMargin: Theme.spacingSmall
        width: Math.round(parent.width * 0.28)

        targets: root.targets
        selectedName: root.selectedName

        onTargetSelected: name => root.targetSelected(name)
        onNewRequested: root.newRequested()
    }

    RemoteSections {
        id: faixa

        anchors.top: lista.top
        anchors.left: lista.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: parent.right

        current: root.section
        sections: [
            { "id": "visao", "label": qsTr("Visão geral") },
            { "id": "workspace", "label": qsTr("Workspace") },
            { "id": "executar", "label": qsTr("Executar") },
            { "id": "sistema", "label": qsTr("Sistema") },
            { "id": "configurar", "label": qsTr("Configurar") }
        ]
        onSelected: id => root.sectionSelected(id)
    }

    Flickable {
        id: rolagem

        anchors.top: faixa.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: faixa.left
        anchors.right: parent.right
        anchors.bottom: lista.bottom
        clip: true
        contentWidth: width
        contentHeight: conteudo.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: conteudo

            width: rolagem.width
            spacing: Theme.spacingSmall

            RemoteVerdict {
                width: parent.width
                visible: root.section === "visao"
                hasTarget: root.selectedSaved
                probed: root.probed
                probing: root.probing
                probeOk: root.probeOk
                probeArch: root.probeArch
                probeKernel: root.probeKernel
                probeMessage: root.probeMessage
                deploying: root.deploying
                deployMessage: root.deployMessage
                lastCommand: root.lastCommand
                lastOutcome: root.lastOutcome
                errorText: root.errorText
                canCopyId: root.canCopyId
                armedCommand: root.armedCommand
                armedName: root.armedName
                onCopyIdRequested: root.copyIdRequested()
                onRunArmedRequested: root.runArmedRequested()
                onDisarmRequested: root.disarmRequested()
            }

            RemoteMirrorView {
                width: parent.width
                visible: root.section === "workspace"
                openPath: root.openPath
                mirror: root.mirror
                isMirror: root.isMirror
                canOpen: root.selectedSaved && !root.syncing
                syncing: root.syncing
                syncMessage: root.syncMessage
                onOpenPathEdited: text => root.openPathEdited(text)
                onOpenFolderRequested: root.openFolderRequested()
                onSyncRequested: direction => root.syncRequested(direction)
            }

            RemoteRunActions {
                width: parent.width
                visible: root.section === "executar"
                ready: root.selectedSaved
                deploying: root.deploying
                program: root.program
                deploySource: root.deploySource
                probeTools: root.probeTools
                probed: root.probed
                onProgramEdited: text => root.programEdited(text)
                onDeploySourceEdited: text => root.deploySourceEdited(text)
                onDeployRequested: root.deployRequested()
                onCommandRequested: kind => root.commandRequested(kind)
            }

            RemoteSystemView {
                width: parent.width
                visible: root.section === "sistema"
                probed: root.probed && root.probeOk
                probeArch: root.probeArch
                probeKernel: root.probeKernel
                probeTools: root.probeTools
            }

            // Em Configurar, os dois caminhos da R0.5 vem ANTES do formulario:
            // se o `ssh <alias>` ja' funciona, o primeiro uso nao deve comecar
            // por campo em branco.
            RemoteDiscovery {
                width: parent.width
                visible: root.section === "configurar"
                discovery: root.discovery
                discovering: root.discovering
                aliases: root.aliases
                aliasSources: root.aliasSources
                resolving: root.resolving
                resolved: root.resolved
                onRefreshRequested: root.discoverRequested()
                onAliasChosen: name => root.aliasChosen(name)
            }

            RemoteNewHost {
                width: parent.width
                visible: root.section === "configurar"
                pasted: root.pasted
                canParse: root.canParse
                proposalSource: root.proposalSource
                errorText: root.errorText
                onPastedEdited: text => root.pastedEdited(text)
                onParseRequested: root.parseRequested()
            }

            RemoteForm {
                width: parent.width
                visible: root.section === "configurar"
                draft: root.draft
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
            }
        }
    }

    Row {
        id: acoes

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall
        layoutDirection: Qt.RightToLeft

        KvButton {
            text: qsTr("Salvar")
            compact: true
            enabled: root.draftNamed
            onClicked: root.saveRequested()
        }

        KvButton {
            text: qsTr("Remover")
            compact: true
            enabled: root.selectedName !== ""
            onClicked: root.removeRequested()
        }
    }
}
