pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A BARRA DE MODO DO MARKDOWN (fatia V5/M1, 2026-09-25).
//
// §3.1: ao abrir `.md`/`.markdown`, a barra do editor oferece os tres modos.
// O "Lado a lado" entrou na M2; ate' entao a barra tinha DOIS botoes, porque
// anunciar um que nao faz nada seria a mentira que este projeto persegue.
Row {
    id: root

    property string mode: "edit"

    signal modeSelected(string mode)

    spacing: Theme.spacingXSmall

    Repeater {
        model: [
            { "id": "edit", "label": qsTr("Editar") },
            { "id": "preview", "label": qsTr("Preview") },
            { "id": "side", "label": qsTr("Lado a lado") }
        ]

        Rectangle {
            id: chip

            required property var modelData

            readonly property bool active: chip.modelData.id === root.mode

            width: rotulo.implicitWidth + 2 * Theme.spacingSmall
            height: 22
            radius: Theme.radius
            color: chip.active ? Theme.surfaceSelected
                               : (area.containsMouse ? Theme.surface2 : "transparent")
            border.width: chip.active ? 1 : 0
            border.color: Theme.borderStrong

            Text {
                id: rotulo

                anchors.centerIn: parent
                text: chip.modelData.label
                color: chip.active ? Theme.textPrimary : Theme.textSecondary
                font.pixelSize: Theme.fontSizeStatus
            }

            MouseArea {
                id: area

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.modeSelected(chip.modelData.id)
            }
        }
    }
}
