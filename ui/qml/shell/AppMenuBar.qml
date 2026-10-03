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
    // O menu principal RECOLHIDO atras do ☰ (0.3.9, pedido do autor: mais
    // espaco para codigo, no modelo da JetBrains). Um clique abre os titulos
    // na propria barra; outro recolhe. A barra de ferramentas mora na MESMA
    // linha, logo depois deste bloco (ShellHeaderHost).
    property bool menuExpanded: false
    readonly property real menuEndX: menuRow.x + menuRow.width
    readonly property real controlsX: windowControls.x

    signal actionRequested(string action)
    signal menuRequested(string key, real menuX, real menuY, var items)
    signal minimizeRequested()
    signal maximizeRestoreRequested()
    signal closeWindowRequested()
    signal moveWindowRequested()

    height: 44
    color: "transparent"
    border.width: 0
    z: 100

    function trigger(action) {
        activeMenu = "";
        actionRequested(action);
    }

    function closeMenu() {
        activeMenu = "";
    }

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
    }

    function toggleMenu(key, item) {
        if (activeMenu === key) {
            activeMenu = "";
            menuRequested("", 0, 0, []);
            return;
        }
        activeMenu = key;
        const point = root.mapFromItem(item, 0, item.height + 2);
        menuRequested(key, point.x, point.y, menuItems(key));
    }

    function menuItems(key) {
        return itens.menuItems(key);
    }

    function projectMenuItems() {
        return itens.projectMenuItems();
    }

    // O QUE cada menu oferece tem dono proprio; ver AppMenuItems.qml.
    AppMenuItems {
        id: itens

        workspaceOpen: root.workspaceOpen
        hasActiveFile: root.hasActiveFile
        coreConnected: root.coreConnected
        running: root.running
        debugging: root.debugging
        workspaceBuildSystems: root.workspaceBuildSystems
        recentWorkspaces: root.recentWorkspaces
    }

    Row {
        id: menuRow

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.spacingMedium
        height: parent.height
        spacing: Theme.spacingXSmall

        // O icone da IDE no canto, como a JetBrains (pedido do autor). Uma
        // copia de 64 px do icone oficial: o de 1254 px pesa 1,4 MB.
        Image {
            anchors.verticalCenter: parent.verticalCenter
            width: 22
            height: 22
            source: "qrc:/KineinVectis/assets/icons/app/kinein-64.png"
            sourceSize.width: 64
            sourceSize.height: 64
            fillMode: Image.PreserveAspectFit
            smooth: true
            mipmap: true
        }

        KvIconButton {
            anchors.verticalCenter: parent.verticalCenter
            iconName: "menu"
            iconSize: 18
            active: root.menuExpanded
            focus: false
            focusOnClick: false
            tooltip: root.menuExpanded ? qsTr("Recolher o menu") : qsTr("Menu principal")
            onClicked: {
                if (root.activeMenu !== "") root.menuRequested("", 0, 0, []);
                root.closeMenu();
                root.menuExpanded = !root.menuExpanded;
            }
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
                { key: "environment", label: qsTr("Ambiente") },
                { key: "tools", label: qsTr("Ferramentas") },
                { key: "help", label: qsTr("Ajuda") }
            ]

            delegate: Rectangle {
                id: menuButton

                required property var modelData

                visible: root.menuExpanded
                anchors.verticalCenter: parent.verticalCenter
                width: menuLabel.implicitWidth + 2 * Theme.spacingSmall
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

    // Faixa de arrastar a janela. Sem titulo desde a 0.3.8 F3: o nome do
    // projeto aparecia tres vezes (aqui, no widget de projeto e no
    // explorador); o widget de projeto e' o dono.
    Item {
        id: dragRegion

        x: menuRow.x + menuRow.width + Theme.spacingSmall
        width: Math.max(0, windowControls.x - Theme.spacingSmall - x)
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
