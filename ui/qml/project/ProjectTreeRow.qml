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
    // O arrasto em curso na arvore e a pasta que recebe o que cair aqui (o
    // explorador e' o dono das duas).
    property var activeDragPaths: []
    property string dropDirectory: ""
    readonly property bool beingDragged: root.activeDragPaths.indexOf(root.path) >= 0
    readonly property bool selfDrop: rowDrop.validDrag && !rowDrop.externalDrag
                                     && ProjectDragRules.insideAny(root.dropDirectory, root.activeDragPaths)
    readonly property string dropFolderName: root.dropDirectory.substring(
        root.dropDirectory.lastIndexOf("/") + 1) + "/"
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
    signal dragStarted()
    signal dragEnded()

    height: 24
    radius: Theme.radius
    color: dropHighlighted ? Theme.surface2
          : selected ? Theme.surfaceSelected
          : (rowHover.hovered ? Theme.surface2 : "transparent")
    border.width: dropHighlighted ? 2 : cursorFocused ? 1 : 0
    border.color: selfDrop ? Theme.errorSoft : Theme.accent
    // A origem do arrasto esmaece: o olho ve o que esta' saindo do lugar.
    opacity: beingDragged ? 0.45 : 1.0

    Drag.dragType: Drag.Automatic
    Drag.active: false
    Drag.supportedActions: Qt.CopyAction | Qt.MoveAction
    Drag.proposedAction: Qt.MoveAction
    Drag.mimeData: ({"text/uri-list": root.dragUrls,
                     "application/x-kinein-project-paths": JSON.stringify(root.dragPaths)})
    // A pilula vai abaixo e a direita do cursor, sem cobrir o "Mover para".
    Drag.hotSpot.x: -Theme.spacingMedium
    Drag.hotSpot.y: -Theme.spacingLarge
    Drag.onDragFinished: root.dragEnded()

    DragHandler {
        id: dragHandler
        enabled: root.dragPaths.length > 0
        acceptedButtons: Qt.LeftButton
        target: null
        // Antes da lista: no arrasto VERTICAL o ListView (que filtra os
        // eventos dos filhos) chegava ao limiar dele (~10 px) primeiro e
        // rolava em vez de arrastar — visto na tela real (0.3.9). Com 4 px o
        // arrasto ganha (toma o gesto da MouseArea da linha), e sem os flags
        // de "aprovar" ninguem o toma depois.
        dragThreshold: 4
        grabPermissions: PointerHandler.CanTakeOverFromAnything
        // O arrasto so' comeca depois de a pilula virar imagem: o sistema a
        // leva no cursor ate' o destino (0.3.9).
        onActiveChanged: {
            if (!active || root.dragPaths.length === 0) {
                root.Drag.active = false;
                return;
            }
            root.dragStarted();
            ghost.visible = true;
            ghost.grabToImage(function(result) {
                ghost.visible = false;
                root.Drag.imageSource = result.url;
                root.Drag.active = dragHandler.active;
            });
        }
    }

    ProjectDragGhost {
        id: ghost

        visible: false
        z: 10
        x: Theme.spacingSmall + root.depth * 12
        anchors.verticalCenter: parent.verticalCenter
        name: root.name
        directory: rules.isDirectory(root.kind)
        count: root.dragPaths.length
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
        interval: 450
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
        // O destino pelo NOME da pasta: num arquivo, cai na pasta dele.
        text: root.selfDrop ? qsTr("não cabe dentro de si mesma")
              : rowDrop.externalDrag ? qsTr("Importar para %1").arg(root.dropFolderName)
              : root.dropCopy ? qsTr("Copiar para %1").arg(root.dropFolderName)
              : qsTr("Mover para %1").arg(root.dropFolderName)
        color: root.selfDrop ? Theme.errorSoft : Theme.accent
        font.pixelSize: Theme.fontSizeCaption
        font.bold: true
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
