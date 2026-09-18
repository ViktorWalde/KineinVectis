import QtQuick
import "../../ui/qml/project"

// O explorer segue o arquivo ativo (Etapa 2, F3, 2026-09-18) no
// ProjectTreeController REAL: abre uma pasta por vez ate' o arquivo e o
// seleciona quando ele aparece; fora do workspace nao faz nada; o pedido
// mais novo substitui o anterior.
Item {
    id: root

    property var listagens: []

    ProjectTreeController {
        id: tree

        workspaceRoot: "/w"
        onListDirRequested: function(path) { root.listagens.push(path); }
    }

    Component.onCompleted: {
        let failures = 0;
        tree.setDirectoryListing("/w", [{ name: "src", kind: "directory" }, { name: "docs", kind: "directory" }, { name: "Cargo.toml", kind: "file" }]);

        // Fora do workspace: nada.
        tree.revealPath("/outro/x.rs");
        if (root.listagens.length !== 0 || tree.selectedPath !== "") failures += 1;

        // Arquivo na raiz: seleciona na hora.
        tree.revealPath("/w/Cargo.toml");
        if (tree.selectedPath !== "/w/Cargo.toml" || root.listagens.length !== 0) failures += 2;

        // Dois niveis: pede src, depois src/remote, depois seleciona.
        tree.revealPath("/w/src/remote/mod.rs");
        if (root.listagens.join(",") !== "/w/src") failures += 4;
        tree.setDirectoryListing("/w/src", [{ name: "remote", kind: "directory" }, { name: "lib.rs", kind: "file" }]);
        if (root.listagens.join(",") !== "/w/src,/w/src/remote") failures += 8;
        tree.setDirectoryListing("/w/src/remote", [{ name: "mod.rs", kind: "file" }]);
        if (tree.selectedPath !== "/w/src/remote/mod.rs" || tree.selectedKind !== "file") failures += 16;

        // Ja' aberto: seleciona sem listar de novo.
        const antes = root.listagens.length;
        tree.revealPath("/w/src/lib.rs");
        if (tree.selectedPath !== "/w/src/lib.rs" || root.listagens.length !== antes) failures += 32;

        // A mesma pasta pendente nao e' pedida duas vezes (duas listagens
        // recolheriam o que a primeira abriu).
        const n = root.listagens.length;
        tree.revealPath("/w/docs/a.md");
        tree.revealPath("/w/docs/b.md");
        if (root.listagens.length !== n + 1 || root.listagens[n] !== "/w/docs") failures += 128;
        tree.setDirectoryListing("/w/docs", [{ name: "b.md", kind: "file" }]);
        if (tree.selectedPath !== "/w/docs/b.md") failures += 256;

        // O pedido mais novo substitui: o antigo nao seleciona quando chega.
        tree.revealPath("/w/src/remote/outro/a.rs");
        tree.revealPath("/w/Cargo.toml");
        if (tree.selectedPath !== "/w/Cargo.toml") failures += 64;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
