import QtQuick
import KineinVectis

// O que cada menu oferece (AppMenuItems, o dono das listas) — e, desde a
// F1 da Etapa 2 (2026-09-18), o menu do widget de PROJETO da barra: abrir +
// recentes + fechar, sem os itens de arquivo/salvar do menu Arquivo.
//
// O que se prova: sem projeto, "Fechar workspace" vem desabilitado; os
// recentes entram com o indice na acao e "caminho ausente" desabilitado; o
// menu Build so' oferece os sistemas que o projeto TEM; o menu de projeto e'
// o comeco do menu Arquivo (um dono para a lista).
Item {
    id: root

    AppMenuItems {
        id: menus
    }

    // Os separadores entre grupos nao sao itens: os testes de conteudo olham
    // so' os itens.
    function withoutSeparators(list) {
        return list.filter(function(i) { return i.separator !== true; });
    }

    function actionsOf(list) {
        return list.map(function(i) { return i.action; }).join(",");
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem projeto, sem recentes.
        let projectItems = withoutSeparators(menus.projectMenuItems());
        // "Criar projeto..." abre a lista (53 §13.0 item 6), antes do Abrir.
        if (projectItems.length !== 3 || projectItems[0].action !== "workspace.createProject" || !projectItems[0].enabled
            || projectItems[1].action !== "workspace.open" || !projectItems[1].enabled) failures += 1;
        if (projectItems[2].action !== "workspace.close" || projectItems[2].enabled) failures += 2;

        // Recentes: indice na acao; ausente desabilitado; fixado no rotulo.
        menus.recentWorkspaces = [
            { name: "KineinVectis", root: "/home/u/KineinVectis", available: true, pinned: true },
            { name: "placa", root: "/tmp/placa", available: false, pinned: false }
        ];
        menus.workspaceOpen = true;
        projectItems = withoutSeparators(menus.projectMenuItems());
        const recents = projectItems.filter(function(i) { return i.action.indexOf("workspace.recent.open:") === 0; });
        if (recents.length !== 2 || recents[0].action !== "workspace.recent.open:0" || !recents[0].enabled) failures += 4;
        if (recents[0].icon !== "pin" || recents[1].enabled || recents[1].label.indexOf("ausente") < 0) failures += 8;
        if (projectItems[projectItems.length - 1].action !== "workspace.close" || !projectItems[projectItems.length - 1].enabled) failures += 16;
        if (actionsOf(projectItems).indexOf("editor.save") >= 0 || actionsOf(projectItems).indexOf("app.quit") >= 0) failures += 32;

        // O menu Arquivo COMECA com a mesma lista.
        const fileMenu = withoutSeparators(menus.menuItems("file"));
        for (let i = 0; i < projectItems.length - 1; i++) {
            if (fileMenu[i].action !== projectItems[i].action) failures += 64;
        }

        // Build: so' os sistemas presentes.
        menus.coreConnected = true;
        menus.workspaceBuildSystems = ["cargo"];
        let build = actionsOf(menus.menuItems("build"));
        const configure = menus.menuItems("build").filter(function(i) { return i.action === "cmake.configure"; })[0];
        if (build.indexOf("build.run.cmake") >= 0 || configure === undefined || configure.enabled) failures += 128;
        menus.workspaceBuildSystems = ["cargo", "cmake"];
        build = actionsOf(menus.menuItems("build"));
        if (build.indexOf("build.run.cargo") < 0 || build.indexOf("test.run.cmake") < 0 || build.indexOf("cmake.configure") < 0) failures += 256;
        if (build.indexOf("coverage.run") < 0 || build.indexOf("quality.run") < 0) failures += 512;

        // Hierarquia (2026-10-03): separador entre grupos, nunca no comeco,
        // no fim ou dobrado; o separador nao e' selecionavel.
        const viewMenu = menus.menuItems("view");
        let separatorCount = 0;
        for (let i = 0; i < viewMenu.length; i++) {
            if (viewMenu[i].separator !== true) continue;
            separatorCount += 1;
            if (i === 0 || i === viewMenu.length - 1 || viewMenu[i - 1].separator === true || viewMenu[i].enabled) failures += 1024;
        }
        if (separatorCount !== 2) failures += 2048;

        // O atalho sai do catalogo (pelo nome do comando, ou pelo mapa
        // commandOf), ou do mapa das teclas so' da UI; sem catalogo, nada.
        if (menus.menuItems("file")[0].shortcut !== "" || withoutSeparators(menus.menuItems("file"))[1].icon !== "project") failures += 4096;
        menus.commandList = [{ id: "workspace.open", defaultShortcut: "Ctrl+O" },
                             { id: "fs.write", defaultShortcut: "Ctrl+S" },
                             { id: "workspace.close" }];
        const withShortcuts = withoutSeparators(menus.menuItems("file"));
        const shortcutFor = function(action) { return withShortcuts.filter(function(i) { return i.action === action; })[0].shortcut; };
        if (shortcutFor("workspace.open") !== "Ctrl+O" || shortcutFor("editor.save") !== "Ctrl+S"
            || shortcutFor("editor.saveAll") !== "Ctrl+Shift+S" || shortcutFor("workspace.close") !== "") failures += 8192;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
