// As respostas do core continuam chegando a arvore e abas apos a extracao.
import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0
    property var listed: []
    property var read: []
    property var renamed: []
    property var closed: []
    property int focusTreeCount: 0

    ProjectTreeController {
        id: tree
        workspaceRoot: "/w"
        onListDirRequested: path => root.listed.push(path)
        onReadFileRequested: path => root.read.push(path)
        onTabsRenameRequested: (from, to) => root.renamed.push([from, to])
        onTabsCloseRequested: path => root.closed.push(path)
        onFocusTreeRequested: root.focusTreeCount += 1
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            failures += 1;
        }
    }

    Component.onCompleted: {
        tree.setDirectoryListing("/w", [
            {name: "src", kind: "directory"},
            {name: "old.rs", kind: "file"}
        ]);
        tree.handleFileCreated("/w/new.rs");
        root.check(tree.selectedPath === "/w/new.rs" && tree.selectedPaths.length === 1,
                   "criar arquivo perdeu selecao");
        root.check(root.read[0] === "/w/new.rs" && root.listed[0] === "/w",
                   "criar arquivo perdeu abertura/refresh");
        tree.handleDirectoryCreated("/w/new-folder");
        root.check(tree.selectedPath === "/w/new-folder" && focusTreeCount === 1,
                   "criar pasta nao devolveu foco a arvore");

        tree.entryRenameKind = "file";
        tree.handlePathRenamed("/w/old.rs", "/w/new-name.rs");
        root.check(root.renamed.length === 1 && root.renamed[0][1] === "/w/new-name.rs",
                   "renomear nao atualizou abas");
        root.check(tree.selectedPath === "/w/new-name.rs" && tree.selectedKind === "file",
                   "renomear nao atualizou selecao");
        root.check(focusTreeCount === 2, "renomear nao devolveu foco a arvore");

        tree.handlePathDeleted("/w/new-name.rs");
        root.check(root.closed[0] === "/w/new-name.rs" && tree.selectedPaths.length === 0,
                   "excluir nao limpou selecao/abas");
        root.check(focusTreeCount === 3, "excluir nao devolveu foco a arvore");

        tree.handleRequestFailed("fs.delete", "permissao negada");
        root.check(tree.entryDeleteVisible && tree.entryDeleteError === "permissao negada",
                   "erro de exclusao nao voltou ao dialogo");

        const before = root.listed.length;
        tree.handleExternalChanges([{path: "/w/novo.rs"}]);
        root.check(root.listed.length === before + 1 && root.listed[before] === "/w",
                   "watcher nao refrescou a raiz");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
