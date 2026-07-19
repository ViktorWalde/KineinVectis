import QtQuick

Item {
    id: root

    default property alias contentItems: contentRow.data

    Row {
        id: contentRow

        anchors.fill: parent
        // §4.2: 1px de fundo da janela entre as regioes E' o divisor.
        spacing: Theme.seamWidth
    }
}
