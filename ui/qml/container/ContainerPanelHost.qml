pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de containers na moldura comum (KvPanelFrame, F8). A altura
// segue o que ha' para mostrar (uma linha por container e por imagem),
// entre o piso e o teto — decisao do autor de 2026-09-04 para o Embarcados,
// que vale aqui tambem.
KvPanelFrame {
    id: root

    property var controller: null

    readonly property int quantos: root.controller
        ? root.controller.containers.length + root.controller.images.length : 0

    panelWidth: 680
    panelHeight: Math.min(680, Math.max(400, 300 + quantos * 24))

    ContainerPanel {
        anchors.fill: parent
        controller: root.controller
        onCloseRequested: root.dismissRequested()
    }
}
