pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

ListView {
    id: panel


    // B2 (docs/roadmaps/24): barra de rolagem. `parent: panel` é OBRIGATÓRIO — um filho
    // declarado dentro de um ListView vira filho do contentItem e ROLARIA
    // junto com a lista. O ListView segue sendo a fonte da verdade.
    VerticalScrollBar {
        id: scrollBar_panel

        parent: panel
        anchors.right: panel.right
        anchors.top: panel.top
        anchors.bottom: panel.bottom

        contentSize: panel.contentHeight
        viewportSize: panel.height
        position: panel.contentY

        onMoveRequested: function(position) {
            panel.contentY = position;
        }
    }
    property var tools: []

    function statusColor(status) {
        if (status === "detected") {
            return Theme.successSoft;
        }
        if (status === "failed") {
            return Theme.warningSoft;
        }
        return Theme.errorSoft;
    }

    clip: true
    model: tools

    delegate: Rectangle {
        id: toolDelegate

        required property var modelData

        width: panel.width
        height: 26
        radius: Theme.radius
        color: "transparent"

        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingSmall
            spacing: Theme.spacingMedium

            Rectangle {
                width: 8
                height: 8
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: panel.statusColor(toolDelegate.modelData.status)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: 130
                text: toolDelegate.modelData.displayName
                color: Theme.textPrimary
                font.pixelSize: 11
                font.bold: true
                elide: Text.ElideRight
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: toolDelegate.modelData.version !== undefined
                      ? toolDelegate.modelData.version
                      : (toolDelegate.modelData.message !== undefined
                         ? toolDelegate.modelData.message : "")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.family: Theme.monoFont
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: toolDelegate.modelData.suggestedInstall !== undefined
                text: toolDelegate.modelData.suggestedInstall !== undefined
                      ? toolDelegate.modelData.suggestedInstall : ""
                color: Theme.accent
                font.pixelSize: 11
                font.family: Theme.monoFont
            }
        }
    }
}
