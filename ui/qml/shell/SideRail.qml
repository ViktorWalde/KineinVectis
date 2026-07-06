import QtQuick

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool explorerActive: false
    property bool toolsActive: false
    property bool logsActive: false
    property bool assistantActive: false

    signal explorerToggled()
    signal toolsRequested()
    signal logsRequested()
    signal assistantToggled()

    width: 42
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    component RailButton: Rectangle {
        id: railButton

        property string glyph: ""
        property bool active: false

        signal activated()

        width: 30
        height: 30
        radius: Theme.radius
        color: active ? Theme.accentDim
                      : (railButtonArea.containsMouse ? Theme.surface2 : "transparent")

        Text {
            anchors.centerIn: parent
            text: railButton.glyph
            color: railButton.active ? Theme.accent : Theme.textSecondary
            font.pixelSize: 14
        }

        MouseArea {
            id: railButtonArea

            anchors.fill: parent
            enabled: railButton.enabled
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: railButton.activated()
        }
    }

    Column {
        anchors.top: parent.top
        anchors.topMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Theme.spacingSmall

        RailButton {
            glyph: "▤"
            active: root.explorerActive && root.workspaceOpen
            enabled: root.workspaceOpen
            opacity: enabled ? 1.0 : 0.4
            onActivated: root.explorerToggled()
        }

        RailButton {
            glyph: "⚒"
            active: root.toolsActive
            onActivated: root.toolsRequested()
        }

        RailButton {
            glyph: "≣"
            active: root.logsActive
            onActivated: root.logsRequested()
        }

        RailButton {
            glyph: "✦"
            active: root.assistantActive
            onActivated: root.assistantToggled()
        }
    }
}
