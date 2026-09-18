pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de embarcados na moldura comum (KvPanelFrame, F8). E' o painel
// com mais informacao (nove secoes): a moldura fica entre o piso e o teto
// (decisao do autor, 2026-09-04: a altura segue o que ha' para mostrar) e
// o que passar ROLA — antes vazava por cima do editor (foto 12c).
KvPanelFrame {
    id: root

    property var controller: null
    property var toolchainController: null

    panelWidth: 560
    panelHeight: Math.min(780, Math.max(560, painel.implicitHeight + 2 * Theme.spacingMedium))
    contentHeight: painel.implicitHeight

    EmbeddedPanel {
        id: painel

        width: parent.width
        controller: root.controller
        toolchainController: root.toolchainController
        onCloseRequested: root.dismissRequested()
    }
}
