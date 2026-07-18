import QtQuick

// Uma linha da arvore de projeto: pinta, e' fonte de arraste e alvo de drop.
//
// Saiu de dentro do `ProjectExplorer` quando ganhou arraste: a linha passou a
// ter comportamento proprio (fonte, alvo, estado de hover do drop) e o painel
// voltou a ser so' cabecalho + lista.
//
// `gestures` e o `ProjectTreeGestures`. A linha NAO decide se um drop e valido
// — ela pergunta. Recalcular a regra de caminho aqui seria duplicar logica no
// visual, que e exatamente o que o gate `verificar-qml-logica.sh` cobra.
Rectangle {
    id: row

    // Injetadas pelo modelo do ListView.
    required property int index
    required property string path
    required property string name
    required property string kind
    required property int depth
    required property bool expanded

    property var gestures: null
    property string selectedPath: ""
    property var gitKinds: ({})
    property int gitRevision: 0
    property bool dragging: false

    signal entryClicked(string path, string kind)
    signal entryToggled(string path, int index, bool expanded)
    signal entryOpened(string path)
    signal scriptRunRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)
    signal dragStarted(string path)
    signal dragEnded()
    signal dropped(string sourcePath, string targetPath, string targetKind)

    function gitFileColor(entryPath, revision) {
        const gitKind = gitKinds[entryPath];
        if (gitKind === undefined) {
            return Theme.textSecondary;
        }
        if (gitKind === "conflicted") {
            return Theme.errorSoft;
        }
        if (gitKind === "untracked" || gitKind === "added") {
            return Theme.successSoft;
        }
        if (gitKind === "deleted") {
            return Theme.textDisabled;
        }
        return Theme.infoSoft;
    }

    function treeIconName(entryName, entryKind, isExpanded) {
        if (entryKind === "directory") {
            return isExpanded ? "tree-folder-open" : "tree-folder-closed";
        }
        const lowerName = entryName.toLowerCase();
        if (lowerName.endsWith(".c") || lowerName.endsWith(".h")) {
            return "tree-file-c";
        }
        if (lowerName.endsWith(".cc") || lowerName.endsWith(".cpp")
                || lowerName.endsWith(".cxx") || lowerName.endsWith(".c++")
                || lowerName.endsWith(".hh") || lowerName.endsWith(".hpp")
                || lowerName.endsWith(".hxx") || lowerName.endsWith(".h++")
                || lowerName.endsWith(".ipp")) {
            return "tree-file-cpp";
        }
        if (lowerName.endsWith(".rs")) {
            return "tree-file-rust";
        }
        return "file";
    }

    function isRunnableScript(entryName, entryKind) {
        if (entryKind !== "file") {
            return false;
        }
        const lower = entryName.toLowerCase();
        return lower.endsWith(".sh") || lower.endsWith(".bash")
                || lower.endsWith(".zsh");
    }

    // Alvo aceso so' quando o drop e' aceitavel: pasta invalida nao pisca.
    readonly property bool dropTarget: entryDrop.containsDrag
                                       && entryDrop.acceptsSource

    height: 24
    radius: Theme.radius
    opacity: row.dragging ? 0.45 : 1
    color: row.dropTarget
           ? Theme.accentDim
           : (row.path === row.selectedPath
              ? Theme.surfaceSelected
              : (entryArea.containsMouse ? Theme.surface2 : "transparent"))
    border.width: row.dropTarget ? 1 : 0
    border.color: Theme.accent

    Row {
        anchors.verticalCenter: parent.verticalCenter
        anchors.left: parent.left
        anchors.leftMargin: Theme.spacingSmall + row.depth * 14
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: 12
            text: row.kind === "directory" ? (row.expanded ? "▾" : "▸") : ""
            color: row.kind === "directory" ? Theme.accent : Theme.textMuted
            font.pixelSize: Theme.fontSizeTree
        }

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            size: 20
            name: row.treeIconName(row.name, row.kind, row.expanded)
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: Math.max(0, row.width - parent.x - x - 30)
            text: row.name
            color: row.kind === "directory"
                   ? Theme.textPrimary
                   : row.gitFileColor(row.path, row.gitRevision)
            font.pixelSize: Theme.fontSizeTree
            elide: Text.ElideRight
        }
    }

    // Carrega o arraste. Fica separado da linha porque arrastar a PROPRIA linha
    // brigaria com o posicionamento do ListView — some do lugar e a lista salta.
    Item {
        id: dragProxy

        width: row.width
        height: row.height

        Drag.active: entryArea.drag.active
        Drag.source: row
        Drag.keys: ["kinein/tree-entry"]
        Drag.hotSpot.x: 12
        Drag.hotSpot.y: row.height / 2
    }

    DropArea {
        id: entryDrop

        anchors.fill: parent
        keys: ["kinein/tree-entry"]

        // O `drag.source` do DropArea chega tipado como QObject. O `as` declara
        // que ali vem uma linha da arvore, em vez de acessar `.path` num tipo
        // que nao a possui — o gate de lint roda com zero warning.
        readonly property ProjectTreeRow sourceRow: drag.source as ProjectTreeRow
        readonly property string sourcePath: sourceRow ? sourceRow.path : ""
        readonly property bool acceptsSource:
            row.gestures !== null && sourcePath !== ""
            && row.gestures.canDropOn(sourcePath, row.path, row.kind)

        onDropped: function(drop) {
            if (acceptsSource) {
                row.dropped(sourcePath, row.path, row.kind);
                drop.accept();
            }
        }
    }

    MouseArea {
        id: entryArea

        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor

        drag.target: dragProxy
        // Sem limiar, um clique com tremor de mao vira arraste e o arquivo se
        // muda de pasta sozinho — dano silencioso em disco.
        drag.threshold: 8

        onClicked: function(mouse) {
            row.entryClicked(row.path, row.kind);
            if (mouse.button === Qt.RightButton) {
                const pt = entryArea.mapToItem(null, mouse.x, mouse.y);
                row.contextMenuRequested(row.path, row.kind, row.name, pt.x, pt.y);
                return;
            }
            if (row.kind === "directory") {
                row.entryToggled(row.path, row.index, row.expanded);
            } else if (row.kind === "file") {
                row.entryOpened(row.path);
            }
        }

        onReleased: {
            if (dragProxy.Drag.active) {
                dragProxy.Drag.drop();
            }
            // O proxy anda com o cursor; devolve-lo evita a linha nascer torta
            // no proximo arraste (o delegate e reciclado pelo ListView).
            dragProxy.x = 0;
            dragProxy.y = 0;
        }

        drag.onActiveChanged: {
            if (entryArea.drag.active) {
                row.dragStarted(row.path);
            } else {
                row.dragEnded();
            }
        }
    }

    KvIconButton {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingXSmall
        anchors.verticalCenter: parent.verticalCenter
        width: 22
        height: 22
        z: 2
        visible: row.isRunnableScript(row.name, row.kind)
                 && (entryArea.containsMouse || row.path === row.selectedPath)
        enabled: visible
        iconName: "run"
        iconSize: 13
        primary: true
        tooltip: qsTr("Executar script")
        onClicked: row.scriptRunRequested(row.path)
    }
}
