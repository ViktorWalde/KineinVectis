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

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: Math.round((parent.width - parent.spacing) / 2)
                label: qsTr("Origem do deploy (vazio = build/)")
                placeholder: "build/app"
                value: root.deploySource
                onEdited: text => root.deploySourceEdited(text)
            }

            DataSourceField {
                width: Math.round((parent.width - parent.spacing) / 2)
                label: qsTr("Programa no alvo (relativo = na pasta de deploy)")
                placeholder: "app, main.py, /opt/app/bin"
                value: root.program
                onEdited: text => root.programEdited(text)
            }
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingSmall

            KvButton {
                text: qsTr("Enviar (deploy)")
                compact: true
                enabled: root.ready && !root.deploying
                onClicked: root.deployRequested()
            }

            KvButton {
                text: qsTr("Rodar em… → config")
                compact: true
                enabled: root.ready
                onClicked: root.commandRequested("run")
            }

            KvButton {
                text: root.falta("gdbserver")
                      ? qsTr("gdbserver → kit (falta no alvo)")
                      : qsTr("gdbserver → kit")
                compact: true
                enabled: root.ready
                onClicked: root.commandRequested("debugServer")
            }

            KvButton {
                text: root.falta("python3")
                      ? qsTr("debugpy → config (sem python3)")
                      : qsTr("debugpy → config")
                compact: true
                enabled: root.ready
                onClicked: root.commandRequested("debugpy")
            }

            KvButton {
                text: qsTr("Shell no terminal")
                compact: true
                enabled: root.ready
                onClicked: root.commandRequested("shell")
            }
        }

        Text {
            width: parent.width
            visible: root.probed && (root.falta("gdbserver") || root.falta("rsync"))
            wrapMode: Text.WordWrap
            text: {
                const partes = [];
                if (root.falta("gdbserver")) {
                    partes.push(qsTr("sem gdbserver, o alvo não depura C/C++ remoto"));
                }
                if (root.falta("rsync")) {
                    partes.push(qsTr("sem rsync, o envio cai para scp (mais lento, sem --delete)"));
                }
                return partes.join(" · ");
            }
            color: Theme.textMuted
            font.pixelSize: 9
        }
    }
}
