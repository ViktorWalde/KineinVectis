pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property string workspaceName: ""
    property var selectedPaths: []
    property var entriesModel
    property var projectTree: null
    // path absoluto -> kind do git (fatia M3.1); a revisão força rebind.
    property var gitKinds: ({})
    property int gitRevision: 0
    property real dragEdgeY: -1
    readonly property bool dropToRoot: rootDrop.validDrag

    signal createFileRequested()
    signal createDirectoryRequested()
    signal refreshRequested()
    signal closeRequested()
    signal entrySelected(string path, string kind, int modifiers)
    signal selectAllRequested()
    signal directoryToggleRequested(string path, int index, bool expanded)
    signal fileOpenRequested(string path)
    signal scriptRunRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)
    signal copyRequested()
    signal cutRequested()
    signal pasteRequested()
    signal filesDropped(var paths, string destination, bool copy, bool external)

    function focusTree() {
        explorerView.forceActiveFocus();
    }

    // O que esta' sendo arrastado agora: as linhas de origem esmaecem.
    property var activeDragPaths: []

    function dragSources(path) {
        const selected = root.selectedPaths.indexOf(path) >= 0
                         ? root.selectedPaths : [path];
        const paths = selected.filter(candidate => !ProjectDragRules.insideAny(
            candidate, selected.filter(other => other !== candidate)));
        return paths.length <= 128 ? paths : [];
    }

    // A pasta que recebe o que for solto numa linha: ela mesma, ou a pasta do
    // arquivo. Um dono: a linha mostra o nome e o soltar usa o caminho.
    function dropDirectory(path, kind) {
        return treeRules.isDirectory(kind) ? path : root.projectTree.parentDir(path);
    }

    function trackDrag(sceneY) {
        dragEdgeY = explorerView.mapFromItem(null, 0, sceneY).y;
    }

    // `revision` existe so' para o binding reavaliar quando o git muda.
    // Arquivo SEM estado de git nao e' um estado de git: cai na cor normal da
    // arvore, e por isso este caso fica aqui e nao no StatusColors.
    function gitFileColor(path, revision) {
        const kind = gitKinds[path];
        return kind === undefined ? Theme.textSecondary : StatusColors.gitKind(kind);
    }

    implicitWidth: 260
    radius: Theme.radiusLarge
    color: Theme.background1
    // Dentro da ilha unica (0.3.9): sem borda propria, so' a de "soltar aqui".
    border.color: Theme.accent
    border.width: dropToRoot ? 2 : 0

    ProjectTreeAutoScroll {
        view: explorerView
        edgeY: root.dragEdgeY
    }

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            // So' o nome: o que o projeto e' (Cargo + CMake) mora no widget
            // de projeto da barra principal desde a F1 — o chip amarelo aqui
            // era o segundo lugar a dizer o mesmo (pedido do autor, 2026-09-18).
            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(40, parent.width - refreshChip.width - newFileChip.width
                                    - newFolderChip.width - closeProjectChip.width
                                    - 5 * Theme.spacingSmall)
                elide: Text.ElideRight
                text: root.workspaceName
                color: Theme.textPrimary
                font.pixelSize: 12
                font.weight: Font.DemiBold
            }

            KvIconButton {
                id: newFileChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "file"
                iconSize: 15
                enabled: root.selectedPaths.length <= 1
                tooltip: enabled ? qsTr("Novo arquivo")
                                 : qsTr("Selecione um item para criar arquivo")
                onClicked: root.createFileRequested()
            }

            KvIconButton {
                id: newFolderChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "folder"
                iconSize: 15
                enabled: root.selectedPaths.length <= 1
                tooltip: enabled ? qsTr("Nova pasta")
                                 : qsTr("Selecione um item para criar pasta")
                onClicked: root.createDirectoryRequested()
            }

            KvIconButton {
                id: refreshChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "refresh"
                iconSize: 15
                tooltip: qsTr("Atualizar projeto")
                onClicked: root.refreshRequested()
            }

            KvIconButton {
                id: closeProjectChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "close"
                iconSize: 14
                danger: true
                tooltip: qsTr("Fechar projeto")
                onClicked: root.closeRequested()
            }
        }

        // "E' pasta?" tem um dono so' no projeto, e e' este.
        ProjectTreeRules {
            id: treeRules
        }

        // O TECLADO DA ARVORE (P1). Medido em 2026-09-26: nao havia nenhum —
        // quem nao usa mouse nao navegava no projeto. Ele emite os MESMOS
        // sinais que o clique, para que "mouse e teclado alcancam os mesmos
        // itens" seja verdade e nao dois caminhos que divergem.
        ProjectTreeKeyboard {
            id: teclado

            model: root.entriesModel
            currentIndex: explorerView.currentIndex

            onMoveRequested: function (indice, modifiers) {
                explorerView.currentIndex = indice;
                explorerView.positionViewAtIndex(indice, ListView.Contain);
                if ((modifiers & Qt.ControlModifier) && !(modifiers & Qt.ShiftModifier)) return;
                const linha = root.entriesModel.get(indice);
                root.entrySelected(linha.path, linha.kind, modifiers);
            }
            onToggleRequested: function (indice) {
                const linha = root.entriesModel.get(indice);
                root.directoryToggleRequested(linha.path, indice, linha.expanded);
            }
            onActivateRequested: function (indice) {
                root.fileOpenRequested(root.entriesModel.get(indice).path);
            }
            onSelectAllRequested: root.selectAllRequested()
            onCopyRequested: root.copyRequested()
            onCutRequested: root.cutRequested()
            onPasteRequested: root.pasteRequested()
            onMenuRequested: function (indice) {
                const linha = root.entriesModel.get(indice);
                if (root.selectedPaths.indexOf(linha.path) < 0) {
                    root.entrySelected(linha.path, linha.kind, Qt.NoModifier);
                }
                const item = explorerView.currentItem;
                const point = item === null
                              ? explorerView.mapToItem(null, 24, 24)
                              : item.mapToItem(null, 24, item.height / 2);
                root.contextMenuRequested(linha.path, linha.kind, linha.name,
                                          point.x, point.y);
            }
        }

        ListView {
            id: explorerView

            width: parent.width
            height: parent.height - y
            clip: true
            model: root.entriesModel
            // A ARVORE PRECISA PODER RECEBER O FOCO para ouvir tecla, e
            // precisa estar no caminho do Tab para ser alcancavel sem mouse.
            focus: true
            activeFocusOnTab: true
            keyNavigationEnabled: false
            highlightFollowsCurrentItem: true
            onActiveFocusChanged: {
                if (!explorerView.activeFocus) {
                    teclado.esquecerDigitacao();
                }
            }
            Keys.onPressed: function (evento) {
                evento.accepted = teclado.handleKey(evento);
            }

            ProjectTreeDropArea {
                id: rootDrop
                anchors.fill: parent
                z: -1
                destination: root.projectTree.workspaceRoot
                urlDecoder: Clipboard
                onDragPosition: function(y) { root.dragEdgeY = y; }
                onDragEnded: { root.dragEdgeY = -1; }
                onFilesDropped: function(paths, destination, copy, external) {
                    root.filesDropped(paths, destination, copy, external);
                }
            }

            delegate: ProjectTreeRow {
                id: treeRow

                width: explorerView.width
                selected: root.selectedPaths.indexOf(treeRow.path) >= 0
                cursorFocused: explorerView.activeFocus
                               && explorerView.currentIndex === treeRow.index
                gitColor: root.gitFileColor(treeRow.path, root.gitRevision)
                runnable: root.projectTree !== null
                          && root.projectTree.isRunnableScript(treeRow.path, treeRow.kind)
                dragPaths: root.dragSources(treeRow.path)
                activeDragPaths: root.activeDragPaths
                dropDirectory: root.dropDirectory(treeRow.path, treeRow.kind)

                onFilesDropped: function(paths, copy, external) {
                    root.dragEdgeY = -1;
                    root.activeDragPaths = [];
                    if (!external && ProjectDragRules.insideAny(treeRow.dropDirectory, paths)) return;
                    root.filesDropped(paths, treeRow.dropDirectory, copy, external);
                }
                onDragStarted: root.activeDragPaths = treeRow.dragPaths
                onDragPosition: function(sceneY) { root.trackDrag(sceneY); }
                onDragEnded: {
                    root.dragEdgeY = -1;
                    root.activeDragPaths = [];
                }
                onDirectoryHoverRequested: {
                    if (treeRules.isDirectory(treeRow.kind) && !treeRow.expanded)
                        root.directoryToggleRequested(treeRow.path,
                                                      treeRow.index, false);
                }

                onClicked: function(modifiers, button, sceneX, sceneY) {
                    explorerView.currentIndex = treeRow.index;
                    explorerView.forceActiveFocus();
                    const alreadySelected = root.selectedPaths.indexOf(treeRow.path) >= 0;
                    if (button !== Qt.RightButton || !alreadySelected) {
                        root.entrySelected(treeRow.path, treeRow.kind, modifiers);
                    }
                    if (button === Qt.RightButton) {
                        root.contextMenuRequested(treeRow.path, treeRow.kind,
                                                  treeRow.name, sceneX, sceneY);
                    } else if (!(modifiers & (Qt.ControlModifier | Qt.ShiftModifier))) {
                        if (treeRules.isDirectory(treeRow.kind)) {
                            root.directoryToggleRequested(treeRow.path, treeRow.index,
                                                          treeRow.expanded);
                        } else if (treeRules.isFile(treeRow.kind)) {
                            root.fileOpenRequested(treeRow.path);
                        }
                    }
                }
                onScriptRunRequested: root.scriptRunRequested(treeRow.path)
            }
        }
    }
}
