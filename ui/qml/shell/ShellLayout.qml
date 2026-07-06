import QtQuick

Item {
    id: root

    default property alias contentItems: contentRow.data

    Row {
        id: contentRow

        anchors.fill: parent
        spacing: Theme.panelGap
    }
}
