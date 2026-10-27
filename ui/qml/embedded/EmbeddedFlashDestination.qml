pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Destino do probe-rs nativo (0.4.1h4b2b4b2, roadmaps/52 §14.35): o alvo
// executavel do CMake e a sonda listada. Burro: le `controller.flash` e
// `controller.probes`; quem valida versao, chip, artefato e sonda e' o core.
Column {
    id: root

    property var controller: null
    readonly property var flash: controller ? controller.flash : null
    readonly property var probes: controller ? controller.probes : []

    visible: flash !== null && flash.probeDestination
    spacing: Theme.spacingSmall

    EmbeddedFlashTargetView { width: parent.width; controller: root.controller }

    Text {
        width: parent.width
        text: qsTr("Sonda de depuração")
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        font.bold: true
    }

    // A dica fica depois da escolha: se sumisse, os controles abaixo subiriam
    // e um segundo clique no mesmo ponto cairia em outro controle.
    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        text: root.probes.length > 0 ? qsTr("Escolha a sonda conectada ao alvo; a porta serial não é usada nesta gravação.")
            : root.controller && root.controller.busy ? qsTr("Procurando sondas…")
            : qsTr("Nenhuma sonda listada. Conecte a sonda por USB e atualize.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    Flow {
        width: parent.width
        spacing: Theme.spacingSmall

        Repeater {
            model: root.probes.filter(entry => root.flash && root.flash.probeSelectable(entry))

            KvToggleChip {
                required property var modelData
                readonly property string selector: root.flash.selectorOf(modelData)

                objectName: "flashProbe_" + selector
                labelText: modelData.name + " · " + modelData.serial
                codeFont: false
                height: 28
                active: root.flash.probe === selector
                tooltip: selector + (modelData.kind ? " (" + modelData.kind + ")" : "")
                onToggled: root.flash.selectProbe(selector)
            }
        }

        KvButton {
            objectName: "flashRefreshProbes"
            text: qsTr("Atualizar sondas")
            compact: true
            enabled: root.controller !== null && !root.controller.busy
            onClicked: root.controller.refreshProbes()
        }
    }

    // Listada, mas nao e' destino: sem serial nao identifica; inacessivel
    // precisa de permissao. O motivo fica visivel, sem chip clicavel.
    Repeater {
        model: root.probes.filter(entry => root.flash && !root.flash.probeSelectable(entry))

        Text {
            required property var modelData

            width: root.width
            wrapMode: Text.WordWrap
            text: modelData.inaccessible
                ? qsTr("%1 (%2:%3): sem permissão de acesso. Ajuste a regra de udev; não rode a IDE como root.").arg(modelData.name).arg(modelData.vid).arg(modelData.pid)
                : qsTr("%1 (%2:%3): sem número de série, não identifica o destino.").arg(modelData.name).arg(modelData.vid).arg(modelData.pid)
            color: Theme.warningSoft
            font.pixelSize: Theme.fontSizeCaption
        }
    }
}
