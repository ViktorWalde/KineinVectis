pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A seccao EXECUTAR (§5.1): o ciclo depois de o alvo existir — enviar, rodar,
// depurar e abrir shell. Cada botao pede ao core a LINHA; o controller a leva
// ao dono certo (configuracao de execucao, kit ou terminal).
//
// Estes dois campos moraram no formulario de PERFIL ate' 2026-09-24. Eles nao
// sao perfil: perfil e' como CHEGAR no alvo, e isto e' o que RODAR la'. Mistura-
// los foi parte do que a §3 chama de "configuracao rara e operacao frequente
// disputando a mesma coluna".
//
// O que a sonda mediu muda o que os botoes DIZEM: oferecer "gdbserver → kit"
// sem dizer que o alvo nao tem `gdbserver` e' deixar a pessoa descobrir depois.
Item {
    id: root

    property bool ready: false
    property bool deploying: false
    property string program: ""
    property string deploySource: ""
    property var probeTools: []
    property bool probed: false
    // O retorno dos gestos AQUI MESMO (2026-10-04: enviar nao dizia nada na
    // seccao; o resultado so' aparecia na Visao).
    property string deployMessage: ""
    property bool deployOk: false
    property string lastOutcome: ""

    signal programEdited(string text)
    signal deploySourceEdited(string text)
    signal deployRequested()
    signal commandRequested(string kind)

    implicitHeight: coluna.implicitHeight

    function temFerramenta(id) {
        for (let i = 0; i < root.probeTools.length; i++) {
            if (root.probeTools[i].id === id) {
                return root.probeTools[i].found === true;
            }
        }
        return false;
    }

    // Sem sonda nao se afirma falta: "nao medi" e "nao tem" sao coisas
    // diferentes, e so' a segunda justifica um aviso.
    function falta(id) {
        return root.probed && !root.temFerramenta(id);
    }

    // Sem alvo salvo, nada aqui roda: o motivo vai no topo e em cada linha.
    readonly property string blockedReason: qsTr("salve um alvo em Configurar primeiro")

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        // Os dois campos EMPILHADOS (2026-10-04): lado a lado, numa janela
        // acoplada, os rotulos longos se sobrepunham.
        DataSourceField {
            width: parent.width
            label: qsTr("Origem do deploy (vazio = build/)")
            placeholder: "build/app"
            value: root.deploySource
            onEdited: text => root.deploySourceEdited(text)
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Programa no alvo (relativo = na pasta de deploy)")
            placeholder: "app, main.py, /opt/app/bin"
            value: root.program
            onEdited: text => root.programEdited(text)
        }

        Text {
            topPadding: Theme.spacingXSmall
            text: qsTr("AÇÕES")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeMicro
            font.weight: Font.DemiBold
            font.letterSpacing: 0.6
        }

        RemoteActionRow {
            width: parent.width
            iconName: "push"
            title: qsTr("Enviar para o alvo")
            detail: !root.ready ? root.blockedReason
                    : root.falta("rsync") ? qsTr("sem rsync no alvo: vai por scp (mais lento, sem --delete)")
                    : qsTr("%1 → pasta de deploy (rsync)").arg(root.deploySource !== "" ? root.deploySource : "build/")
            caution: root.ready && root.falta("rsync")
            available: root.ready && !root.deploying
            busy: root.deploying
            onTriggered: root.deployRequested()
        }

        RemoteActionRow {
            width: parent.width
            iconName: "run"
            title: qsTr("Rodar no alvo")
            detail: root.ready ? qsTr("cria a configuração de execução “Rodar em …”") : root.blockedReason
            available: root.ready
            onTriggered: root.commandRequested("run")
        }

        RemoteActionRow {
            width: parent.width
            iconName: "debug"
            title: qsTr("Depurar C/C++ (gdbserver)")
            detail: !root.ready ? root.blockedReason
                    : root.falta("gdbserver") ? qsTr("falta gdbserver no alvo: instale antes de depurar")
                    : qsTr("preenche o kit com o alvo e o gdbserver")
            caution: root.ready && root.falta("gdbserver")
            available: root.ready
            onTriggered: root.commandRequested("debugServer")
        }

        RemoteActionRow {
            width: parent.width
            iconName: "debug"
            title: qsTr("Depurar Python (debugpy)")
            detail: !root.ready ? root.blockedReason
                    : root.falta("python3") ? qsTr("o alvo não tem python3")
                    : qsTr("cria a configuração com debugpy no alvo")
            caution: root.ready && root.falta("python3")
            available: root.ready
            onTriggered: root.commandRequested("debugpy")
        }

        RemoteActionRow {
            width: parent.width
            iconName: "terminal"
            title: qsTr("Abrir shell no terminal")
            detail: root.ready ? qsTr("ssh no painel de baixo; a senha, se houver, pergunta lá") : root.blockedReason
            available: root.ready
            onTriggered: root.commandRequested("shell")
        }

        // O resultado do ultimo gesto: enviando, enviado, falhou, configurado.
        Rectangle {
            width: parent.width
            visible: root.deploying || root.deployMessage !== "" || root.lastOutcome !== ""
            height: resultColumn.implicitHeight + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: !root.deploying && root.deployMessage !== "" && !root.deployOk ? Theme.errorSoft : Theme.borderSoft

            Column {
                id: resultColumn

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                spacing: 2

                Row {
                    visible: root.deploying || root.deployMessage !== ""
                    width: parent.width
                    spacing: Theme.spacingSmall

                    KvIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        name: root.deploying ? "refresh" : (root.deployOk ? "check" : "warning")
                        size: 13
                        success: !root.deploying && root.deployOk
                        error: !root.deploying && !root.deployOk
                    }

                    Text {
                        width: parent.width - 13 - parent.spacing
                        text: root.deploying ? qsTr("Enviando…") : root.deployMessage
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeCaption
                        wrapMode: Text.WordWrap
                    }
                }

                Text {
                    width: parent.width
                    visible: root.lastOutcome !== ""
                    text: root.lastOutcome
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
