pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel do Grafana na moldura comum (KvPanelFrame, fechamento da Etapa
// 2). A altura segue o que ha' para mostrar (achados, fontes, dashboards),
// entre o piso (o formulario inteiro) e o teto (decisao do autor,
// 2026-09-04).
KvPanelFrame {
    id: root

    property var controller: null

    readonly property int quantosAchados: root.controller
        ? root.controller.matches.length + root.controller.dataSources.length
          + root.controller.dashboards.length
        : 0

    panelWidth: 560
    panelHeight: Math.min(620, Math.max(320, 300 + quantosAchados * 30))

    GrafanaPanel {
        anchors.fill: parent
        controller: root.controller
    }
}
