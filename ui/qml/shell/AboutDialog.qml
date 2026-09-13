import QtQuick

Item {
    id: root

    property real maxAvailableWidth: 640
    property real maxAvailableHeight: 480

    signal dismissRequested()

    onVisibleChanged: {
        if (visible) {
            forceActiveFocus();
        }
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: root.dismissRequested()
    }

    Rectangle {
        anchors.centerIn: parent
        width: Math.min(480, root.maxAvailableWidth)
        height: Math.min(300, root.maxAvailableHeight)
        radius: Theme.radiusDialog
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        MouseArea {
            anchors.fill: parent
        }

        KvIconButton {
            anchors.top: parent.top
            anchors.right: parent.right
            anchors.margins: Theme.spacingSmall
            iconName: "close"
            tooltip: qsTr("Fechar")
            onClicked: root.dismissRequested()
        }

        Column {
            anchors.centerIn: parent
            width: parent.width - 2 * Theme.spacingRegion
            spacing: Theme.spacingMedium

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: qsTr("Kinein Vectis")
                color: Theme.textPrimary
                font.pixelSize: 20
                font.bold: true
            }

            Text {
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("IDE para C, C++, Rust, Python e sistemas embarcados")
                color: Theme.textSecondary
                font.pixelSize: 12
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                horizontalAlignment: Text.AlignHCenter
                text: qsTr("Qt/QML frontend · Rust core · IPC JSON-RPC local")
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 11
            }
        }
    }

    Keys.onEscapePressed: root.dismissRequested()
}
