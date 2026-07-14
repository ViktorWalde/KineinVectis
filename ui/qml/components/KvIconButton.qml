import QtQuick

Rectangle {
    id: root

    property string iconName: "file"
    property string tooltip: ""
    property string accessibleName: tooltip
    property bool active: false
    property bool primary: false
    property bool danger: false
    property bool compact: false
    property int iconSize: 20
    property string tooltipPlacement: "bottom"

    signal clicked()

    implicitWidth: compact ? 24 : 32
    implicitHeight: compact ? 24 : 32
    radius: Theme.radius
    color: primary ? (buttonArea.pressed ? Theme.accentDim : Theme.accent)
                   : active ? Theme.surfaceSelected
                   : (buttonArea.containsMouse || activeFocus
                      ? Theme.surface2 : "transparent")
    border.color: activeFocus ? Theme.accent : "transparent"
    border.width: 1
    opacity: enabled ? 1.0 : 0.72
    focus: true
    Accessible.role: Accessible.Button
    Accessible.name: accessibleName
    Keys.onSpacePressed: root.clicked()
    Keys.onReturnPressed: root.clicked()

    KvIcon {
        anchors.centerIn: parent
        name: root.iconName
        size: root.compact ? Math.min(root.iconSize, 16) : root.iconSize
        active: root.active
        disabled: !root.enabled
        error: root.danger
        iconColor: root.primary ? Theme.background0
                                : (root.danger ? Theme.errorSoft
                                   : (root.active ? Theme.accent
                                      : (buttonArea.containsMouse
                                         ? Theme.textPrimary
                                         : Theme.textSecondary)))
    }

    MouseArea {
        id: buttonArea

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onContainsMouseChanged: {
            if (containsMouse) {
                TooltipController.showFor(root, root.tooltip,
                                          root.tooltipPlacement);
            } else {
                TooltipController.hideFor(root);
            }
        }
        onClicked: {
            root.forceActiveFocus();
            TooltipController.hideFor(root);
            root.clicked();
        }
    }
}
