import QtQuick
import KineinVectis

// O layout da casca: trilho, area da esquerda e centro lado a lado.
//
// A ILHA UNICA (0.3.9, pedido do autor: "em vez de 3 ilhas, tudo dentro de
// uma ilha so', e ao redor um unico rodape"): o trilho fica na moldura; a
// area da esquerda, o editor e o painel de baixo ficam sobre UM fundo
// arredondado, sem borda propria, separados por divisorias finas no meio dos
// vaos. Quem monta (ShellWorkspaceHost) so' diz quem e' quem.
Item {
    id: root

    default property alias contentItems: contentRow.data
    property Item railItem: null
    property Item leftItem: null
    property Item centerItem: null
    property Item bottomItem: null

    Rectangle {
        id: island

        x: root.railItem !== null ? root.railItem.width + Theme.panelGap : 0
        width: Math.max(0, root.width - x)
        height: root.height
        radius: Theme.radiusLarge
        color: Theme.background1
    }

    Row {
        id: contentRow

        anchors.fill: parent
        spacing: Theme.panelGap
    }

    // Entre a area da esquerda e o centro.
    Rectangle {
        visible: root.leftItem !== null && root.leftItem.visible
        x: root.leftItem !== null ? root.leftItem.x + root.leftItem.width + Theme.panelGap / 2 : 0
        y: Theme.spacingSmall
        width: 1
        height: root.height - 2 * Theme.spacingSmall
        color: Theme.borderSoft
    }

    // Entre o editor e o painel de baixo.
    Rectangle {
        visible: root.bottomItem !== null && root.bottomItem.visible
                 && root.centerItem !== null && root.centerItem.visible
        x: root.centerItem !== null ? root.centerItem.x + Theme.spacingSmall : 0
        y: root.centerItem !== null && root.bottomItem !== null
           ? root.centerItem.y + root.bottomItem.y - Theme.panelGap / 2 : 0
        width: root.centerItem !== null ? root.centerItem.width - 2 * Theme.spacingSmall : 0
        height: 1
        color: Theme.borderSoft
    }
}
