pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Column {
    id: root

    property var actions: null
    signal newMenuRequested(real x, real y)
    signal closeRequested()
    width: parent.width
    spacing: Theme.spacingXSmall

    Row {
        width: parent.width
        height: 24
        Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - 24
            text: qsTr("Banco")
            color: Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            font.weight: Font.DemiBold
        }
        KvIconButton {
            compact: true
            iconName: "close"
            tooltip: qsTr("Fechar a janela do Banco")
            onClicked: root.closeRequested()
        }
    }

    Row {
        height: 24
        spacing: Theme.spacingXSmall
        KvIconButton {
            id: addButton
            compact: true
            iconName: "add"
            tooltip: qsTr("Criar conexão… (Alt+Insert)")
            onClicked: {
                const point = addButton.mapToItem(root, 0, addButton.height);
                root.newMenuRequested(point.x, point.y);
            }
        }
        KvIconButton {
            compact: true
            iconName: "refresh"
            tooltip: qsTr("Ler estrutura de novo (F5)")
            enabled: root.actions !== null && root.actions.canRefresh(root.actions.selectedRow)
            onClicked: root.actions.dispatch("database.refresh", root.actions.selectedRow)
        }
        KvIconButton {
            compact: true
            iconName: "terminal"
            tooltip: qsTr("Abrir console da conexão")
            enabled: root.actions !== null && root.actions.selectedRow !== null
                     && (DataSourceKinds.isConnection(root.actions.selectedRow.kind) || DataSourceKinds.hasData(root.actions.selectedRow.kind))
            onClicked: root.actions.dispatch("database.console", root.actions.selectedRow)
        }
        KvIconButton {
            compact: true
            iconName: "table"
            tooltip: qsTr("Ver dados")
            enabled: root.actions !== null && root.actions.selectedRow !== null && DataSourceKinds.hasData(root.actions.selectedRow.kind)
            onClicked: root.actions.dispatch("database.data", root.actions.selectedRow)
        }
        KvIconButton {
            compact: true
            iconName: "collapse"
            tooltip: qsTr("Recolher tudo")
            enabled: root.actions !== null && root.actions.treeModel.rows.some(row => row.expanded)
            onClicked: root.actions.dispatch("database.collapse", null)
        }
    }
}
