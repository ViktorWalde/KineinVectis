pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O alvo Linux por SSH na tela: escolher, editar, salvar, SONDAR, enviar e
// virar configuracao de execucao/kit. Componente burro: recebe por property,
// pede por signal. "Sondar" e' a acao primaria — no cabecalho comum (F8):
// e' o que prova a chave, a rede e o que a placa tem antes de qualquer
// deploy. O veredito vem ANTES do formulario.
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
    // "Usar o SSH que ja' funciona" (R0.5): vem do core, nao de palpite.
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
    signal probeRequested()
    signal deployRequested()
    signal commandRequested(string kind)
    signal openPathEdited(string text)
    signal openFolderRequested()
    signal syncRequested(string direction)
    signal closeRequested()

    readonly property bool draftNamed: root.draft !== null && root.draft.name.trim() !== ""

    // A primeira linha comum dos paineis de ambiente (F8): titulo, uma
    // linha, a acao primaria — "Sondar" — e o x.
    KvPanelHeader {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: qsTr("Alvo remoto (Linux por SSH)")
        subtitle: qsTr("Raspberry Pi ou placa com imagem própria: sondar, enviar (rsync), rodar e depurar (gdbserver/debugpy) pelo ssh do sistema.")
        primaryLabel: qsTr("Sondar")
        primaryEnabled: root.selectedSaved
        primaryBusy: root.probing
        onPrimaryRequested: root.probeRequested()
        onCloseRequested: root.closeRequested()
    }

    RemoteList {
        id: lista

        anchors.top: cabecalho.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.bottom: acoes.top
        anchors.bottomMargin: Theme.spacingSmall
        width: Math.round(parent.width * 0.32)

        targets: root.targets
        selectedName: root.selectedName

        onTargetSelected: name => root.targetSelected(name)
        onNewRequested: root.newRequested()
    }

    Flickable {
        id: rolagem

        anchors.top: lista.top
        anchors.left: lista.right
        anchors.leftMargin: Theme.spacingMedium
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
                probing: root.probing
                probeOk: root.probeOk
                probeArch: root.probeArch
                probeKernel: root.probeKernel
                probeTools: root.probeTools
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

            // ANTES do formulario, de proposito: se o `ssh <alias>` ja'
            // funciona, o primeiro uso nao deve comecar por campo em branco.
            RemoteDiscovery {
                width: parent.width
                discovery: root.discovery
                discovering: root.discovering
                aliases: root.aliases
                aliasSources: root.aliasSources
                resolving: root.resolving
                resolved: root.resolved
                onRefreshRequested: root.discoverRequested()
                onAliasChosen: name => root.aliasChosen(name)
            }

            RemoteForm {
                width: parent.width
                draft: root.draft
                program: root.program
                deploySource: root.deploySource
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
                onProgramEdited: text => root.programEdited(text)
                onDeploySourceEdited: text => root.deploySourceEdited(text)
            }

            // O workspace ESPELHADO (fatia 2): a pasta do alvo vira espelho
            // local por rsync; salvar empurra; Puxar/Empurrar sincronizam.
            RemoteMirrorView {
                width: parent.width
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

            // O ciclo depois de salvo: deploy -> rodar -> depurar. Cada botao
            // pede ao core a linha; o controller a leva ao dono certo.
            Flow {
                width: parent.width
                spacing: Theme.spacingSmall

                KvButton {
                    text: qsTr("Enviar (deploy)")
                    compact: true
                    enabled: root.selectedSaved && !root.deploying
                    onClicked: root.deployRequested()
                }

                KvButton {
                    text: qsTr("Rodar em… → config")
                    compact: true
                    enabled: root.selectedSaved
                    onClicked: root.commandRequested("run")
                }

                KvButton {
                    text: qsTr("gdbserver → kit")
                    compact: true
                    enabled: root.selectedSaved
                    onClicked: root.commandRequested("debugServer")
                }

                KvButton {
                    text: qsTr("debugpy → config")
                    compact: true
                    enabled: root.selectedSaved
                    onClicked: root.commandRequested("debugpy")
                }

                KvButton {
                    text: qsTr("Shell no terminal")
                    compact: true
                    enabled: root.selectedSaved
                    onClicked: root.commandRequested("shell")
                }
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
