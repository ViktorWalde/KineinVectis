pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CABECALHO da grade comum (saiu do KvDataGrid em 2026-10-03, quando ganhou
// a alca de largura): o nome de cada coluna, a direita nas de numero, e na
// borda direita a alca — arrastar muda a largura, clique duplo volta ao
// natural. As larguras e o que foi arrastado sao do KvDataGrid (`grid`); na
// grade que ordena, o clique no nome alterna a ordem e o icone diz o sentido.
Row {
    id: root

    property var grid: null

    spacing: 1
    clip: true

    Repeater {
        model: root.grid.columns

        delegate: Rectangle {
            id: headerCell

            required property var modelData
            required property int index

            // A coluna da ordem: "asc", "desc" ou "" (passo 14 da 0.3.9).
            readonly property string direction: root.grid.sortable && root.grid.sorting.column === index
                                                ? root.grid.sorting.direction : ""

            width: root.grid.widths[index]
            height: root.grid.rowHeight
            color: sortArea.containsMouse ? Theme.surfaceSelected : Theme.surface2

            Text {
                anchors.fill: parent
                anchors.leftMargin: 5
                anchors.rightMargin: headerCell.direction === "" ? 5 : 5 + sortIcon.width + 2
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: root.grid.numeric[headerCell.index] ? Text.AlignRight : Text.AlignLeft
                text: headerCell.modelData.label !== undefined
                      ? headerCell.modelData.label : headerCell.modelData.key
                textFormat: Text.PlainText
                color: Theme.textPrimary
                font.family: root.grid.mono ? Theme.monoFont : ""
                font.pixelSize: Theme.fontSizeCaption
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }

            KvIcon {
                id: sortIcon

                anchors.right: parent.right
                anchors.rightMargin: 5
                anchors.verticalCenter: parent.verticalCenter
                visible: headerCell.direction !== ""
                size: 10
                name: root.grid.sorting.gridRules.isDescending(headerCell.direction) ? "chevron-down" : "chevron-up"
                active: true
            }

            // O clique no NOME ordena (so' na grade que ordena); a alca, por
            // cima, continua sendo da largura.
            MouseArea {
                id: sortArea

                anchors.fill: parent
                enabled: root.grid.sortable
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.grid.sorting.toggle(headerCell.index)
            }

            // A ALCA de largura, na borda direita do cabecalho: arrastar
            // muda a coluna; clique duplo volta ao natural.
            MouseArea {
                property real startX: 0
                property real startWidth: 0

                anchors.right: parent.right
                anchors.rightMargin: -3
                width: 7
                height: parent.height
                z: 2
                hoverEnabled: true
                cursorShape: Qt.SplitHCursor
                preventStealing: true
                onPressed: mouse => {
                    startX = mapToItem(root, mouse.x, 0).x;
                    startWidth = headerCell.width;
                }
                onPositionChanged: mouse => {
                    if (pressed) root.grid.resizeColumn(headerCell.index,
                                                        startWidth + mapToItem(root, mouse.x, 0).x - startX);
                }
                onDoubleClicked: root.grid.autoFitColumn(headerCell.index)
            }
        }
    }
}
