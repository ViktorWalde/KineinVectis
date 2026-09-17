pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As portas seriais USB: o canal que TODA placa compartilha (integracoes/38
// §2). Dono proprio pela mesma regra do EmbeddedSizeView: o painel ja' esta'
// no limite, e listar portas e' outra responsabilidade que sonda/kit/depurador.
//
// Burro: le do controller. O que mostra, por porta: o no' e a identidade USB,
// o que o VID:PID diz do ELO (nunca do chip atras de uma ponte), a permissao
// MEDIDA pelo core com a dica quando falta, e o aviso do ModemManager quando
// ele esta' vivo e a porta e' candidata.
//
// Desde 2026-09-17 a porta tambem se ESCOLHE (chip "Executar"): num projeto
// MicroPython o Executar roda o .py na placa por `mpremote connect <porta>
// run`, e sem escolha o mpremote pega a primeira que acha. A escolha e' do
// controller; aqui so' se mostra e se pede.
Item {
    id: root

    property var controller: null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Portas seriais")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            // E2: o que falta em cada canal (grupo/ACL, ModemManager, regra
            // das sondas) e o passo oficial — medido, nunca suposto; nada
            // roda sem o clique no passo.
            KvButton {
                anchors.verticalCenter: parent.verticalCenter
                text: root.controller && root.controller.access.busy ? qsTr("medindo…") : qsTr("Permissões")
                compact: true
                enabled: root.controller !== null && !root.controller.access.busy
                onClicked: root.controller.access.diagnose("")
            }
        }

        Repeater {
            model: root.controller ? root.controller.ports : []

            Column {
                id: linhaPorta

                required property var modelData
                readonly property bool escolhida: root.controller.selectedPort === modelData.device

                width: coluna.width
                spacing: Theme.spacingXSmall

                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall

                    Text {
                        width: parent.width - escolha.width - 2 * 26 - 3 * Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: "● " + root.controller.portSummary(linhaPorta.modelData)
                        // A escolhida veste o acento junto com o chip ativo:
                        // dois sinais, como o KvToggleChip pede de si mesmo.
                        color: linhaPorta.escolhida ? Theme.accent : Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                        elide: Text.ElideMiddle
                    }

                    // A porta do Executar. Sem acesso nao se escolhe o que vai
                    // falhar; a escolhida some da lista -> a escolha cai (controller).
                    KvToggleChip {
                        id: escolha

                        anchors.verticalCenter: parent.verticalCenter
                        labelText: qsTr("Executar")
                        active: linhaPorta.escolhida
                        enabled: linhaPorta.modelData.access.readableWritable
                        opacity: enabled ? 1.0 : 0.5
                        tooltip: linhaPorta.escolhida
                                 ? qsTr("Executar usa esta porta (mpremote connect %1 run); clique para desfazer").arg(linhaPorta.modelData.device)
                                 : qsTr("Usar esta porta no Executar de um projeto MicroPython")
                        onToggled: root.controller.selectPort(linhaPorta.modelData.device)
                    }

                    // Identificar PELO CANAL (E5): o esptool abre a porta e a
                    // placa reseta — por isso e' um clique, nunca automatico.
                    // Um pedido por vez (o esptool prende a porta).
                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "search"
                        tooltip: qsTr("Identificar o chip pelo esptool (flash-id): abre a porta e RESETA a placa")
                        compact: true
                        enabled: linhaPorta.modelData.access.readableWritable
                                 && !root.controller.identity.busy
                        onClicked: root.controller.identity.identify(linhaPorta.modelData.device)
                    }

                    // Os arquivos na placa (C2): o mpremote entra no raw REPL
                    // e interrompe o programa dela — clique, nunca automatico.
                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "folder"
                        tooltip: qsTr("Arquivos na placa (mpremote fs): interrompe o programa em execução")
                        compact: true
                        enabled: linhaPorta.modelData.access.readableWritable
                                 && !root.controller.files.busy
                        onClicked: root.controller.files.list(linhaPorta.modelData.device, "")
                    }

                    // O monitor abre numa aba de terminal com a ferramenta do
                    // kit (tio/picocom/minicom/espflash). Sem acesso a porta,
                    // nao se oferece o que vai falhar.
                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "terminal"
                        tooltip: qsTr("Monitor serial a 115200 (aba de terminal)")
                        compact: true
                        enabled: linhaPorta.modelData.access.readableWritable
                        onClicked: root.controller.openMonitor(linhaPorta.modelData.device)
                    }
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: text !== ""
                    text: linhaPorta.modelData.family !== undefined ? linhaPorta.modelData.family : ""
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                // A permissao e' o que o core MEDIU com access(2); a dica e' o
                // passo oficial, nunca um `sudo` que a IDE rodaria.
                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: !linhaPorta.modelData.access.readableWritable
                    text: qsTr("sem acesso: %1").arg(
                              linhaPorta.modelData.access.hint !== undefined
                              ? linhaPorta.modelData.access.hint : linhaPorta.modelData.access.mode)
                    color: Theme.errorSoft
                    font.pixelSize: 10
                }

                Text {
                    width: parent.width
                    wrapMode: Text.WordWrap
                    visible: root.controller.modemManagerWarns(linhaPorta.modelData)
                    text: qsTr("o ModemManager está ativo e pode ocupar esta porta por alguns segundos após conectar; uma regra udev com ID_MM_DEVICE_IGNORE=1 evita isso")
                    color: Theme.warningSoft
                    font.pixelSize: 10
                }
            }
        }

        // O que o Executar vai fazer com a escolha — dito aqui, onde se
        // escolhe, e nao descoberto na aba Terminal depois do clique.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && root.controller.portFound
            text: root.controller
                  ? (root.controller.selectedPort !== ""
                     ? qsTr("Executar (MicroPython): mpremote connect %1 run").arg(root.controller.selectedPort)
                     : qsTr("nenhuma porta escolhida: o Executar de MicroPython usa a primeira que o mpremote achar"))
                  : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.controller && !root.controller.portFound
            text: root.controller
                  ? (root.controller.portsBusy ? qsTr("procurando…")
                     : (root.controller.portsHint !== "" ? root.controller.portsHint
                                                         : qsTr("nenhuma porta serial USB")))
                  : ""
            color: Theme.textSecondary
            font.pixelSize: 11
        }
    }
}
