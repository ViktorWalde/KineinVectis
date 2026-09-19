pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A GRADE comum dos paineis de ambiente (Etapa 2 F8): cabecalho fixo,
// colunas com a largura da regra pura (GridRules), `null` em italico,
// rolagem nos dois sentidos quando nao cabe. Nasceu da grade do banco
// (DataSourceQuery) e serve a containers, portas seriais e sondas.
Item {
    id: root

    // [{ key, label }] — `label` opcional (cai no `key`).
    property var columns: []
    // Linhas: objetos (por `key`) ou arrays (por posicao).
    property var rows: []
    property string emptyText: ""
    property bool mono: true
    property int maxHeight: 220
    property int rowHeight: 20
    // Selecao por linha (E3-6): -1 = nenhuma; quem escuta `rowClicked`
    // decide o que a linha escolhida permite (a barra de acoes).
    property int selectedIndex: -1
    property bool selectable: false

    signal rowClicked(int index)

    readonly property var widths: rules.columnWidths(columns, rows, width)
    readonly property int contentWidth: widths.reduce((sum, w) => sum + w + 1, 0)

    implicitHeight: columns.length === 0
                    ? (emptyText === "" ? 0 : vazio.implicitHeight)
                    : Math.min(maxHeight, rowHeight + 1 + rows.length * (rowHeight + 1))

    GridRules { id: rules }

    Text {
        id: vazio

        width: parent.width
        visible: root.columns.length === 0 && root.emptyText !== ""
        text: root.emptyText
        color: Theme.textMuted
        font.pixelSize: 11
        wrapMode: Text.WordWrap
    }

    Row {
        id: cabecalho

        visible: root.columns.length > 0
        x: -corpo.contentX
        spacing: 1
        clip: true

        Repeater {
            model: root.columns

            delegate: Rectangle {
                id: celulaCabecalho

                required property var modelData
                required property int index

                width: root.widths[index]
                height: root.rowHeight
                color: Theme.surface2

                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 5
                    verticalAlignment: Text.AlignVCenter
                    text: celulaCabecalho.modelData.label !== undefined
                          ? celulaCabecalho.modelData.label : celulaCabecalho.modelData.key
                    color: Theme.textPrimary
                    font.family: root.mono ? Theme.monoFont : ""
                    font.pixelSize: 10
                    font.weight: Font.DemiBold
                    elide: Text.ElideRight
                }
            }
        }
    }

    Flickable {
        id: corpo

        anchors.top: cabecalho.bottom
        anchors.topMargin: 1
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        visible: root.columns.length > 0
        clip: true
        contentWidth: Math.max(width, root.contentWidth)
        contentHeight: linhas.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: linhas

            spacing: 1

            Repeater {
                model: root.rows

                delegate: Row {
                    id: linha

                    required property var modelData
                    required property int index

                    readonly property bool selected: root.selectable && root.selectedIndex === index

                    spacing: 1

                    Repeater {
                        model: root.columns

                        delegate: Rectangle {
                            id: celula

                            required property var modelData
                            required property int index

                            readonly property var value: rules.cellOf(linha.modelData, modelData, index)

                            width: root.widths[index]
                            height: root.rowHeight
                            color: linha.selected ? Theme.surfaceSelected
                                   : (area.containsMouse ? Theme.surface2 : Theme.background1)

                            Text {
                                anchors.fill: parent
                                anchors.leftMargin: 5
                                verticalAlignment: Text.AlignVCenter
                                text: rules.cellText(celula.value)
                                font.italic: rules.isNull(celula.value)
                                color: rules.isNull(celula.value) ? Theme.textMuted
                                       : (linha.selected ? Theme.textPrimary : Theme.textSecondary)
                                font.family: root.mono ? Theme.monoFont : ""
                                font.pixelSize: 10
                                elide: Text.ElideRight
                            }

                            // O realce ao pairar (a linha inteira acende) e, se a
                            // grade e' selecionavel, o clique escolhe a linha.
                            MouseArea {
                                id: area

                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: root.selectable ? Qt.LeftButton : Qt.NoButton
                                cursorShape: root.selectable ? Qt.PointingHandCursor : Qt.ArrowCursor
                                onClicked: root.rowClicked(linha.index)
                            }
                        }
                    }
                }
            }
        }
    }
}
