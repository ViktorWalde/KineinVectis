import QtQuick
import KineinVectis

ListView {
    id: panel

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
                color: panel.statusColor(modelData.status)
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: 130
                text: modelData.displayName
                color: Theme.textPrimary
                font.pixelSize: 11
                font.bold: true
                elide: Text.ElideRight
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: modelData.version !== undefined
                      ? modelData.version
                      : (modelData.message !== undefined
                         ? modelData.message : "")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.family: Theme.monoFont
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: modelData.suggestedInstall !== undefined
                text: modelData.suggestedInstall !== undefined
                      ? modelData.suggestedInstall : ""
                color: Theme.accent
                font.pixelSize: 11
                font.family: Theme.monoFont
            }
        }
    }
}
