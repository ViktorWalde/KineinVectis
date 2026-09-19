pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de bibliotecas na moldura comum (KvPanelFrame, fechamento da
// Etapa 2): o CONTEUDO (LibraryPanel, burro) separado do CHROME.
KvPanelFrame {
    id: root

    property var controller: null

    signal applyStepRequested(string actionId, var params)

    panelWidth: 560
    panelHeight: 460

    LibraryPanel {
        anchors.fill: parent
        libraries: root.controller ? root.controller.libraries : []
        selectedId: root.controller ? root.controller.selectedId : ""
        target: root.controller ? root.controller.target : ""
        targets: root.controller ? root.controller.targets : []
        targetsOrigin: root.controller ? root.controller.targetsOrigin : ""
        plan: root.controller ? root.controller.plan : null
        errorText: root.controller ? root.controller.errorText : ""
        onLibrarySelected: function(id) { root.controller.select(id); }
        onTargetEdited: function(name) { root.controller.setTarget(name); }
        onCloseRequested: root.dismissRequested()
        onApplyStepRequested: function(actionId, params) {
            root.applyStepRequested(actionId, params);
        }
    }
}
