pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool hasActiveFile: false
    property bool coreConnected: false
    property bool running: false
    property bool debugging: false
    property bool windowMaximized: false
    property string workspaceName: ""
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property var recentWorkspaces: []
    property string activeMenu: ""
    // Barra UNICA (F2, 2026-07-18): os menus ficam recolhidos atras do
    // hamburguer, como no IntelliJ New UI; o segundo clique recolhe de volta.
    property bool menusExpanded: false
    // Largura que o cluster de toolbar (irmao no ShellHeaderHost) ocupa a
    // direita: o drag region para antes dele.
    property real reservedRight: 0
    readonly property real controlsWidth: windowControls.width
    readonly property string windowContextLabel: workspaceName !== ""
                                                 ? workspaceName
                                                 : qsTr("sem workspace")

    signal actionRequested(string action)
    signal menuRequested(string key, real menuX, real menuY, var items)
    signal minimizeRequested()
    signal maximizeRestoreRequested()
    signal closeWindowRequested()
    signal moveWindowRequested()

    // §4.2: a barra e a propria moldura da janela (background0, sem contorno);
    // menus e controles de janela ficam EMBUTIDOS nela, nao num cartao.
    height: 46
    color: Theme.background0
    z: 100

    function trigger(action) {
        activeMenu = "";
        actionRequested(action);
    }

    AppMenuModel {
        id: menuModel

        workspaceOpen: root.workspaceOpen
        hasActiveFile: root.hasActiveFile
        coreConnected: root.coreConnected
        running: root.running
        debugging: root.debugging
        recentWorkspaces: root.recentWorkspaces
        workspaceBuildSystems: root.workspaceBuildSystems
    }

    function toggleMenusExpanded() {
        if (menusExpanded) {
            menusExpanded = false;
            closeMenu();
            menuRequested("", 0, 0, []);
            return;
        }
        menusExpanded = true;
    }

    function closeMenu() {
        activeMenu = "";
    }

    function toggleMenu(key, item) {
        if (activeMenu === key) {
            activeMenu = "";
            menuRequested("", 0, 0, []);
            return;
        }
        activeMenu = key;
        const point = root.mapFromItem(item, 0, item.height + 2);
        menuRequested(key, point.x, point.y, menuModel.menuItems(key));
    }

    Row {
        id: menuRow

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.spacingMedium
        height: parent.height
        spacing: Theme.spacingXSmall

        // Hamburguer: desenhado com 3 retangulos (nao ha glifo no KvIcon e o
        // Canvas dele esta no teto da catraca).
        Rectangle {
            id: burgerButton

            anchors.verticalCenter: parent.verticalCenter
            width: 30
            height: 28
            radius: Theme.radius
            color: root.menusExpanded || burgerArea.containsMouse
                   ? Theme.surface2 : "transparent"

            Column {
                anchors.centerIn: parent
                spacing: 3

                Repeater {
                    model: 3

                    Rectangle {
                        width: 14
                        height: 1.6
                        radius: 1
                        color: root.menusExpanded
                               ? Theme.textPrimary : Theme.textSecondary
                    }
                }
            }

            MouseArea {
                id: burgerArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.toggleMenusExpanded()
            }
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Kinein")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        // Nome do workspace ao lado da marca (idioma IntelliJ); some quando o
        // hamburguer expande os menus, que precisam do espaco.
        Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: !root.menusExpanded
            width: Math.min(implicitWidth, 260)
            text: root.windowContextLabel
            color: Theme.textMuted
            font.pixelSize: 11
            elide: Text.ElideMiddle
        }

        Item {
            width: Theme.spacingSmall
            height: 1
        }

        Repeater {
            model: [
                { key: "file", label: qsTr("Arquivo") },
                { key: "edit", label: qsTr("Editar") },
                { key: "view", label: qsTr("Exibir") },
                { key: "navigate", label: qsTr("Navegar") },
                { key: "code", label: qsTr("Código") },
                { key: "build", label: qsTr("Build") },
                { key: "run", label: qsTr("Executar") },
                { key: "tools", label: qsTr("Ferramentas") },
                { key: "help", label: qsTr("Ajuda") }
            ]

            delegate: Rectangle {
                id: menuButton

                required property var modelData

                anchors.verticalCenter: parent.verticalCenter
                visible: root.menusExpanded
                width: visible
                       ? menuLabel.implicitWidth + 2 * Theme.spacingSmall : 0
                height: 28
                radius: Theme.radius
                color: root.activeMenu === modelData.key || menuArea.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    id: menuLabel

                    anchors.centerIn: parent
                    text: menuButton.modelData.label
                    color: root.activeMenu === menuButton.modelData.key
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 13
                }

                MouseArea {
                    id: menuArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggleMenu(menuButton.modelData.key,
                                               menuButton)
                }
            }
        }
    }

    Item {
        id: dragRegion

        x: menuRow.x + menuRow.width + Theme.spacingSmall
        // Para antes do cluster de toolbar (irmao no host) e dos controles.
        width: Math.max(0, Math.min(windowControls.x,
                                    root.width - root.reservedRight)
                        - Theme.spacingSmall - x)
        height: parent.height

        MouseArea {
            id: dragArea

            property real pressX: 0
            property real pressY: 0
            property bool systemMoveStarted: false

            anchors.fill: parent
            enabled: parent.width > 0
            acceptedButtons: Qt.LeftButton
            onPressed: function(mouse) {
                pressX = mouse.x;
                pressY = mouse.y;
                systemMoveStarted = false;
            }
            onPositionChanged: function(mouse) {
                const distance = Math.abs(mouse.x - pressX)
                               + Math.abs(mouse.y - pressY);
                if (pressed && !systemMoveStarted && distance >= 6) {
                    systemMoveStarted = true;
                    root.moveWindowRequested();
                }
            }
            onDoubleClicked: function(mouse) {
                systemMoveStarted = false;
                root.maximizeRestoreRequested();
                mouse.accepted = true;
            }
        }
    }

    WindowControls {
        id: windowControls

        anchors.top: parent.top
        anchors.right: parent.right
        height: parent.height
        maximized: root.windowMaximized
        onMinimizeRequested: root.minimizeRequested()
        onMaximizeRestoreRequested: root.maximizeRestoreRequested()
        onCloseRequested: root.closeWindowRequested()
    }
}
