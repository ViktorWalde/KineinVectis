pragma ComponentBehavior: Bound
import QtQuick

// Painel da arvore de projeto: cabecalho + lista. A linha vive no
// `ProjectTreeRow`; a decisao de gesto, no `ProjectTreeGestures`.
//
// `gestures` entra aqui como CONSULTA (o alvo do arraste precisa saber, no
// hover, se aceita o drop — e sincrono). Toda ORDEM sai como sinal, para o
// painel nao virar dono de logica.
Rectangle {
    id: root

    property string workspaceName: ""
    property string workspaceKindLabel: ""
    property string selectedPath: ""
    property int selectedIndex: -1
    property var entriesModel
    property var gestures: null
    property string moveError: ""
    // path absoluto -> kind do git (fatia M3.1); a revisão força rebind.
    property var gitKinds: ({})
    property int gitRevision: 0
    // Caminho sendo arrastado agora; a linha de origem fica esmaecida.
    property string draggingPath: ""

    signal refreshRequested()
    signal closeRequested()
    signal entrySelected(string path, string kind)
    signal directoryToggleRequested(string path, int index, bool expanded)
    signal fileOpenRequested(string path)
    signal scriptRunRequested(string path)
    signal contextMenuRequested(string path, string kind, string name,
                                real sceneX, real sceneY)
    signal dropRequested(string sourcePath, string targetPath, string targetKind)
    signal moveSelectionRequested(int delta)
    signal expandSelectedRequested()
    signal collapseSelectedRequested()
    signal activateSelectedRequested()
    signal deleteSelectedRequested()
    signal focusEditorRequested()

    function focusTree() {
        explorerView.forceActiveFocus();
    }

    // §4.2: painel arredondado SEM contorno sobre o fundo da janela; a
    // separacao vem do divisor de 1px do layout.
    implicitWidth: 260
    radius: Theme.radiusLarge
    color: Theme.background1

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        // Cabecalho por ANCORA, nao Row com espacador calculado: nome longo
        // ELIDE em vez de empurrar os chips por baixo do chip do build system
        // (sobreposicao reportada pelo autor em 2026-07-18). Criar arquivo e
        // pasta saiu daqui a pedido dele: o contexto (botao direito) e o menu
        // Arquivo ja cobrem; ficaram atualizar e fechar.
        Item {
            id: headerRow

            width: parent.width
            height: 24

            Row {
                id: headerChips

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                KvIconButton {
                    width: 22
                    height: 22
                    iconName: "refresh"
                    iconSize: 15
                    tooltip: qsTr("Atualizar projeto")
                    onClicked: root.refreshRequested()
                }

                KvIconButton {
                    width: 22
                    height: 22
                    iconName: "close"
                    iconSize: 14
                    danger: true
                    tooltip: qsTr("Fechar workspace")
                    onClicked: root.closeRequested()
                }
            }

            Row {
                id: headerTitle

                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                // O nome do projeto E' o no raiz: soltar sobre ele move para a
                // raiz do workspace. Sem isto nao ha como tirar um arquivo de
                // uma subpasta, porque a raiz nunca e uma LINHA da arvore.
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: workspaceLabel.width + 8
                    height: 18
                    radius: Theme.radius
                    color: rootDrop.containsDrag && rootDrop.acceptsSource
                           ? Theme.accentDim : "transparent"
                    border.width: rootDrop.containsDrag && rootDrop.acceptsSource
                                  ? 1 : 0
                    border.color: Theme.accent

                    Text {
                        id: workspaceLabel

                        anchors.centerIn: parent
                        // O que sobra do cabecalho depois do chip do build
                        // system e dos botoes; alem disso, elide.
                        width: Math.min(implicitWidth,
                                        headerRow.width - headerChips.width
                                        - kindChip.width - 8
                                        - 3 * Theme.spacingSmall)
                        text: root.workspaceName
                        color: Theme.textPrimary
                        font.pixelSize: 13
                        font.bold: true
                        elide: Text.ElideRight
                    }

                    DropArea {
                        id: rootDrop

                        anchors.fill: parent
                        keys: ["kinein/tree-entry"]

                        readonly property ProjectTreeRow sourceRow:
                            drag.source as ProjectTreeRow
                        readonly property string sourcePath:
                            sourceRow ? sourceRow.path : ""
                        readonly property bool acceptsSource:
                            root.gestures !== null && sourcePath !== ""
                            && root.gestures.canDropOn(sourcePath, "", "directory")

                        onDropped: function(drop) {
                            if (acceptsSource) {
                                root.dropRequested(sourcePath, "", "directory");
                                drop.accept();
                            }
                        }
                    }
                }

                Rectangle {
                    id: kindChip

                    anchors.verticalCenter: parent.verticalCenter
                    width: kindText.width + 10
                    height: 16
                    radius: 8
                    color: Theme.accentDim

                    Text {
                        id: kindText

                        anchors.centerIn: parent
                        text: root.workspaceKindLabel
                        color: Theme.accent
                        font.pixelSize: 9
                        font.bold: true
                    }
                }
            }
        }

        // Falha de arraste nao pode abrir dialogo (o usuario nao pediu nenhum),
        // mas tambem nao pode sumir em silencio: o arquivo simplesmente nao se
        // moveu e a causa (quase sempre nome ja existente no destino) fica aqui.
        Rectangle {
            id: moveErrorStrip

            width: parent.width
            height: visible ? 22 : 0
            visible: root.moveError !== ""
            radius: Theme.radius
            color: Theme.errorSoft

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingSmall
                text: root.moveError
                color: Theme.textPrimary
                font.pixelSize: 11
                elide: Text.ElideRight
            }
        }

        ListView {
            id: explorerView

            width: parent.width
            height: parent.height - y
            clip: true
            model: root.entriesModel
            focus: true
            currentIndex: root.selectedIndex
            // O ListView navega por setas SOZINHO. Somadas aos handlers abaixo,
            // a selecao andaria duas linhas por tecla e o `currentIndex` teria
            // dois donos — o que quebraria o binding acima em silencio. Aqui a
            // arvore tem um dono so': o ProjectTreeGestures.
            keyNavigationEnabled: false

            // Teclado JetBrains: setas navegam, ←/→ fecham/abrem, Enter abre,
            // Delete exclui, Esc devolve o foco ao editor.
            Keys.onUpPressed: root.moveSelectionRequested(-1)
            Keys.onDownPressed: root.moveSelectionRequested(1)
            Keys.onLeftPressed: root.collapseSelectedRequested()
            Keys.onRightPressed: root.expandSelectedRequested()
            Keys.onReturnPressed: root.activateSelectedRequested()
            Keys.onEnterPressed: root.activateSelectedRequested()
            Keys.onDeletePressed: root.deleteSelectedRequested()
            Keys.onEscapePressed: root.focusEditorRequested()

            // Navegar por teclado sem rolar a vista deixa a selecao fora da
            // tela: o usuario perde de vista o proprio cursor.
            onCurrentIndexChanged: {
                if (currentIndex >= 0) {
                    positionViewAtIndex(currentIndex, ListView.Contain);
                }
            }

            delegate: ProjectTreeRow {
                id: treeRow

                width: explorerView.width
                gestures: root.gestures
                selectedPath: root.selectedPath
                gitKinds: root.gitKinds
                gitRevision: root.gitRevision
                dragging: root.draggingPath !== ""
                          && root.draggingPath === treeRow.path

                onEntryClicked: function(entryPath, entryKind) {
                    explorerView.forceActiveFocus();
                    root.entrySelected(entryPath, entryKind);
                }
                onEntryToggled: function(entryPath, entryIndex, entryExpanded) {
                    root.directoryToggleRequested(entryPath, entryIndex, entryExpanded);
                }
                onEntryOpened: function(entryPath) {
                    root.fileOpenRequested(entryPath);
                }
                onScriptRunRequested: function(entryPath) {
                    root.scriptRunRequested(entryPath);
                }
                onContextMenuRequested: function(entryPath, entryKind, entryName,
                                                 sceneX, sceneY) {
                    root.contextMenuRequested(entryPath, entryKind, entryName,
                                              sceneX, sceneY);
                }
                onDragStarted: function(entryPath) {
                    root.draggingPath = entryPath;
                }
                onDragEnded: root.draggingPath = ""
                onDropped: function(sourcePath, targetPath, targetKind) {
                    root.dropRequested(sourcePath, targetPath, targetKind);
                }
            }
        }
    }
}
