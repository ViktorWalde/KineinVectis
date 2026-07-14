import QtQuick

Rectangle {
    id: root

    property string text: ""
    property string iconName: ""
    property bool primary: false
    property bool selected: false
    property bool compact: false

    signal clicked()

    implicitWidth: buttonContent.implicitWidth + 2 * Theme.spacingMedium
    implicitHeight: compact ? 28 : 32
    radius: Theme.radius
    color: primary ? (buttonArea.pressed ? Theme.accentDim : Theme.accent)
                   : selected ? Theme.surfaceSelected
                   : (buttonArea.containsMouse || activeFocus
                      ? Theme.surface2 : Theme.surface1)
    border.color: activeFocus || selected ? Theme.accent : Theme.borderSoft
    border.width: primary ? 0 : 1
    opacity: enabled ? 1.0 : 0.72
    focus: true
    Accessible.role: Accessible.Button
    Accessible.name: text
    Keys.onSpacePressed: root.clicked()
    Keys.onReturnPressed: root.clicked()

    Row {
        id: buttonContent

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.iconName !== ""
            name: root.iconName
            size: 16
            active: root.selected
            disabled: !root.enabled
            iconColor: root.primary ? Theme.background0
                                    : (root.selected ? Theme.accent
                                       : Theme.textSecondary)
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.primary ? Theme.background0
                                : (root.enabled ? Theme.textPrimary
                                   : Theme.textDisabled)
            font.pixelSize: 12
            font.bold: root.primary || root.selected
        }
    }

    MouseArea {
        id: buttonArea

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: {
            root.forceActiveFocus();
            root.clicked();
        }
    }
}
