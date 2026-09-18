import QtQuick
import "../../ui/qml/shell"

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
        id: itens
    }

    function acoes(lista) {
        return lista.map(function(i) { return i.action; }).join(",");
    }

    Component.onCompleted: {
        let failures = 0;

        // Sem projeto, sem recentes.
        let projeto = itens.projectMenuItems();
        if (projeto.length !== 2 || projeto[0].action !== "workspace.open" || !projeto[0].enabled) failures += 1;
        if (projeto[1].action !== "workspace.close" || projeto[1].enabled) failures += 2;

        // Recentes: indice na acao; ausente desabilitado; fixado no rotulo.
        itens.recentWorkspaces = [
            { name: "KineinVectis", root: "/home/u/KineinVectis", available: true, pinned: true },
            { name: "placa", root: "/tmp/placa", available: false, pinned: false }
        ];
        itens.workspaceOpen = true;
        projeto = itens.projectMenuItems();
        const recentes = projeto.filter(function(i) { return i.action.indexOf("workspace.recent.open:") === 0; });
        if (recentes.length !== 2 || recentes[0].action !== "workspace.recent.open:0" || !recentes[0].enabled) failures += 4;
        if (recentes[0].label.indexOf("Fixado") < 0 || recentes[1].enabled || recentes[1].label.indexOf("ausente") < 0) failures += 8;
        if (projeto[projeto.length - 1].action !== "workspace.close" || !projeto[projeto.length - 1].enabled) failures += 16;
        if (acoes(projeto).indexOf("editor.save") >= 0 || acoes(projeto).indexOf("app.quit") >= 0) failures += 32;

        // O menu Arquivo COMECA com a mesma lista.
        const arquivo = itens.menuItems("file");
        for (let i = 0; i < projeto.length - 1; i++) {
            if (arquivo[i].action !== projeto[i].action) failures += 64;
        }

        // Build: so' os sistemas presentes.
        itens.coreConnected = true;
        itens.workspaceBuildSystems = ["cargo"];
        let build = acoes(itens.menuItems("build"));
        const configurar = itens.menuItems("build").filter(function(i) { return i.action === "cmake.configure"; })[0];
        if (build.indexOf("build.run.cmake") >= 0 || configurar === undefined || configurar.enabled) failures += 128;
        itens.workspaceBuildSystems = ["cargo", "cmake"];
        build = acoes(itens.menuItems("build"));
        if (build.indexOf("build.run.cargo") < 0 || build.indexOf("test.run.cmake") < 0 || build.indexOf("cmake.configure") < 0) failures += 256;
        if (build.indexOf("coverage.run") < 0 || build.indexOf("quality.run") < 0) failures += 512;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
