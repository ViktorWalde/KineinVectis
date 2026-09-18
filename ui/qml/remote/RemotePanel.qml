pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O alvo Linux por SSH na tela: escolher, editar, salvar, SONDAR, enviar e
// virar configuracao de execucao/kit. Componente burro: recebe por property,
// pede por signal. "Sondar" e' a acao primaria: e' o que prova a chave, a
// rede e o que a placa tem antes de qualquer deploy.
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
    signal closeRequested()

    readonly property bool draftNamed: root.draft !== null && root.draft.name.trim() !== ""

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Alvo remoto (Linux por SSH)")
        color: Theme.textPrimary
        font.pixelSize: 12
        font.weight: Font.DemiBold
    }

    Text {
        id: subtitulo

        anchors.top: titulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("Raspberry Pi ou placa com imagem própria: sondar, enviar (rsync), "
                   + "rodar e depurar (gdbserver/debugpy) pelo ssh do sistema.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    RemoteList {
        id: lista

        anchors.top: subtitulo.bottom
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

            RemoteForm {
                width: parent.width
                draft: root.draft
                program: root.program
                deploySource: root.deploySource
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
                onProgramEdited: text => root.programEdited(text)
                onDeploySourceEdited: text => root.deploySourceEdited(text)
            }

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
            text: qsTr("Fechar")
            compact: true
            onClicked: root.closeRequested()
        }

        KvButton {
            text: qsTr("Sondar")
            primary: true
            compact: true
            enabled: root.selectedSaved && !root.probing
            onClicked: root.probeRequested()
        }

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
