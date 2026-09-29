// Selecao por path: Ctrl alterna, Shift usa a ordem visivel, Ctrl+A fica na arvore.
import QtQuick
import KineinVectis

Item {
    id: root
    property int failures: 0

    ProjectTreeController {
        id: tree
        workspaceRoot: "/w"
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            failures += 1;
        }
    }

    function equal(paths, expected, message) {
        check(JSON.stringify(paths) === JSON.stringify(expected),
              message + ": " + JSON.stringify(paths));
    }

    Component.onCompleted: {
        tree.setDirectoryListing("/w", [
            {name: "src", kind: "directory"},
            {name: "docs", kind: "directory"},
            {name: "a.rs", kind: "file"},
            {name: "b.rs", kind: "file"}
        ]);
        tree.selectEntry("/w/src", "directory");
        tree.selectEntry("/w/a.rs", "file", Qt.ControlModifier);
        root.equal(tree.selectedPaths, ["/w/src", "/w/a.rs"], "Ctrl nao somou itens");
        root.check(tree.selectedPath === "/w/a.rs", "primario nao acompanhou Ctrl");

        tree.selectEntry("/w/b.rs", "file", Qt.ShiftModifier);
        root.equal(tree.selectedPaths, ["/w/a.rs", "/w/b.rs"],
                   "Shift nao substituiu pelo intervalo desde a ancora");
        tree.selectEntry("/w/src", "directory", Qt.ControlModifier | Qt.ShiftModifier);
        root.equal(tree.selectedPaths, ["/w/a.rs", "/w/b.rs", "/w/src", "/w/docs"],
                   "Ctrl+Shift nao uniu o intervalo");

        tree.selectEntry("/w/src", "directory", Qt.ControlModifier);
        root.equal(tree.selectedPaths, ["/w/a.rs", "/w/b.rs", "/w/docs"],
                   "Ctrl nao removeu item independente");
        root.check(tree.selectedPath === "/w/docs" && tree.selectedKind === "directory",
                   "primario ficou no item desmarcado");
        tree.openCreateDialog("file");
        root.check(!tree.createDialogVisible, "toolbar aceitou destino ambiguo com varios itens");

        tree.selectAllEntries();
        root.equal(tree.selectedPaths, ["/w/src", "/w/docs", "/w/a.rs", "/w/b.rs"],
                   "Ctrl+A nao escolheu so as linhas visiveis");
        tree.setDirectoryListing("/w/src", [{name: "main.rs", kind: "file"}]);
        tree.selectEntry("/w/src/main.rs", "file");
        tree.toggleDirectory("/w/src", 0, true);
        root.equal(tree.selectedPaths, [], "collapse manteve filho invisivel selecionado");

        tree.selectEntry("/w/a.rs", "file");
        tree.setDirectoryListing("/w", [
            {name: "src", kind: "directory"},
            {name: "a.rs", kind: "file"}
        ]);
        root.equal(tree.selectedPaths, ["/w/a.rs"], "refresh perdeu path que ainda existe");
        tree.setDirectoryListing("/w", [
            {name: "src", kind: "directory"},
            {name: "a.rs", kind: "directory"}
        ]);
        root.check(tree.selectedKind === "directory", "refresh manteve tipo antigo do mesmo path");
        tree.setDirectoryListing("/w", [{name: "src", kind: "directory"}]);
        root.equal(tree.selectedPaths, [], "refresh manteve path removido");

        Qt.exit(failures === 0 ? 0 : 1);
    }
}
