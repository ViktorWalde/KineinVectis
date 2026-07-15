pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool hasActiveFile: false
    property bool coreConnected: false
    property bool running: false
    property bool debugging: false
    property string workspaceKind: ""
    property var workspaceBuildSystems: []
    property var recentWorkspaces: []
    property string activeMenu: ""

    signal actionRequested(string action)
    signal menuRequested(string key, real menuX, real menuY, var items)

    height: 40
    color: Theme.background0
    border.color: Theme.borderSoft
    border.width: 1
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
        const fileItems = [
            { label: qsTr("Abrir workspace..."), action: "workspace.open", enabled: true }
        ];
        if (recentWorkspaces.length > 0) {
            fileItems.push({ label: qsTr("Abrir recente"), action: "", enabled: false });
            for (let index = 0; index < Math.min(recentWorkspaces.length, 8); index++) {
                const recent = recentWorkspaces[index];
                const prefix = recent.pinned ? qsTr("Fixado — ") : "";
                const suffix = recent.available ? "" : qsTr(" — caminho ausente");
                fileItems.push({
                    label: "  " + prefix + recent.name + suffix,
                    action: "workspace.recent.open:" + index,
                    enabled: recent.available
                });
            }
            fileItems.push({
                label: qsTr("Limpar workspaces recentes"),
                action: "workspace.recent.clear",
                enabled: true
            });
        }
        fileItems.push(
            { label: qsTr("Novo arquivo..."), action: "project.createFile", enabled: workspaceOpen },
            { label: qsTr("Nova pasta..."), action: "project.createDirectory", enabled: workspaceOpen },
            { label: qsTr("Fechar workspace"), action: "workspace.close", enabled: workspaceOpen },
            { label: qsTr("Salvar"), action: "editor.save", enabled: hasActiveFile },
            { label: qsTr("Salvar tudo"), action: "editor.saveAll", enabled: hasActiveFile },
            { label: qsTr("Sair"), action: "app.quit", enabled: true }
        );
        const cargoAvailable = hasBuildSystem("cargo");
        const cmakeAvailable = hasBuildSystem("cmake");
        const hybrid = cargoAvailable && cmakeAvailable;
        const buildItems = [
            { label: qsTr("Configurar CMake"), action: "cmake.configure",
              enabled: workspaceOpen && cmakeAvailable && coreConnected }
        ];
        if (hybrid) {
            buildItems.push(
                { label: qsTr("Compilar com Cargo"), action: "build.run.cargo", enabled: workspaceOpen && coreConnected },
                { label: qsTr("Testar com Cargo"), action: "test.run.cargo", enabled: workspaceOpen && coreConnected },
                { label: qsTr("Compilar com CMake"), action: "build.run.cmake", enabled: workspaceOpen && coreConnected },
                { label: qsTr("Testar com CMake"), action: "test.run.cmake", enabled: workspaceOpen && coreConnected }
            );
        } else {
            buildItems.push(
                { label: qsTr("Compilar"), action: "build.run", enabled: workspaceOpen && coreConnected },
                { label: qsTr("Testes"), action: "test.run", enabled: workspaceOpen && coreConnected }
            );
        }
        buildItems.push({ label: qsTr("Análise Cargo"), action: "quality.run",
                          enabled: workspaceOpen && cargoAvailable && coreConnected });
        const menus = {
            file: fileItems,
            edit: [
                { label: qsTr("Buscar no arquivo"), action: "editor.find", enabled: hasActiveFile },
                { label: qsTr("Substituir no arquivo"), action: "editor.replace", enabled: hasActiveFile },
                { label: qsTr("Substituir no projeto"), action: "fs.replace", enabled: workspaceOpen },
                { label: qsTr("Configurações"), action: "settings.open", enabled: true }
            ],
            view: [
                { label: qsTr("Explorador do projeto"), action: "view.project", enabled: workspaceOpen },
                { label: qsTr("KV Context"), action: "view.context", enabled: workspaceOpen },
                { label: qsTr("Terminal"), action: "view.terminal", enabled: workspaceOpen },
                { label: qsTr("Ferramentas"), action: "view.tools", enabled: true }
            ],
            navigate: [
                { label: qsTr("Search Everywhere"), action: "search.everywhere", enabled: workspaceOpen },
                { label: qsTr("Arquivos recentes"), action: "search.recent", enabled: workspaceOpen },
                { label: qsTr("Ir para linha"), action: "editor.gotoLine", enabled: hasActiveFile },
                { label: qsTr("Símbolos do arquivo"), action: "search.documentSymbols", enabled: hasActiveFile }
            ],
            code: [
                { label: qsTr("Formatar arquivo"), action: "editor.format", enabled: hasActiveFile },
                { label: qsTr("Renomear símbolo"), action: "lsp.rename", enabled: hasActiveFile },
                { label: qsTr("Ações de código"), action: "lsp.codeActions", enabled: hasActiveFile },
                { label: qsTr("Alternar source/header"), action: "lsp.switchSourceHeader", enabled: hasActiveFile }
            ],
            build: buildItems,
            run: [
                { label: qsTr("Executar"), action: "run.start", enabled: workspaceOpen && coreConnected && !running },
                { label: qsTr("Depurar"), action: "debug.start", enabled: workspaceOpen && coreConnected && !debugging },
                { label: qsTr("Parar execução"), action: "run.stop", enabled: running },
                { label: qsTr("Parar debug"), action: "debug.stop", enabled: debugging }
            ],
            tools: [
                { label: qsTr("Terminal"), action: "view.terminal", enabled: workspaceOpen },
                { label: qsTr("Git"), action: "view.git", enabled: workspaceOpen },
                { label: qsTr("Detectar ferramentas"), action: "tools.detect", enabled: true },
                { label: qsTr("Reiniciar LSP"), action: "lsp.restart", enabled: workspaceOpen }
            ],
            help: [
                { label: qsTr("Manual da IDE"), action: "help.manual", enabled: true },
                { label: qsTr("Sobre Kinein Vectis"), action: "help.about", enabled: true }
            ]
        };
        return menus[key];
    }

    Row {
        id: menuRow

        anchors.left: parent.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.leftMargin: Theme.spacingMedium
        height: parent.height
        spacing: Theme.spacingXSmall

        Image {
            anchors.verticalCenter: parent.verticalCenter
            width: 24
            height: 24
            source: "qrc:/KineinVectis/assets/app-icon.png"
            fillMode: Image.PreserveAspectFit
            smooth: true
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Kinein")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
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

    Text {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingMedium
        anchors.verticalCenter: parent.verticalCenter
        text: root.workspaceOpen ? qsTr("workspace ativo") : qsTr("sem workspace")
        color: Theme.textMuted
        font.pixelSize: 11
    }
}
