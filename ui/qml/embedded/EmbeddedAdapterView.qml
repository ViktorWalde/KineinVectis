pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O depurador do kit: o rotulo do escolhido e um chip por candidato. Saiu do
// EmbeddedPanel em 2026-09-13, quando o provedor de instalacao de toolchain
// precisou de lugar no painel (293/300): escolher o adaptador e' outra
// responsabilidade que a de compor o painel. Burro: le e pede ao controller.
Item {
    id: root

    property var toolchainController: null

    readonly property var adapterOptions: root.toolchainController
        ? root.toolchainController.candidatesFor("debugAdapter") : []

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Depurador do kit: %1").arg(
                      root.toolchainController ? root.toolchainController.labelFor("debugAdapter") : "")
            color: Theme.textSecondary
            font.pixelSize: 11
            font.bold: true
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall

            Repeater {
                model: root.adapterOptions

                KvToggleChip {
                    id: chipAdaptador

                    required property var modelData

                    labelText: String(chipAdaptador.modelData.label)
                    active: {
                        const selecao = root.toolchainController.selectionFor("debugAdapter");
                        return selecao !== null && selecao.id === chipAdaptador.modelData.id;
                    }
                    onToggled: root.toolchainController.choose("debugAdapter",
                                                               chipAdaptador.modelData.id)
                }
            }
        }
    }
}
