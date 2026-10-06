pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Linhas prontas do DataSourceTree. Mouse e teclado compartilham seleção;
// consulta, edição e releitura seguem os donos existentes.
ListView {
    id: root

    property var controller: null
    property var treeModel: null
    property var actions: null
    signal contextMenuRequested(string key, real x, real y)
    signal newRequested()
    readonly property int selectedRowIndex: root.treeModel ? root.treeModel.selectedIndex : -1
    currentIndex: root.selectedRowIndex
    keyNavigationEnabled: false
    onSelectedRowIndexChanged: Qt.callLater(root.revealSelection)
    onModelChanged: Qt.callLater(root.revealSelection)

    function revealSelection() {
        if (root.selectedRowIndex >= 0 && root.selectedRowIndex < root.count)
            root.positionViewAtIndex(root.selectedRowIndex, ListView.Contain);
    }

    function focusTree() { root.forceActiveFocus(); root.revealSelection(); }

    function selectRow(row) { root.treeModel.select(row.key); root.forceActiveFocus(); }

    function menuForSelection() {
        const row = root.treeModel.selectedRow;
        if (row) root.contextMenuRequested(row.key, 20,
            Math.max(0, Math.min(root.height - 26, root.selectedRowIndex * 26 - root.contentY)) + 26);
    }

    function isMenuKey(event) {
        return event.key === Qt.Key_Menu || event.key === Qt.Key_F10 && event.modifiers === Qt.ShiftModifier;
    }

    // Shift+F10 é Executar fora da árvore; F5 pode ser Depurar. Com foco
    // aqui, o gesto pertence ao objeto selecionado e não chega ao atalho global.
    Keys.onShortcutOverride: event => {
        if (root.isMenuKey(event) || event.key === Qt.Key_F5 && event.modifiers === Qt.NoModifier)
            event.accepted = true;
    }

    function handleKey(event) {
        const row = root.treeModel.selectedRow;
        event.accepted = true;
        if (event.key === Qt.Key_Down) root.treeModel.moveSelection(root.selectedRowIndex + 1);
        else if (event.key === Qt.Key_Up) root.treeModel.moveSelection(root.selectedRowIndex - 1);
        else if (event.key === Qt.Key_Home) root.treeModel.moveSelection(0);
        else if (event.key === Qt.Key_End) root.treeModel.moveSelection(root.count - 1);
        else if (event.key === Qt.Key_Right && row) {
            if (row.expandable && !row.expanded) root.activate(row);
            else if (row.expanded && root.selectedRowIndex + 1 < root.count
                     && root.treeModel.rows[root.selectedRowIndex + 1].depth > row.depth)
                root.treeModel.moveSelection(root.selectedRowIndex + 1);
        } else if (event.key === Qt.Key_Left && row) {
            if (row.expanded) root.treeModel.toggle(row.key);
            else root.treeModel.selectParent();
        } else if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && row) {
            if (DataSourceKinds.hasData(row.kind)) root.openData(row);
            else root.activate(row);
        } else if (event.key === Qt.Key_F5 && event.modifiers === Qt.NoModifier) root.actions.dispatch("database.refresh", row);
        else if (root.isMenuKey(event)) root.menuForSelection();
        else event.accepted = false;
    }

    Keys.onPressed: event => root.handleKey(event)

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
            if (DataSourceKinds.isConnection(row.kind) && row.expanded === false
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

    Column {
        anchors.centerIn: parent
        width: parent.width - 2 * Theme.spacingMedium
        visible: root.count === 0
        spacing: Theme.spacingSmall
        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: qsTr("Nenhuma conexão")
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeSmall
        }
        KvButton {
            anchors.horizontalCenter: parent.horizontalCenter
            text: qsTr("Criar conexão… (Alt+Insert)")
            onClicked: root.newRequested()
        }
    }

    delegate: Rectangle {
        id: treeRow

        required property var modelData

        readonly property bool selected: root.treeModel !== null && root.treeModel.selectedRow !== null
                                         && root.treeModel.selectedRow.key === treeRow.modelData.key
        readonly property bool leaf: !treeRow.modelData.expandable
        readonly property bool action: treeRow.modelData.kind === "read" || treeRow.modelData.kind === "failed"
        readonly property bool connection: DataSourceKinds.isConnection(treeRow.modelData.kind)
        readonly property bool hasData: DataSourceKinds.hasData(treeRow.modelData.kind)

        width: root.width - treeScrollBar.width
        height: 26
        radius: Theme.radius
        color: treeRow.selected ? Theme.surfaceSelected
               : rowHover.hovered ? Theme.surface2 : "transparent"
        border.width: root.activeFocus && treeRow.selected ? 1 : 0
        border.color: Theme.accent
        Accessible.role: Accessible.TreeItem
        Accessible.name: treeRow.modelData.name
        Accessible.selected: treeRow.selected
        Accessible.onPressAction: { root.selectRow(treeRow.modelData); root.activate(treeRow.modelData); }

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
            iconColor: treeRow.connection ? DataSourceKinds.engineColor(treeRow.modelData.engine) : Theme.iconDefault
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
                onClicked: { root.selectRow(treeRow.modelData); root.openData(treeRow.modelData); }
            }

            KvIconButton {
                visible: treeRow.connection
                compact: true
                iconName: "terminal"
                iconSize: 14
                tooltip: qsTr("Abrir o console SQL no editor")
                onClicked: { root.selectRow(treeRow.modelData); root.consoleRequested(treeRow.modelData.connection); }
            }

            KvIconButton {
                visible: treeRow.connection
                compact: true
                iconName: "settings"
                iconSize: 14
                tooltip: qsTr("Editar a conexão")
                onClicked: { root.selectRow(treeRow.modelData); root.editRequested(treeRow.modelData.connection); }
            }
        }

        MouseArea {
            id: rowArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: treeRow.modelData.expandable || treeRow.action ? Qt.PointingHandCursor : Qt.ArrowCursor
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: mouse => {
                root.selectRow(treeRow.modelData);
                if (mouse.button === Qt.RightButton) {
                    const point = treeRow.mapToItem(root, mouse.x, mouse.y);
                    root.contextMenuRequested(treeRow.modelData.key, point.x, point.y);
                } else root.activate(treeRow.modelData);
            }
            onDoubleClicked: mouse => { if (mouse.button === Qt.LeftButton) root.openData(treeRow.modelData); }
        }
    }
}
