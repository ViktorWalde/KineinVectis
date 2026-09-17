pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A identidade PELO CANAL (E5 do integracoes/38 §6): o que o esptool leu da
// placa atras da porta perguntada, e o kit que isso sugere. Dono proprio
// pela mesma regra do EmbeddedSerialView: o painel ja' esta' no limite.
//
// Burro: le do controller filho (`controller.identity`). O botao que PEDE
// esta' na linha da porta (EmbeddedSerialView); aqui e' so' o resultado —
// vazio ate' o primeiro clique, porque identificar reseta a placa e nada
// aqui roda sozinho.
Item {
    id: root

    // O EmbeddedController; o que se le e' `controller.identity`.
    property var controller: null
    readonly property var identity: controller ? controller.identity : null

    implicitHeight: coluna.implicitHeight
    visible: identity !== null && identity.device !== ""

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            text: root.identity
                  ? qsTr("Identidade pelo canal — %1").arg(root.identity.device) : ""
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.identity && root.identity.busy
            text: qsTr("perguntando ao esptool… (a placa reseta para entrar no bootloader)")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        // O que a placa disse, uma linha: chip · flash · MAC.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.identity && root.identity.found
            text: root.identity && root.identity.found ? "● " + root.identity.summary(root.identity.identity) : ""
            color: Theme.textPrimary
            font.family: Theme.monoFont
            font.pixelSize: 11
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.identity && root.identity.found && root.identity.identity.features !== undefined
                     && root.identity.identity.features.length > 0
            text: root.identity && root.identity.identity.features !== undefined
                  ? root.identity.identity.features.join(", ") : ""
            color: Theme.textMuted
            font.pixelSize: 10
        }

        // O kit SUGERIDO e o botao que o aplica: so' o chip muda no kit.
        Row {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.identity && root.identity.target.chip !== undefined

            Text {
                width: parent.width - aplicar.width - Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                wrapMode: Text.WordWrap
                text: root.identity ? qsTr("sugere: %1").arg(root.identity.targetSummary(root.identity.target)) : ""
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            KvButton {
                id: aplicar

                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Usar chip no kit")
                compact: true
                onClicked: root.identity.applyToKit()
            }
        }

        // A recusa ou a falha, com a mensagem do core/esptool.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.identity && root.identity.errorText !== ""
            text: root.identity ? root.identity.errorText : ""
            color: Theme.errorSoft
            font.pixelSize: 10
        }

        // A saida crua so' quando o parser nao entendeu (sem chip): e' o que
        // deixa ver se o formato mudou — "nao entendi, e aqui esta' o que veio".
        Text {
            width: parent.width
            wrapMode: Text.WrapAnywhere
            visible: root.identity && !root.identity.busy && !root.identity.found
                     && root.identity.rawOutput !== ""
            text: root.identity ? root.identity.rawOutput : ""
            color: Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 10
        }
    }
}
