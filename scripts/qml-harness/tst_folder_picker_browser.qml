import QtQuick
import KineinVectis

// O navegador de pastas amplo (0.3.9, retorno do autor: "encolhido"). Prova
// o que a tela promete: historico voltar/avancar so' com listagem que chegou,
// ocultas fora ate' pedir, migalhas do caminho, teclado na lista, a pasta de
// projeto marcada pelo core, "nova pasta" sem derrubar o projeto em criacao,
// "Usar esta pasta" como local — e o cartao LARGO numa tela 1366x768.
Item {
    id: root

    width: 1366
    height: 768

    property var browsed: []
    property var foldersCreated: []

    FolderPickerDialog {
        id: dialog

        anchors.fill: parent
        homePath: "/home/x"
        onBrowseRequested: function(path) { root.browsed.push(path); }
        onCreateFolderRequested: function(parent, name) {
            root.foldersCreated.push(parent + "|" + name);
        }
    }

    function check(condition, label) {
        if (!condition) console.error("FALHOU: " + label);
        return condition ? 0 : 1;
    }

    function find(item, typeName) {
        if (String(item).startsWith(typeName)) return item;
        for (let i = 0; i < item.children.length; i++) {
            const found = find(item.children[i], typeName);
            if (found !== null) return found;
        }
        return null;
    }

    readonly property var places: [
        { id: "home", path: "/home/x" },
        { id: "documents", path: "/home/x/Documentos" },
        { id: "root", path: "/" }
    ]

    function listing(path, names) {
        const entries = names.map(name => ({
            name: name.split(":")[0],
            path: path + "/" + name.split(":")[0],
            kind: name.includes(":") ? name.split(":")[1] : undefined
        }));
        const parent = path.substring(0, path.lastIndexOf("/")) || "/";
        dialog.setListing(path, parent, entries, root.places);
    }

    Component.onCompleted: {
        let failures = 0;
        dialog.open("/home/x");
        const picker = find(dialog, "FolderPickerController");
        failures += check(picker !== null, "controller achado");

        // Ocultas fora ate' pedir; a contagem diz quantas.
        listing("/home/x", [".cache", ".config", "app:rustCargo", "notas"]);
        failures += check(picker.entriesModel.count === 2, "so' as visiveis");
        failures += check(picker.hiddenCount === 2, "duas ocultas contadas");
        failures += check(picker.entriesModel.get(0).kind === "rustCargo", "projeto marcado");
        failures += check(picker.entriesModel.get(1).kind === "", "pasta comum sem marca");
        picker.setShowHidden(true);
        failures += check(picker.entriesModel.count === 4, "ocultas a mostra");
        picker.setShowHidden(false);
        failures += check(picker.places.length === 3, "locais do core");

        // Migalhas: "/" e cada pasta, clicaveis.
        failures += check(picker.crumbs.length === 3 && picker.crumbs[2].path === "/home/x",
                          "migalhas " + JSON.stringify(picker.crumbs));

        // Teclado: desce, entra; a primeira abertura nao empilha nada.
        failures += check(picker.backStack.length === 0, "abrir nao empilha");
        picker.moveSelection(1);
        failures += check(picker.selectedPath === "/home/x/app", "seta seleciona a primeira");
        picker.moveSelection(1);
        picker.enterSelected();
        failures += check(root.browsed[root.browsed.length - 1] === "/home/x/notas", "Enter entra");
        listing("/home/x/notas", []);
        failures += check(picker.backStack.join() === "/home/x", "historico empilhou");

        // Voltar e avancar mexem nas pilhas so' quando a listagem chega.
        picker.goBack();
        failures += check(picker.backStack.length === 1, "pedido sozinho nao mexe");
        listing("/home/x", ["app:rustCargo", "notas"]);
        failures += check(picker.backStack.length === 0
                          && picker.forwardStack.join() === "/home/x/notas", "voltou");
        picker.goForward();
        picker.showError("sem permissao");
        failures += check(picker.forwardStack.length === 1, "falha nao consome o avancar");
        picker.goForward();
        listing("/home/x/notas", []);
        failures += check(picker.forwardStack.length === 0
                          && picker.backStack.join() === "/home/x", "avancou");

        // Criando projeto, "nova pasta" nao derruba o projeto em montagem.
        dialog.openCreateProject("/home/x", "python");
        listing("/home/x", ["app"]);
        picker.createName = "demo";
        picker.createBrowsing = true;
        picker.beginCreateFolder();
        failures += check(picker.creatingProject && picker.creatingFolder, "os dois juntos");
        picker.folderName = "codigo";
        picker.submitCreateFolder();
        failures += check(root.foldersCreated.join() === "/home/x|codigo", "pasta pedida");
        dialog.selectAfterRefresh("/home/x/codigo");
        failures += check(!picker.creatingFolder && picker.createName === "demo",
                          "linha fechou, projeto intacto");

        // O cartao e' LARGO no navegador e o "Usar esta pasta" recolhe.
        const card = find(dialog, "FolderPickerCard");
        failures += check(card.width >= 960, "cartao largo: " + card.width);
        const placesColumn = find(card, "FolderPickerPlaces");
        failures += check(placesColumn !== null && placesColumn.visible, "coluna de locais");
        picker.selectedPath = "/home/x/app";
        picker.useAsLocation();
        failures += check(!picker.createBrowsing, "navegador recolheu");
        failures += check(root.browsed[root.browsed.length - 1] === "/home/x/app", "local escolhido");

        if (failures !== 0) console.error("FALHAS " + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
