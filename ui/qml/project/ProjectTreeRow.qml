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
    property var dragPaths: []
    readonly property bool dropHighlighted: rowDrop.validDrag
    readonly property bool dropCopy: rowDrop.copy
    readonly property string dragUrls: dragPaths.map(path =>
        "file://" + path.split("/").map(encodeURIComponent).join("/")
    ).join("\r\n") + "\r\n"

    signal clicked(int modifiers, int button, real sceneX, real sceneY)
    signal scriptRunRequested()
    signal filesDropped(var paths, bool copy, bool external)
    signal directoryHoverRequested()
    signal dragPosition(real sceneY)
    signal dragEnded()

    height: 24
    radius: Theme.radius
    color: dropHighlighted ? Theme.surface2
          : selected ? Theme.surfaceSelected
          : (rowHover.hovered ? Theme.surface2 : "transparent")
    border.width: dropHighlighted ? 2 : cursorFocused ? 1 : 0
    border.color: Theme.accent

    Drag.dragType: Drag.Automatic
    Drag.active: false
    Drag.supportedActions: Qt.CopyAction | Qt.MoveAction
    Drag.proposedAction: Qt.MoveAction
    Drag.mimeData: ({"text/uri-list": root.dragUrls,
                     "application/x-kinein-project-paths": JSON.stringify(root.dragPaths)})
    Drag.onDragFinished: root.dragEnded()

    DragHandler {
        id: dragHandler
        enabled: root.dragPaths.length > 0
        acceptedButtons: Qt.LeftButton
        target: null
        onActiveChanged: root.Drag.active = active && root.dragPaths.length > 0
    }

    ProjectTreeDropArea {
        id: rowDrop
        anchors.fill: parent
        destination: root.path
        urlDecoder: Clipboard
        onValidDragChanged: {
            if (validDrag && rules.isDirectory(root.kind) && !root.expanded)
                hoverExpand.restart();
            else hoverExpand.stop();
        }
        onDragPosition: function(y) {
            root.dragPosition(root.mapToItem(null, 0, y).y);
        }
        onFilesDropped: function(paths, destination, copy, external) {
            root.filesDropped(paths, copy, external);
        }
    }

    Timer {
        id: hoverExpand
        interval: 650
        onTriggered: root.directoryHoverRequested()
    }

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

    Text {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingSmall
        anchors.verticalCenter: parent.verticalCenter
        visible: root.dropHighlighted
        text: rowDrop.externalDrag ? qsTr("Importar aqui")
              : root.dropCopy ? qsTr("Copiar aqui") : qsTr("Mover aqui")
        color: Theme.accent
        font.pixelSize: 10
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
