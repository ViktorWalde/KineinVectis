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

        // A escolha de uma opcao, na linguagem do KvToggleChip (2026-10-03).
        KvToggleChip {
            id: chip

            required property var modelData

            height: 22
            outlined: true
            codeFont: false
            labelText: chip.modelData.label
            active: chip.modelData.id === root.mode
            onToggled: root.modeSelected(chip.modelData.id)
        }
    }
}
