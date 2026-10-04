import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool active: false
    property string status: "idle"
    property string message: ""
    property string actionLabel: ""

    signal actionRequested()
    signal dismissRequested()

    height: visible ? 34 : 0
    visible: active
    radius: Theme.radius
    color: status === "warning" ? Theme.surface2 : Theme.surface1
    border.color: statusColor()
    border.width: 1

    function statusColor() {
        if (status === "warning") {
            return Theme.warningSoft;
        }
        if (status === "busy") {
            return Theme.accent;
        }
        return Theme.infoSoft;
    }

    Rectangle {
        id: statusDot

        width: 8
        height: 8
        radius: Theme.radiusLarge
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        color: root.statusColor()
    }

    Text {
        id: titleText

        anchors.left: statusDot.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        text: qsTr("Project Health")
        color: Theme.textPrimary
        font.pixelSize: Theme.fontSizePanelTitle
        font.bold: true
    }

    Text {
        anchors.left: titleText.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: actionButton.visible
                       ? actionButton.left : dismissButton.left
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        text: root.message
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
        elide: Text.ElideRight
    }

    KvButton {
        id: actionButton

        anchors.right: dismissButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        compact: true
        visible: root.actionLabel !== ""
        text: root.actionLabel
        onClicked: root.actionRequested()
    }

    KvIconButton {
        id: dismissButton
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: Theme.spacingSmall
        compact: true
        iconName: "close"
        tooltip: qsTr("Dispensar")
        onClicked: root.dismissRequested()
    }
}
