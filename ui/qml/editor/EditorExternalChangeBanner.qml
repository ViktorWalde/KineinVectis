import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool active: false
    property bool deleted: false
    property bool watcherFailure: false
    property string message: ""

    signal reloadRequested()
    signal keepLocalRequested()
    signal dismissRequested()

    height: visible ? 38 : 0
    visible: active
    radius: Theme.radius
    color: Theme.surface2
    border.color: watcherFailure ? Theme.errorSoft : Theme.warningSoft
    border.width: 1

    Text {
        id: messageText

        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: reloadButton.visible ? reloadButton.left
                      : (keepButton.visible ? keepButton.left : dismissButton.left)
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        text: root.message
        color: Theme.textPrimary
        font.pixelSize: 11
        elide: Text.ElideRight
    }

    Rectangle {
        id: reloadButton

        width: reloadText.width + 2 * Theme.spacingSmall
        height: 24
        anchors.right: keepButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: !root.watcherFailure && !root.deleted
        radius: Theme.radius
        color: reloadArea.containsMouse ? Theme.surfaceSelected : "transparent"
        border.color: Theme.borderStrong
        border.width: 1

        Text {
            id: reloadText

            anchors.centerIn: parent
            text: qsTr("Recarregar do disco")
            color: Theme.textPrimary
            font.pixelSize: 10
        }

        MouseArea {
            id: reloadArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.reloadRequested()
        }
    }

    Rectangle {
        id: keepButton

        width: keepText.width + 2 * Theme.spacingSmall
        height: 24
        anchors.right: dismissButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: !root.watcherFailure
        radius: Theme.radius
        color: keepArea.containsMouse ? Theme.surfaceSelected : "transparent"
        border.color: Theme.accentDim
        border.width: 1

        Text {
            id: keepText

            anchors.centerIn: parent
            text: root.deleted ? qsTr("Manter buffer") : qsTr("Manter local")
            color: Theme.accentActive
            font.pixelSize: 10
        }

        MouseArea {
            id: keepArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.keepLocalRequested()
        }
    }

    KvIconButton {
        id: dismissButton
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: Theme.spacingSmall
        visible: root.watcherFailure
        compact: true
        iconName: "close"
        danger: true
        tooltip: qsTr("Dispensar aviso")
        onClicked: root.dismissRequested()
    }
}
