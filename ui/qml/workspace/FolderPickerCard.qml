import QtQuick
import KineinVectis

Rectangle {
    id: root

    property var controller
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 560

    signal closeRequested()

    width: Math.min(root.maxAvailableWidth, 720)
    height: Math.min(root.maxAvailableHeight, 560)
    radius: Theme.radiusLarge
    color: Theme.surface1
    border.color: Theme.borderSoft
    border.width: 1

    function focusPath() {
        pathBar.focusPath();
    }

    function focusCreateName() {
        createPanel.focusName();
    }

    MouseArea {
        anchors.fill: parent
    }

    Column {
        id: dialogColumn

        anchors.fill: parent
        anchors.margins: Theme.spacingLarge
        spacing: Theme.spacingMedium

        FolderPickerHeader {
            onCloseRequested: root.closeRequested()
        }

        FolderPickerPathBar {
            id: pathBar

            controller: root.controller
        }

        FolderPickerCreatePanel {
            id: createPanel

            controller: root.controller
        }

        FolderPickerDirectoryList {
            width: parent.width
            height: Math.max(150, parent.height - 28 - pathBar.height
                             - createPanel.height - actionRow.height
                             - errorLabel.height - 6 * Theme.spacingMedium)
            controller: root.controller
        }

        Text {
            id: errorLabel

            visible: root.controller.errorText !== ""
            width: parent.width
            height: visible ? implicitHeight : 0
            wrapMode: Text.WordWrap
            text: root.controller.errorText
            color: Theme.errorSoft
            font.pixelSize: 11
        }

        FolderPickerActionRow {
            id: actionRow

            pickingFolder: root.controller.purpose !== "workspace"
            onCancelRequested: root.closeRequested()
            onOpenRequested: root.controller.openSelected()
        }
    }
}
