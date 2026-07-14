pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property var items: []
    property alias collapsed: outlineController.collapsed
    readonly property int visibleCount: outlineController.count

    signal openRequested(int line, int column)
    signal collapseRequested()

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
                text: qsTr("Estrutura")
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

        ListView {
            id: outlineList

            width: parent.width
            height: parent.height - 30
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
