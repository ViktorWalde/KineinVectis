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
        font.pixelSize: Theme.fontSizeSmall
        elide: Text.ElideRight
    }

    KvButton {
        id: reloadButton

        anchors.right: keepButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        compact: true
        visible: !root.watcherFailure && !root.deleted
        text: qsTr("Recarregar do disco")
        onClicked: root.reloadRequested()
    }

    KvButton {
        id: keepButton

        anchors.right: dismissButton.left
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        height: 24
        compact: true
        selected: true
        visible: !root.watcherFailure
        text: root.deleted ? qsTr("Manter buffer") : qsTr("Manter local")
        onClicked: root.keepLocalRequested()
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
