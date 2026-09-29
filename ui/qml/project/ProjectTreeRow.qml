pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Uma linha visual da arvore. Selecao, cursor e hover sao estados distintos.
Rectangle {
    id: root

    required property int index
    required property string path
    required property string name
    required property string kind
    required property int depth
    required property bool expanded
    required property bool machine

    property bool selected: false
    property bool cursorFocused: false
    property color gitColor: Theme.textSecondary
    property bool runnable: false

    signal clicked(int modifiers, int button, real sceneX, real sceneY)
    signal scriptRunRequested()

    height: 24
    radius: Theme.radius
    color: selected ? Theme.surfaceSelected
          : (rowHover.hovered ? Theme.surface2 : "transparent")
    border.width: cursorFocused ? 1 : 0
    border.color: Theme.accent

    ProjectTreeRules {
        id: rules
    }

    HoverHandler {
        id: rowHover
    }

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + root.depth * 12
        spacing: Theme.spacingXSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: 12
            text: rules.isDirectory(root.kind) ? (root.expanded ? "▾" : "▸") : ""
            color: rules.isDirectory(root.kind) && !root.machine
                   ? Theme.accent : Theme.textMuted
            font.pixelSize: Theme.fontSizeTree
        }

        KvFileIcon {
            anchors.verticalCenter: parent.verticalCenter
            size: 20
            opacity: root.machine ? 0.55 : 1
            fileName: root.name
            directory: rules.isDirectory(root.kind)
            expanded: root.expanded
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, root.width - parent.x - x - 30)
            text: root.name
            color: root.machine ? Theme.textMuted
                   : (rules.isDirectory(root.kind) ? Theme.textPrimary : root.gitColor)
            font.pixelSize: Theme.fontSizeTree
            elide: Text.ElideRight
        }
    }

    MouseArea {
        id: entryArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: function(mouse) {
            const point = entryArea.mapToItem(null, mouse.x, mouse.y);
            root.clicked(mouse.modifiers, mouse.button, point.x, point.y);
        }
    }

    KvIconButton {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 22
        height: 22
        z: 2
        visible: root.runnable && (rowHover.hovered || root.selected)
        enabled: visible
        iconName: "run"
        iconSize: 13
        primary: true
        tooltip: qsTr("Executar script")
        onClicked: root.scriptRunRequested()
    }
}
