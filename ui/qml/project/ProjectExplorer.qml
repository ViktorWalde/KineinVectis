pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property string workspaceName: ""
    property string selectedPath: ""
    property var entriesModel
    // path absoluto -> kind do git (fatia M3.1); a revisão força rebind.
    property var gitKinds: ({})
    property int gitRevision: 0

    signal createFileRequested()
    signal createDirectoryRequested()
    signal refreshRequested()
    signal closeRequested()
    signal entrySelected(string path, string kind)
    signal directoryToggleRequested(string path, int index, bool expanded)
    signal fileOpenRequested(string path)
    signal scriptRunRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)

    // `revision` existe so' para o binding reavaliar quando o git muda.
    // Arquivo SEM estado de git nao e' um estado de git: cai na cor normal da
    // arvore, e por isso este caso fica aqui e nao no StatusColors.
    function gitFileColor(path, revision) {
        const kind = gitKinds[path];
        return kind === undefined ? Theme.textSecondary : StatusColors.gitKind(kind);
    }

    // As extensoes executaveis vem do core pelo ProjectTreeController
    // (run.capabilities): aqui so' o icone da linha, sem lista propria.
    property var runnableExtensions: []

    function isRunnableScript(name, kind) {
        if (kind !== "file") return false;
        const ponto = name.lastIndexOf(".");
        const ext = ponto < 0 ? "" : name.substring(ponto + 1).toLowerCase();
        return runnableExtensions.indexOf(ext) >= 0;
    }

    implicitWidth: 260
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

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
                tooltip: qsTr("Novo arquivo")
                onClicked: root.createFileRequested()
            }

            KvIconButton {
                id: newFolderChip

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                iconName: "folder"
                iconSize: 15
                tooltip: qsTr("Nova pasta")
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
                tooltip: qsTr("Fechar workspace")
                onClicked: root.closeRequested()
            }
        }

        // "E' pasta?" tem um dono so' no projeto, e e' este.
        ProjectTreeRules {
            id: regrasDaArvore
        }

        // O TECLADO DA ARVORE (P1). Medido em 2026-09-26: nao havia nenhum —
        // quem nao usa mouse nao navegava no projeto. Ele emite os MESMOS
        // sinais que o clique, para que "mouse e teclado alcancam os mesmos
        // itens" seja verdade e nao dois caminhos que divergem.
        ProjectTreeKeyboard {
            id: teclado

            model: root.entriesModel
            currentIndex: explorerView.currentIndex

            onMoveRequested: function (indice) {
                explorerView.currentIndex = indice;
                const linha = root.entriesModel.get(indice);
                root.entrySelected(linha.path, linha.kind);
            }
            onToggleRequested: function (indice) {
                const linha = root.entriesModel.get(indice);
                root.directoryToggleRequested(linha.path, indice, linha.expanded);
            }
            onActivateRequested: function (indice) {
                root.fileOpenRequested(root.entriesModel.get(indice).path);
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

            delegate: Rectangle {
                id: treeRow

                required property int index
                required property string path
                required property string name
                required property string kind
                required property int depth
                required property bool expanded
                required property bool machine

                width: explorerView.width
                height: 24
                radius: Theme.radius
                color: treeRow.path === root.selectedPath
                       ? Theme.surfaceSelected
                       : (rowHover.hovered ? Theme.surface2 : "transparent")
                // ONDE O TECLADO ESTA' FALANDO. Sem isto a arvore responde a
                // setas sem dizer que e' ela quem responde — e, com o painel
                // sem foco, um realce de selecao pareceria foco.
                border.width: explorerView.activeFocus
                              && explorerView.currentIndex === treeRow.index ? 1 : 0
                border.color: Theme.accent

                // Observa o delegate inteiro sem tomar eventos dos filhos. Ao
                // passar sobre o botao de executar, o hover continua ativo e
                // evita o ciclo aparece/some que fazia o atalho piscar.
                HoverHandler {
                    id: rowHover
                }

                Row {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall + treeRow.depth * 12
                    spacing: Theme.spacingXSmall

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 12
                        text: regrasDaArvore.isDirectory(treeRow.kind)
                              ? (treeRow.expanded ? "▾" : "▸") : ""
                        color: regrasDaArvore.isDirectory(treeRow.kind) && !treeRow.machine
                               ? Theme.accent : Theme.textMuted
                        font.pixelSize: Theme.fontSizeTree
                    }

                    KvFileIcon {
                        anchors.verticalCenter: parent.verticalCenter
                        size: 20
                        opacity: treeRow.machine ? 0.55 : 1
                        fileName: treeRow.name
                        directory: regrasDaArvore.isDirectory(treeRow.kind)
                        expanded: treeRow.expanded
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.max(0, treeRow.width - parent.x - x - 30)
                        text: treeRow.name
                        color: treeRow.machine ? Theme.textMuted
                               : (regrasDaArvore.isDirectory(treeRow.kind)
                                  ? Theme.textPrimary
                                  : root.gitFileColor(treeRow.path,
                                                      root.gitRevision))
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
                        // O CLIQUE MOVE O CURSOR DO TECLADO, e traz o foco
                        // para a arvore: sem isto, clicar numa linha e depois
                        // apertar a seta recomecaria de onde o teclado estava
                        // antes — um salto que ninguem pediu.
                        explorerView.currentIndex = treeRow.index;
                        explorerView.forceActiveFocus();
                        root.entrySelected(treeRow.path, treeRow.kind);
                        if (mouse.button === Qt.RightButton) {
                            const pt = entryArea.mapToItem(null, mouse.x, mouse.y);
                            root.contextMenuRequested(treeRow.path, treeRow.kind,
                                                      treeRow.name, pt.x, pt.y);
                            return;
                        }
                        if (regrasDaArvore.isDirectory(treeRow.kind)) {
                            root.directoryToggleRequested(treeRow.path, treeRow.index,
                                                          treeRow.expanded);
                        } else if (treeRow.kind === "file") {
                            root.fileOpenRequested(treeRow.path);
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
                    visible: root.isRunnableScript(treeRow.name, treeRow.kind)
                             && (rowHover.hovered
                                 || treeRow.path === root.selectedPath)
                    enabled: visible
                    iconName: "run"
                    iconSize: 13
                    primary: true
                    tooltip: qsTr("Executar script")
                    onClicked: root.scriptRunRequested(treeRow.path)
                }
            }
        }
    }
}
