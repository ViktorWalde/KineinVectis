pragma ComponentBehavior: Bound
import QtQuick
import "KvLists.js" as KvLists

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
    // O catalogo de comandos do core (command.list): o atalho de cada item sai
    // DAQUI, a mesma fonte da paleta — o menu nunca anuncia atalho que nao
    // existe (2026-10-03; antes uns vinham no texto, "(Ctrl+F6)", e a maioria
    // nao vinha).
    property var commandList: []

    visible: false

    // MENUS COM HIERARQUIA (2026-10-03, pedido do autor: modernizar o que
    // ainda tinha o estilo antigo; os menus eram listas planas numa cor so').
    // Cada item ganha icone e atalho; os grupos, um separador entre eles.
    readonly property var icons: ({
        "workspace.createProject": "add", "workspace.open": "project", "workspace.recent.clear": "trash",
        "workspace.close": "close", "project.createFile": "file", "project.createDirectory": "folder",
        "editor.find": "search", "editor.replace": "search", "fs.replace": "search", "settings.open": "settings",
        "view.project": "project", "view.terminal": "terminal", "view.tools": "tools", "view.areas": "more",
        "view.focusMode": "maximize", "rail.restore": "refresh", "search.everywhere": "search",
        "search.recent": "recent", "search.documentSymbols": "outline", "editor.format": "configure",
        "lsp.codeActions": "problems", "cmake.configure": "configure", "build.run": "build",
        "build.run.cargo": "build", "build.run.cmake": "build", "test.run": "test", "test.run.cargo": "test",
        "test.run.cmake": "test", "test.run.python": "test", "quality.run": "problems", "coverage.run": "test",
        "run.start": "run", "debug.start": "debug", "run.stop": "stop", "debug.stop": "stop",
        "library.list": "documents", "datasource.list": "database", "remote.list": "remote",
        "grafana.get": "observability", "probe.list": "embedded", "container.list": "container",
        "setup.list": "download", "toolchain.get": "tools", "python.context": "configure",
        "configAction.list": "configure", "tools.detect": "refresh", "view.git": "git", "lsp.restart": "refresh",
        "help.manual": "help", "help.about": "help", "app.quit": "close", "editor.save": "save",
        "editor.saveAll": "save", "editor.gotoLine": "goto", "lsp.rename": "rename",
        "view.returnToEditor": "file", "view.cycleFocus": "chevron-down", "view.cycleFocusBack": "chevron-up",
        "lsp.switchSourceHeader": "restore"
    })

    // Item cuja tecla mora num comando de OUTRO nome no catalogo, e as poucas
    // teclas que so' a UI liga (sem comando no core). O gate
    // verificar_atalhos.py confere os dois mapas: o comando existe e declara
    // atalho; a tecla esta' num Shortcut de verdade.
    readonly property var commandOf: ({
        "editor.save": "fs.write", "editor.format": "format.text", "settings.open": "settings.get",
        "view.terminal": "terminal.open", "search.everywhere": "fs.findFiles"
    })
    readonly property var uiShortcuts: ({
        "editor.saveAll": "Ctrl+Shift+S", "editor.gotoLine": "Ctrl+G", "search.recent": "Ctrl+E"
    })

    function shortcutOf(action) {
        if (root.uiShortcuts[action] !== undefined) return root.uiShortcuts[action];
        const id = root.commandOf[action] || action;
        for (const command of KvLists.listOf(root.commandList)) {
            if (command.id === id) return command.defaultShortcut !== undefined ? command.defaultShortcut : "";
        }
        return "";
    }

    function item(label, action, enabled) {
        return { label: label, action: action, enabled: enabled, icon: root.icons[action] || "",
                 shortcut: root.shortcutOf(action) };
    }

    // Os grupos, na ordem, com um separador entre os que nao vieram vazios.
    function grouped(groups) {
        const out = [];
        for (const group of groups) {
            if (group.length === 0) continue;
            if (out.length > 0) out.push({ separator: true, label: "", action: "", enabled: false });
            for (const entry of group) out.push(entry);
        }
        return out;
    }

    // Um projeto pode ter Cargo, CMake, os dois, ou nenhum. Oferecer "Build
    // (CMake)" num projeto so' de Cargo e' oferecer um caminho que termina em
    // erro.
    function hasBuildSystem(buildSystem) {
        return root.workspaceBuildSystems.indexOf(buildSystem) >= 0;
    }

    // Abrir + recentes: o comeco do menu Arquivo e o menu inteiro do widget
    // de projeto da barra (F1 do roadmaps/43). Um dono para a lista.
    function openGroup() {
        return [root.item(qsTr("Criar projeto…"), "workspace.createProject", true),
                root.item(qsTr("Abrir projeto…"), "workspace.open", true)];
    }

    function recentGroup() {
        const items = [];
        for (let index = 0; index < Math.min(root.recentWorkspaces.length, 8); index++) {
            const recent = root.recentWorkspaces[index];
            items.push({ label: recent.name + (recent.available ? "" : qsTr(" — caminho ausente")),
                         action: "workspace.recent.open:" + index, enabled: recent.available,
                         icon: recent.pinned ? "pin" : "recent", shortcut: "" });
        }
        if (items.length > 0) items.push(root.item(qsTr("Limpar projetos recentes"), "workspace.recent.clear", true));
        return items;
    }

    function projectMenuItems() {
        return root.grouped([root.openGroup(), root.recentGroup(),
                             [root.item(qsTr("Fechar projeto"), "workspace.close", root.workspaceOpen)]]);
    }

    // O Build de cada sistema que o projeto TEM (hasBuildSystem, acima).
    function buildGroups() {
        const project = root.workspaceOpen && root.coreConnected;
        const cargo = root.hasBuildSystem("cargo");
        const cmake = root.hasBuildSystem("cmake");
        const python = root.hasBuildSystem("python");
        const cpp = cmake || root.hasBuildSystem("make");
        const compile = cargo && cmake
            ? [root.item(qsTr("Compilar com Cargo"), "build.run.cargo", project),
               root.item(qsTr("Compilar com CMake"), "build.run.cmake", project)]
            : [root.item(qsTr("Compilar"), "build.run", project)];
        const tests = cargo && cmake
            ? [root.item(qsTr("Testar com Cargo"), "test.run.cargo", project),
               root.item(qsTr("Testar com CMake"), "test.run.cmake", project)]
            : [root.item(qsTr("Testes"), "test.run", project)];
        // Python (2026-09-13): num projeto hibrido o "Testes" generico iria
        // para o outro sistema.
        if (python && (cargo || cmake)) tests.push(root.item(qsTr("Testar com pytest"), "test.run.python", project));
        // A analise de cada ecossistema (clippy, clang-tidy, ruff) e a
        // cobertura (cargo-llvm-cov, coverage.py).
        const quality = [
            root.item(cargo ? qsTr("Análise (clippy)") : cpp ? qsTr("Análise (clang-tidy)") : qsTr("Análise (ruff)"),
                      "quality.run", project && (cargo || cpp || python)),
            root.item(qsTr("Cobertura dos testes"), "coverage.run", project && (cargo || python))
        ];
        return [[root.item(qsTr("Configurar CMake"), "cmake.configure", project && cmake)], compile, tests, quality];
    }

    function menuItems(key) {
        const open = root.workspaceOpen;
        const file = root.hasActiveFile;
        const menus = {
            file: [root.openGroup(), root.recentGroup(),
                   [root.item(qsTr("Novo arquivo…"), "project.createFile", open),
                    root.item(qsTr("Nova pasta…"), "project.createDirectory", open)],
                   [root.item(qsTr("Salvar"), "editor.save", file),
                    root.item(qsTr("Salvar tudo"), "editor.saveAll", file)],
                   [root.item(qsTr("Fechar projeto"), "workspace.close", open),
                    root.item(qsTr("Sair"), "app.quit", true)]],
            edit: [[root.item(qsTr("Buscar no arquivo"), "editor.find", file),
                    root.item(qsTr("Substituir no arquivo"), "editor.replace", file),
                    root.item(qsTr("Substituir no projeto"), "fs.replace", open)],
                   [root.item(qsTr("Configurações"), "settings.open", true)]],
            view: [[root.item(qsTr("Explorador do projeto"), "view.project", open),
                    root.item(qsTr("Terminal"), "view.terminal", open),
                    root.item(qsTr("Ferramentas"), "view.tools", open),
                    root.item(qsTr("Áreas da IDE…"), "view.areas", open)],
                   [root.item(qsTr("Modo Foco"), "view.focusMode", open),
                    root.item(qsTr("Voltar ao editor"), "view.returnToEditor", open),
                    root.item(qsTr("Próxima área"), "view.cycleFocus", open),
                    root.item(qsTr("Área anterior"), "view.cycleFocusBack", open)],
                   [root.item(qsTr("Restaurar trilho padrão"), "rail.restore", open)]],
            navigate: [[root.item(qsTr("Buscar em tudo"), "search.everywhere", open),
                        root.item(qsTr("Arquivos recentes"), "search.recent", open)],
                       [root.item(qsTr("Ir para linha"), "editor.gotoLine", file),
                        root.item(qsTr("Símbolos do arquivo"), "search.documentSymbols", file)]],
            code: [[root.item(qsTr("Formatar arquivo"), "editor.format", file),
                    root.item(qsTr("Renomear símbolo"), "lsp.rename", file),
                    root.item(qsTr("Ações de código"), "lsp.codeActions", file)],
                   [root.item(qsTr("Alternar source/header"), "lsp.switchSourceHeader", file)]],
            build: root.buildGroups(),
            run: [[root.item(qsTr("Executar"), "run.start", open && root.coreConnected && !root.running),
                   root.item(qsTr("Depurar"), "debug.start", open && root.coreConnected && !root.debugging)],
                  [root.item(qsTr("Parar execução"), "run.stop", root.running),
                   root.item(qsTr("Parar debug"), "debug.stop", root.debugging)]],
            // "Ambiente" existe porque ate' 2026-09-04 os paineis de
            // configuracao so' eram alcancaveis por atalho ou pela paleta
            // (relato do autor: "quero acessar visualmente pelo mouse").
            // Configurar ambiente e' o que se faz ANTES de compilar. Desde
            // 2026-10-03 tudo aqui e' do projeto (Containers inclusive: a tela
            // de boas-vindas e' so' de boas-vindas), menos instalar e detectar
            // ferramentas e as preferencias.
            environment: [[root.item(qsTr("Banco de dados"), "datasource.list", open),
                           root.item(qsTr("Containers"), "container.list", open),
                           root.item(qsTr("Alvo remoto (SSH)…"), "remote.list", open),
                           root.item(qsTr("Observabilidade…"), "grafana.get", open),
                           root.item(qsTr("Embarcados…"), "probe.list", open)],
                          [root.item(qsTr("Bibliotecas…"), "library.list", open),
                           root.item(qsTr("Toolchain e kits…"), "toolchain.get", open),
                           root.item(qsTr("Python do projeto…"), "python.context", open),
                           root.item(qsTr("Ações de configuração…"), "configAction.list", open),
                           root.item(qsTr("Configurar CMake"), "cmake.configure", open && root.coreConnected)],
                          [root.item(qsTr("Instalar ferramentas…"), "setup.list", true),
                           root.item(qsTr("Detectar ferramentas"), "tools.detect", true),
                           root.item(qsTr("Preferências…"), "settings.open", true)]],
            tools: [[root.item(qsTr("Terminal"), "view.terminal", open),
                     root.item(qsTr("Git"), "view.git", open)],
                    [root.item(qsTr("Detectar ferramentas"), "tools.detect", true),
                     root.item(qsTr("Reiniciar LSP"), "lsp.restart", open)]],
            help: [[root.item(qsTr("Manual da IDE"), "help.manual", true),
                    root.item(qsTr("Sobre Kinein Vectis"), "help.about", true)]]
        };
        return menus[key] === undefined ? [] : root.grouped(menus[key]);
    }

}
