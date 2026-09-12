pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de containers: o MOTOR (a tela de "ativar a ferramenta"), os
// containers com as acoes de ciclo de vida, as imagens, e o compose do projeto.
//
// Burro de proposito, como o EmbeddedPanel: recebe estado e emite pedidos.
// Nada aqui chama `docker`/`podman` — e' o core que sobe processo, e e' ele
// que diz qual dos dois respondeu (roadmaps/28 §4).
Item {
    id: root

    property var controller: null

    signal closeRequested()

    Column {
        id: coluna

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Containers")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            // O QUE A TELA PROMETE E' O QUE ELA FAZ.
            text: qsTr("Docker ou Podman, o que responder nesta máquina. Parar, iniciar e remover são jobs canceláveis; logs e shell abrem numa aba do terminal. Nada roda como root.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.controller ? root.controller.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        // --- Motor: a tela de ATIVAR ------------------------------------
        Row {
            spacing: Theme.spacingSmall

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 8
                height: 8
                radius: 4
                color: !root.controller || !root.controller.engineFound ? Theme.textDisabled
                       : (root.controller.reachable ? Theme.successSoft : Theme.errorSoft)
            }

            Text {
                text: root.controller && root.controller.engineFound
                      ? qsTr("Motor: %1").arg(root.controller.engineLabel)
                      : (root.controller && root.controller.statusBusy ? qsTr("procurando o motor…")
                                                                        : qsTr("nenhum motor de container"))
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && root.controller.engineFound
            text: {
                if (!root.controller || !root.controller.engineFound) return "";
                const s = root.controller.status;
                const partes = [];
                partes.push(s.reachable ? qsTr("responde") : qsTr("NÃO responde"));
                if (s.rootless === true) partes.push("rootless");
                if (s.rootless === false) partes.push(qsTr("com daemon/root"));
                if (s.socket !== undefined && s.socket !== "") partes.push(s.socket);
                partes.push(s.compose !== undefined && s.compose !== "" ? qsTr("compose: %1").arg(s.compose)
                                                                        : qsTr("sem compose"));
                return partes.join(" · ");
            }
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 10
        }

        // O passo oficial quando falta algo: instalar, entrar no grupo, subir o
        // daemon. A IDE nunca roda isso — imprime.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && root.controller.status.hint !== undefined
                     && root.controller.status.hint !== "" && !root.controller.statusBusy
            text: root.controller && root.controller.status.hint !== undefined
                  ? root.controller.status.hint : ""
            color: Theme.warningSoft
            font.pixelSize: 10
        }

        // --- Containers e imagens: dono proprio -------------------------
        ContainerListView {
            width: parent.width
            height: coluna.height - y - Theme.spacingSmall
            controller: root.controller
        }
    }

    Row {
        id: rodape

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
            text: qsTr("Atualizar")
            compact: true
            enabled: root.controller !== null && !root.controller.listBusy
            onClicked: root.controller.refresh()
        }

        // Compose do PROJETO: o arquivo padrao no workspace. `up` e' -d; a
        // saida viva mora na aba de logs de cada container.
        KvButton {
            text: qsTr("compose down")
            compact: true
            enabled: root.controller !== null && root.controller.reachable
                     && root.controller.status.compose !== undefined
                     && root.controller.workspaceRoot !== ""
            onClicked: root.controller.composeDown()
        }

        KvButton {
            text: qsTr("compose up")
            primary: true
            compact: true
            enabled: root.controller !== null && root.controller.reachable
                     && root.controller.status.compose !== undefined
                     && root.controller.workspaceRoot !== ""
            onClicked: root.controller.composeUp()
        }

        KvToggleChip {
            anchors.verticalCenter: parent.verticalCenter
            labelText: qsTr("parados também")
            active: root.controller ? root.controller.showAll : true
            onToggled: root.controller.setShowAll(!root.controller.showAll)
        }
    }
}
