pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de instalacao na moldura comum (KvPanelFrame, fechamento da
// Etapa 2): o CONTEUDO (SetupPanel, burro) separado do CHROME.
KvPanelFrame {
    id: root

    property var controller: null

    signal commandRequested(string command)

    panelWidth: 620
    panelHeight: 500

    SetupPanel {
        anchors.fill: parent
        distroName: root.controller ? root.controller.distroName : ""
        tools: root.controller ? root.controller.tools : []
        expandedId: root.controller ? root.controller.expandedId : ""
        errorText: root.controller ? root.controller.errorText : ""
        onToggleRequested: id => root.controller.toggle(id)
        onCommandRequested: comando => root.commandRequested(comando)
        onCloseRequested: root.dismissRequested()
    }
}
