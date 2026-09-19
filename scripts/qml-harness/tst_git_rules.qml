import QtQuick
import "../../ui/qml/git"

// As regras puras da HUD do Git: raias do grafo (linha reta, merge, ramo
// que fecha), arquivos de um patch com +/-, classe da linha, pasta, refs.
Item {
    id: root

    GitRules { id: rules }

    Component.onCompleted: {
        let failures = 0;

        // Uma linha reta: tudo na raia 0, uma coluna viva.
        const reta = rules.lanes([
            { sha: "c", parents: ["b"] }, { sha: "b", parents: ["a"] }, { sha: "a", parents: [] }
        ]);
        if (reta.length !== 3 || reta.some(r => r.lane !== 0 || r.merge || r.laneCount !== 1)) failures += 1;

        // Um merge: m tem dois pais; o segundo pai abre a raia 1 ate' fechar em a.
        //   m (b, x) -> b (a) -> x (a) -> a
        const merge = rules.lanes([
            { sha: "m", parents: ["b", "x"] },
            { sha: "b", parents: ["a"] },
            { sha: "x", parents: ["a"] },
            { sha: "a", parents: [] }
        ]);
        if (!merge[0].merge || merge[0].lane !== 0 || merge[0].laneCount !== 2) failures += 2;
        if (merge[1].lane !== 0 || merge[2].lane !== 1) failures += 4;
        // depois de x, as duas colunas esperam "a": viram uma.
        if (merge[3].lane !== 0 || merge[3].laneCount !== 1) failures += 8;
        // As arestas (fatia 2c): m liga a b (0->0) e a x (0->1); b segue reto
        // (0->0) enquanto x passa (1->1); x liga a a (1->0); a nao tem saida.
        const arestas = r => r.edges.map(e => e.from + ">" + e.to).join(",");
        if (arestas(merge[0]) !== "0>0,0>1" || arestas(merge[1]) !== "0>0,1>1"
            || arestas(merge[2]) !== "0>0,1>0" || arestas(merge[3]) !== "") failures += 8192;

        // Sem pais informados (core antigo): linha reta, sem quebrar.
        const semPais = rules.lanes([{ sha: "z" }, { sha: "y" }]);
        if (semPais.length !== 2 || semPais[1].lane !== 0) failures += 16;

        // Arquivos de um patch com contagem de + e -.
        const patch = "diff --git a/src/a.rs b/src/a.rs\nindex 1..2 100644\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1,2 +1,2 @@\n-x\n+y\n+z\ndiff --git a/README.md b/README.md\n--- a/README.md\n+++ b/README.md\n@@ -1 +1 @@\n-a\n+b\n";
        const files = rules.patchFiles(patch);
        if (files.length !== 2 || files[0].path !== "src/a.rs" || files[0].added !== 2 || files[0].removed !== 1
            || files[1].path !== "README.md" || files[1].added !== 1) failures += 32;
        if (rules.patchFiles("").length !== 0) failures += 64;

        if (rules.lineKind("+++ b/x") !== "meta" || rules.lineKind("+novo") !== "add"
            || rules.lineKind("-velho") !== "del" || rules.lineKind("@@ -1 +1 @@") !== "hunk"
            || rules.lineKind(" ctx") !== "ctx" || rules.lineKind("index 1..2") !== "meta") failures += 128;

        if (rules.folderOf("ui/qml/git/GitPanel.qml") !== "ui/qml/git" || rules.folderOf("Cargo.toml") !== "(raiz)") failures += 256;
        // A mesma pasta nao aparece duas vezes: modificados e novos da
        // mesma pasta ficam vizinhos; o original nao e' tocado.
        const status = [{ path: "ui/qml/git/A.qml" }, { path: "ui/CMakeLists.txt" }, { path: "ui/qml/git/B.qml" }, { path: "Cargo.toml" }];
        const ordenado = rules.byFolder(status).map(e => e.path).join(",");
        if (ordenado !== "Cargo.toml,ui/CMakeLists.txt,ui/qml/git/A.qml,ui/qml/git/B.qml" || status[0].path !== "ui/qml/git/A.qml") failures += 4096;

        const head = rules.refChip("HEAD -> main");
        const tag = rules.refChip("tag: v1.2");
        const remoto = rules.refChip("origin/main");
        if (head.name !== "main" || !head.head || tag.name !== "v1.2" || !tag.tag || remoto.name !== "origin/main" || remoto.head) failures += 512;
        // Um delegate nasce antes do modelData: a regra nao pode explodir.
        if (rules.refChip(undefined).name !== "" || rules.refChip(undefined).head) failures += 1024;

        // Stage por pasta: o estado da secao a partir do modelo de mudancas.
        const modelo = Qt.createQmlObject("import QtQuick; ListModel {}", root);
        modelo.append({ folder: "src", absPath: "/w/src/a.rs", staged: true });
        modelo.append({ folder: "src", absPath: "/w/src/b.rs", staged: false });
        modelo.append({ folder: "(raiz)", absPath: "/w/README.md", staged: true });
        const src = rules.folderState(modelo, "src");
        const raiz = rules.folderState(modelo, "(raiz)");
        if (src.paths.length !== 2 || src.staged !== 1 || src.all || !raiz.all || raiz.paths[0] !== "/w/README.md"
            || rules.folderState(modelo, "nada").paths.length !== 0 || rules.folderState(modelo, "nada").all) failures += 2048;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
