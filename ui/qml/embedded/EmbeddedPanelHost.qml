pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de embarcados na moldura comum (KvPanelFrame, F8). Em ABAS
// desde a E3-5 (roadmaps/44): a moldura e' fixa em 640x560 — cada aba
// cabe sem rolar a 800 px de janela; o que passar ROLA. Antes a coluna
// de nove secoes ia ate' 780 e passava da janela (foto de 2026-09-19).
KvPanelFrame {
    id: root

    property var controller: null
    property var toolchainController: null

    panelWidth: 640
    panelHeight: 560
    contentHeight: painel.implicitHeight

    EmbeddedPanel {
        id: painel

        width: parent.width
        controller: root.controller
        toolchainController: root.toolchainController
        onCloseRequested: root.dismissRequested()
    }
}
