import QtQuick
import "../../ui/qml/editor"

// A busca de simbolos da aba direita (E3-2): pede ao indice com o texto, so'
// aceita a resposta que pediu, recorta pela pasta do arquivo ativo, abre no
// arquivo:linha, e esquece tudo com texto vazio.
Item {
    id: root

    property var pedidos: []
    property var aberto: null

    SymbolsController {
        id: simbolos

        onIndexSymbolsRequested: function(q) { root.pedidos.push(q); }
        onOpenRequested: function(path, line, column) { root.aberto = { path: path, line: line, column: column }; }
    }

    Component.onCompleted: {
        let failures = 0;
        simbolos.activeRelativePath = "crates/kinein-core/src/git/parse.rs";
        if (simbolos.activeFolder !== "crates/kinein-core/src/git") failures += 1;

        // Resposta sem pedido: ignorada.
        simbolos.handleIndexSymbols([{ name: "x", path: "a.rs", line: 1 }], 1, "ready");
        if (simbolos.results.length !== 0) failures += 2;

        simbolos.setQuery("parse_");
        if (!simbolos.searching || !simbolos.active) failures += 4;
        simbolos.requestNow();
        if (root.pedidos.length !== 1 || root.pedidos[0] !== "parse_" || !simbolos.waiting) failures += 8;
        simbolos.handleIndexSymbols([
            { name: "parse_log", kind: "function", path: "crates/kinein-core/src/git/parse.rs", line: 305 },
            { name: "parse_status", kind: "function", path: "crates/kinein-core/src/git/operations.rs", line: 10 },
            { name: "parse_args", kind: "function", path: "crates/kinein-cli/src/main.rs", line: 40 }
        ], 3, "ready");
        if (simbolos.searching || simbolos.waiting || simbolos.results.length !== 3 || simbolos.total !== 3) failures += 16;
        // A pasta do arquivo ativo: os dois de git/, nao o da CLI.
        if (simbolos.folderResults.length !== 2 || simbolos.folderResults[1].name !== "parse_status") failures += 32;

        simbolos.open(simbolos.results[2]);
        if (!root.aberto || root.aberto.path !== "crates/kinein-cli/src/main.rs" || root.aberto.line !== 40) failures += 64;

        simbolos.setQuery("");
        if (simbolos.active || simbolos.results.length !== 0 || simbolos.folderResults.length !== 0) failures += 128;
        simbolos.activeRelativePath = "Cargo.toml";
        if (simbolos.activeFolder !== "") failures += 256;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
