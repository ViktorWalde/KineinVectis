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

        // A primeira linha comum dos paineis de ambiente (F8): titulo, uma
        // linha, a acao primaria — "Atualizar" — e o x.
        KvPanelHeader {
            width: parent.width
            title: qsTr("Containers")
            // O QUE A TELA PROMETE E' O QUE ELA FAZ.
            subtitle: qsTr("Docker ou Podman, o que responder nesta máquina. Parar, iniciar e remover são jobs canceláveis; logs e shell abrem numa aba do terminal. Nada roda como root.")
            primaryLabel: qsTr("Atualizar")
            primaryIcon: "refresh"
            primaryEnabled: root.controller !== null
            primaryBusy: root.controller !== null && root.controller.listBusy
            onPrimaryRequested: root.controller.refresh()
            onCloseRequested: root.closeRequested()
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: text !== ""
            text: root.controller ? root.controller.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        // --- Motor: o veredito comum (F8) ---------------------------------
        KvVerdict {
            width: parent.width
            busy: root.controller !== null && root.controller.statusBusy
            busyText: qsTr("procurando o motor…")
            ok: root.controller !== null && root.controller.engineFound && root.controller.reachable
            message: {
                if (!root.controller) return "";
                if (!root.controller.engineFound) return qsTr("nenhum motor de container");
                const s = root.controller.status;
                const partes = [qsTr("Motor: %1").arg(root.controller.engineLabel)];
                partes.push(s.reachable ? qsTr("responde") : qsTr("NÃO responde"));
                if (s.rootless === true) partes.push("rootless");
                if (s.rootless === false) partes.push(qsTr("com daemon/root"));
                if (s.socket !== undefined && s.socket !== "") partes.push(s.socket);
                partes.push(root.controller.composeSummary);
                return partes.join(" · ");
            }
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

        // Compose do PROJETO: o arquivo padrao na raiz do workspace (o core
        // diz qual). `up` e' -d; a saida viva mora na aba de logs de cada
        // container. Sem arquivo o botao fica desligado — e a linha do motor
        // diz o que falta.
        KvButton {
            text: qsTr("compose down")
            compact: true
            enabled: root.controller !== null && root.controller.canCompose
            onClicked: root.controller.composeDown()
        }

        KvButton {
            text: qsTr("compose up")
            primary: true
            compact: true
            enabled: root.controller !== null && root.controller.canCompose
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
