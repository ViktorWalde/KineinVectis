import QtQuick

// Modelo dos menus da App Bar: SO os dados (rotulos, acoes, habilitacao).
// Extraido do AppMenuBar quando a barra unica (F2) o levou acima do limite:
// o suspeito era a mistura barra visual + modelo, nao o arquivo (§4 regra 9).
// Quem executa as acoes continua sendo o ShellHeaderHost; aqui nao ha efeito.
Item {
    id: root

    property bool workspaceOpen: false
    property bool hasActiveFile: false
    property bool coreConnected: false
    property bool running: false
    property bool debugging: false
    property var recentWorkspaces: []
    property var workspaceBuildSystems: []

    visible: false

    function hasBuildSystem(buildSystem) {
        const systems = workspaceBuildSystems !== undefined
                && workspaceBuildSystems !== null ? workspaceBuildSystems : [];
        return systems.indexOf(buildSystem) >= 0;
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
        buildItems.push({ label: qsTr("Análise estática"), action: "quality.run",
                          enabled: workspaceOpen && coreConnected
                                   && (cargoAvailable || cmakeAvailable) });
        // Auditoria le o Cargo.lock: so faz sentido em projeto Cargo.
        buildItems.push({ label: qsTr("Auditoria de segurança"), action: "audit.run",
                          enabled: workspaceOpen && coreConnected && cargoAvailable });
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
                { label: qsTr("Terminal"), action: "view.terminal", enabled: workspaceOpen },
                { label: qsTr("Jobs"), action: "view.jobs", enabled: workspaceOpen },
                { label: qsTr("Log da IDE"), action: "view.logs", enabled: true },
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
}
