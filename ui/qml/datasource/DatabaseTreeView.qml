pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A ARVORE desenhada da janela do Banco (2026-10-03): as linhas vem prontas
// do DataSourceTree; aqui so' o desenho e o mouse. Clique abre/fecha (e le a
// estrutura da conexao se ainda nao leu); clique duplo numa tabela, ou o
// botao "ver dados" que aparece com o mouse em cima, traz as linhas dela
// para a secao de dados da propria janela.
ListView {
    id: root

    property var controller: null
    property var treeModel: null

    signal consoleRequested(string name)
    signal editRequested(string name)
    signal tableDataRequested(string connection, string engine, string schema, string table, string readSql)

    clip: true
    boundsBehavior: Flickable.StopAtBounds
    model: root.treeModel ? root.treeModel.rows : []

    function activate(row) {
        if (row.kind === "read" || row.kind === "failed") {
            root.controller.introspectProfile(row.connection);
        } else if (row.expandable) {
            root.treeModel.toggle(row.key);
            if (row.kind === "connection" && row.expanded === false
                    && DataSourceMap.get(root.controller.structures, row.connection) === undefined) {
                root.controller.introspectProfile(row.connection);
            }
        }
    }

    function openData(row) {
        if (DataSourceKinds.hasData(row.kind)) {
            root.tableDataRequested(row.connection, row.engine, row.schema || "", row.table, row.readSql || "");
        }
    }

    FlickableScrollBar {
        id: treeScrollBar

        view: root
    }

    Text {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingMedium
        visible: root.count === 0
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: qsTr("Nenhuma conexão salva. Use + para criar, ou escolha uma desta máquina, abaixo.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeSmall
    }

    delegate: Rectangle {
        id: treeRow

        required property var modelData

        readonly property bool leaf: !treeRow.modelData.expandable
        readonly property bool action: treeRow.modelData.kind === "read" || treeRow.modelData.kind === "failed"
        readonly property bool connection: treeRow.modelData.kind === "connection"
        readonly property bool hasData: DataSourceKinds.hasData(treeRow.modelData.kind)

        width: root.width - treeScrollBar.width
        height: 26
        radius: Theme.radius
        color: rowHover.hovered ? Theme.surface2 : "transparent"

        // O hover da LINHA e' passivo: com o mouse no botao de acao a linha
        // continua "em cima". Com o containsMouse do MouseArea o botao roubava
        // o hover, sumia, devolvia e voltava — piscava (2026-10-03, autor).
        HoverHandler {
            id: rowHover
        }

        Text {
            id: chevron

            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingXSmall + treeRow.modelData.depth * 14
            anchors.verticalCenter: parent.verticalCenter
            width: 12
            text: treeRow.modelData.expandable ? (treeRow.modelData.expanded ? "▾" : "▸") : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }

        KvIcon {
            id: rowIcon

            anchors.left: chevron.right
            anchors.leftMargin: 2
            anchors.verticalCenter: parent.verticalCenter
            visible: name !== ""
            size: 16
            name: treeRow.connection ? DataSourceKinds.engineIcon(treeRow.modelData.engine)
                                     : DataSourceKinds.kindIcon(treeRow.modelData.kind)
            active: treeRow.connection
            warning: treeRow.modelData.kind === "failed"
        }

        Text {
            anchors.left: rowIcon.visible ? rowIcon.right : chevron.right
            anchors.leftMargin: Theme.spacingXSmall
            anchors.right: rowActions.visible ? rowActions.left : detailText.left
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            text: treeRow.modelData.name
            textFormat: Text.PlainText
            color: treeRow.action ? Theme.accent : Theme.textPrimary
            font.family: treeRow.leaf && !treeRow.action ? Theme.monoFont : Theme.uiFont
            font.pixelSize: Theme.fontSizeSmall
            font.weight: treeRow.connection ? Font.DemiBold : Font.Normal
            elide: Text.ElideRight
        }

        Text {
            id: detailText

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            visible: !rowActions.visible
            width: Math.min(implicitWidth, treeRow.width * 0.45)
            horizontalAlignment: Text.AlignRight
            text: treeRow.modelData.detail
            textFormat: Text.PlainText
            color: treeRow.modelData.production === true ? Theme.errorSoft : Theme.textMuted
            font.family: treeRow.leaf ? Theme.monoFont : Theme.uiFont
            font.pixelSize: Theme.fontSizeMicro
            elide: Text.ElideLeft
        }

        // Com o mouse em cima: na conexao, o console e o editar; na tabela,
        // ver os dados (o mesmo do clique duplo, achavel).
        Row {
            id: rowActions

            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingXSmall
            anchors.verticalCenter: parent.verticalCenter
            z: 2
            visible: (treeRow.connection || treeRow.hasData) && rowHover.hovered
            spacing: 2

            KvIconButton {
                visible: treeRow.hasData
                compact: true
                iconName: "table"
                iconSize: 14
                tooltip: qsTr("Ver os dados (clique duplo)")
                onClicked: root.openData(treeRow.modelData)
            }

            KvIconButton {
                visible: treeRow.connection
                compact: true
                iconName: "terminal"
                iconSize: 14
                tooltip: qsTr("Abrir o console SQL no editor")
                onClicked: root.consoleRequested(treeRow.modelData.connection)
            }

            KvIconButton {
                visible: treeRow.connection
                compact: true
                iconName: "settings"
                iconSize: 14
                tooltip: qsTr("Editar a conexão")
                onClicked: root.editRequested(treeRow.modelData.connection)
            }
        }

        MouseArea {
            id: rowArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: treeRow.modelData.expandable || treeRow.action ? Qt.PointingHandCursor : Qt.ArrowCursor
            onClicked: root.activate(treeRow.modelData)
            onDoubleClicked: root.openData(treeRow.modelData)
        }
    }
}
