pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de bibliotecas dentro de um dialogo, como os outros overlays.
//
// Separa o CONTEUDO (LibraryPanel, burro) do CHROME (fundo, moldura, dispensar
// por clique fora). Mesmo desenho do ConfigActionsDialog.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520

    signal dismissRequested()
    signal applyStepRequested(string actionId, var params)

    // Clique fora dispensa. Dentro, nao — senao escolher uma biblioteca
    // fecharia o painel.
    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: moldura

        anchors.centerIn: parent
        width: Math.min(560, root.maxAvailableWidth)
        height: Math.min(460, root.maxAvailableHeight)
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        LibraryPanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium

            libraries: root.controller ? root.controller.libraries : []
            selectedId: root.controller ? root.controller.selectedId : ""
            target: root.controller ? root.controller.target : ""
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
}
