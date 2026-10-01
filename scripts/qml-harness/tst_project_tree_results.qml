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
    property var deleteCalls: []
    property var trashCalls: []
    property bool dirtyUnderDelete: false

    QtObject {
        id: fakeCore
        function listDir(path) {}
        function readFile(path) {}
        function deletePath(path) { root.deleteCalls.push(path); }
        function trashPath(path) { root.trashCalls.push(path); }
    }

    QtObject {
        id: fakeEditor
        function hasUnsavedUnderPath(path) { return root.dirtyUnderDelete; }
    }

    ProjectTreeController {
        id: tree
        workspaceRoot: "/w"
        onListDirRequested: path => root.listed.push(path)
        onReadFileRequested: path => root.read.push(path)
        onTabsRenameRequested: (from, to) => root.renamed.push([from, to])
        onTabsCloseRequested: path => root.closed.push(path)
        onFocusTreeRequested: root.focusTreeCount += 1
    }

    ProjectTreeRequestRouter {
        coreClient: fakeCore
        projectTree: tree
        documentController: fakeEditor
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

        tree.entryDeletePath = "/w/new-name.rs";
        tree.confirmEntryTrash();
        tree.confirmEntryTrash();
        root.check(root.trashCalls.length === 1 && tree.entryDeletePending,
                   "clique repetido enviou remocao duplicada");
        tree.handlePathDeleted("/w/outro.rs");
        root.check(root.closed.length === 0 && tree.entryDeletePending,
                   "resposta de outra remocao fechou abas");
        tree.handlePathDeleted("/w/new-name.rs");
        root.check(root.closed[0] === "/w/new-name.rs" && tree.selectedPaths.length === 0,
                   "excluir nao limpou selecao/abas");
        root.check(focusTreeCount === 3 && !tree.entryDeletePending,
                   "excluir nao devolveu foco ou liberou estado pendente");

        tree.entryDeletePath = "/w/src";
        tree.confirmEntryDelete();
        tree.handleRequestFailed("fs.delete", "permissao negada");
        root.check(tree.entryDeleteVisible && tree.entryDeleteError === "permissao negada",
                   "erro de exclusao nao voltou ao dialogo");
        tree.confirmEntryTrash();
        tree.handleRequestFailed("fs.trash", "lixeira indisponível");
        root.check(tree.entryDeleteVisible && !tree.entryDeletePending
                   && tree.entryDeleteError === "lixeira indisponível",
                   "falha da lixeira nao voltou ao dialogo");

        root.dirtyUnderDelete = true;
        tree.confirmEntryDelete();
        tree.confirmEntryTrash();
        root.check(root.deleteCalls.length === 1 && tree.entryDeleteVisible
                   && root.trashCalls.length === 2
                   && tree.entryDeleteError.indexOf("não salvas") >= 0,
                   "exclusao de pasta com buffer sujo chegou ao core");
        root.dirtyUnderDelete = false;
        tree.confirmEntryDelete();
        root.check(root.deleteCalls.length === 2 && root.deleteCalls[1] === "/w/src",
                   "exclusao limpa nao chegou ao core");
        tree.confirmEntryTrash();
        root.check(root.trashCalls.length === 2 && tree.entryDeletePending,
                   "operacao pendente aceitou segunda remocao");
        tree.handleRequestFailed("fs.trash", "resposta antiga");
        root.check(tree.entryDeletePending && tree.entryDeleteError === "",
                   "falha de outra remocao alterou operacao pendente");
        tree.handleRequestFailed("fs.delete", "teste");
        root.check(!tree.entryDeletePending, "falha nao liberou estado pendente");

        const before = root.listed.length;
        tree.handleExternalChanges([{path: "/w/novo.rs"}]);
        root.check(root.listed.length === before + 1 && root.listed[before] === "/w",
                   "watcher nao refrescou a raiz");

        tree.confirmEntryTrash();
        tree.clear();
        tree.workspaceRoot = "/outro";
        const closedBefore = root.closed.length;
        tree.handlePathDeleted("/w/src");
        root.check(root.closed.length === closedBefore && !tree.entryDeletePending,
                   "resposta tardia da pasta anterior fechou abas novas");
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
