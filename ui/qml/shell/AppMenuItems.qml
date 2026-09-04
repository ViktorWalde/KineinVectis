pragma ComponentBehavior: Bound
import QtQuick

// O QUE CADA MENU OFERECE, dado o estado da IDE.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). O `AppMenuBar` passou de 300 linhas
// ao ganhar o menu "Ambiente", e a catraca disparou. Os tres suspeitos da §4
// regra 9, na ordem: a mudanca (sete itens de menu legitimos — o gatilho), a
// categoria (a barra DESENHA de verdade: hover, clique, popup) e o arquivo.
//
// Foi o arquivo, e o vocabulario misturado estava a vista: a barra fazia duas
// coisas de natureza oposta — DECIDIR o que cada menu contem, com regra de
// habilitacao ("Executar" so' com projeto aberto, "Depurar" so' se nao estiver
// depurando, os alvos de build so' dos sistemas que o projeto TEM), e DESENHAR
// a barra. A primeira nao tem um pixel; a segunda nao tem uma regra.
//
// Este componente nao desenha nada. Recebe o estado e devolve a lista, e por
// isso da' para exercita-lo no harness sem GUI nenhuma.
Item {
    id: root

    property bool workspaceOpen: false
    property bool hasActiveFile: false
    property bool coreConnected: false
    property bool running: false
    property bool debugging: false
    property var workspaceBuildSystems: []
    property var recentWorkspaces: []

    visible: false

    // Um projeto pode ter Cargo, CMake, os dois, ou nenhum. Oferecer "Build
    // (CMake)" num projeto so' de Cargo e' oferecer um caminho que termina em
    // erro.
    function hasBuildSystem(buildSystem) {
        return workspaceBuildSystems.indexOf(buildSystem) >= 0;
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
            // "Ambiente" existe porque ate' 2026-09-04 os quatro paineis de
            // configuracao so' eram alcancaveis por atalho ou pela paleta.
            // Relato de uso do autor: "quero acessar visualmente pelo mouse".
            // Configurar ambiente NAO e' "ferramenta": e' o que se faz ANTES
            // de compilar, e por isso ganhou menu proprio em vez de virar o
            // quinto item de Ferramentas.
            environment: [
                { label: qsTr("Bibliotecas..."), action: "library.list",
                  enabled: workspaceOpen },
                { label: qsTr("Banco de dados..."), action: "datasource.list",
                  enabled: workspaceOpen },
                { label: qsTr("Observabilidade..."), action: "grafana.get",
                  enabled: workspaceOpen },
                { label: qsTr("Instalar ferramentas..."), action: "setup.list",
                  enabled: true },
                { label: qsTr("Toolchain e kits..."), action: "toolchain.get",
                  enabled: workspaceOpen },
                { label: qsTr("Ações de configuração..."), action: "configAction.list",
                  enabled: workspaceOpen },
                { label: qsTr("Configurar CMake"), action: "cmake.configure",
                  enabled: workspaceOpen && coreConnected },
                { label: qsTr("Detectar ferramentas"), action: "tools.detect",
                  enabled: true },
                { label: qsTr("Preferências..."), action: "settings.open", enabled: true }
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
