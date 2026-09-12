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
Item {
    id: root

    property var controller: null

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Portas seriais")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Repeater {
            model: root.controller ? root.controller.ports : []

            Column {
                id: linhaPorta

                required property var modelData

                width: coluna.width
                spacing: Theme.spacingXSmall

                Row {
                    width: parent.width
                    spacing: Theme.spacingSmall

                    Text {
                        width: parent.width - 26 - Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: "● " + root.controller.portSummary(linhaPorta.modelData)
                        color: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 11
                        elide: Text.ElideMiddle
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
