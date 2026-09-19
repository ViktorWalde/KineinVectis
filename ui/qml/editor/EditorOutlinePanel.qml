pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property var items: []
    property alias collapsed: outlineController.collapsed
    readonly property int visibleCount: outlineController.count
    // A busca de simbolos no projeto (E3-2): com texto, a lista vira os
    // resultados — "nesta pasta" e "no projeto"; sem texto, a estrutura.
    property var symbols: null

    signal openRequested(int line, int column)
    signal collapseRequested()

    // Com `query`, o campo ja' chega preenchido e buscando (Alt+7 passa "").
    function focusSearch(query) {
        if (query !== undefined && query !== "") {
            campoBusca.text = query;
            if (root.symbols) root.symbols.setQuery(query);
        }
        campoBusca.forceActiveFocus();
        campoBusca.selectAll();
    }

    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1
    radius: Theme.radius

    function nodeKey(parentKey, node, index) {
        return outlineController.nodeKey(parentKey, node, index);
    }

    function rebuild() {
        outlineController.rebuild();
    }

    function toggle(key) {
        outlineController.toggle(key);
    }

    EditorOutlineController {
        id: outlineController

        items: root.items
    }

    Column {
        anchors.fill: parent
        anchors.margins: 1
        spacing: 0

        Rectangle {
            width: parent.width
            height: 30
            color: Theme.surface1

            Text {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Símbolos")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeTree
                font.bold: true
            }

            KvIconButton {
                anchors.right: parent.right
                anchors.rightMargin: Theme.spacingXSmall
                anchors.verticalCenter: parent.verticalCenter
                width: 24
                height: 24
                compact: true
                iconName: "chevron-down"
                tooltip: qsTr("Recolher Estrutura")
                onClicked: root.collapseRequested()
            }
        }

        // O campo: por nome, no projeto inteiro (o indice) e na pasta do arquivo.
        Rectangle {
            width: parent.width
            height: 30
            color: Theme.background0

            TextInput {
                id: campoBusca

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.pixelSize: 11
                clip: true
                selectByMouse: true
                onTextEdited: if (root.symbols) root.symbols.setQuery(text)
                onAccepted: {
                    if (root.symbols && root.symbols.folderResults.length > 0) root.symbols.open(root.symbols.folderResults[0]);
                    else if (root.symbols && root.symbols.results.length > 0) root.symbols.open(root.symbols.results[0]);
                }
                Keys.onEscapePressed: { text = ""; if (root.symbols) root.symbols.setQuery(""); }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: campoBusca.text === ""
                    text: qsTr("Buscar função, tipo… no projeto")
                    color: Theme.textMuted
                    font.pixelSize: 11
                }
            }
        }

        // Os resultados da busca (E3-2): a pasta do arquivo, depois o projeto.
        SymbolResultsList {
            id: symbolList

            width: parent.width
            height: visible ? parent.height - 60 : 0
            visible: root.symbols && root.symbols.active
            symbols: root.symbols
        }

        ListView {
            id: outlineList

            width: parent.width
            height: visible ? parent.height - 60 : 0
            visible: !(root.symbols && root.symbols.active)
            clip: true
            model: outlineController.model

            delegate: Rectangle {
                id: outlineRow

                required property string nodeKey
                required property string displayName
                required property string symbolKind
                required property int targetLine
                required property int targetColumn
                required property int depth
                required property bool hasChildren
                required property bool expanded

                width: ListView.view.width
                height: 25
                color: rowMouse.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    id: disclosure

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingXSmall + outlineRow.depth * 14
                    anchors.verticalCenter: parent.verticalCenter
                    width: 14
                    visible: outlineRow.hasChildren
                    text: outlineRow.expanded ? "▾" : "▸"
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeTree - 1
                    z: 2

                    MouseArea {
                        anchors.fill: parent
                        anchors.margins: -3
                        onClicked: function(mouse) {
                            root.toggle(outlineRow.nodeKey);
                            mouse.accepted = true;
                        }
                    }
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingXSmall + outlineRow.depth * 14 + 16
                    anchors.right: kindLabel.left
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    text: outlineRow.displayName
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeTree - 1
                    elide: Text.ElideRight
                }

                Text {
                    id: kindLabel

                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    text: outlineRow.symbolKind
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeTree - 3
                }

                MouseArea {
                    id: rowMouse

                    anchors.fill: parent
                    hoverEnabled: true
                    z: 1
                    onClicked: root.openRequested(outlineRow.targetLine,
                                                  outlineRow.targetColumn)
                }
            }

            Text {
                anchors.centerIn: parent
                visible: outlineController.count === 0
                text: qsTr("Sem símbolos locais")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeTree - 1
            }
        }
    }
}
