import QtQuick

// Regras PURAS da arvore do projeto, num dono so' (Etapa 2, F4 do
// roadmaps/43, 2026-09-18): o que e' da MAQUINA e nao do autor — pastas de
// build, cache, VCS e IDE — e a ordem das entradas. A referencia marca
// essas pastas como "excluded"; aqui vao para o fim das pastas e em cinza.
// Sem estado, sem UI: o ProjectTreeController as usa ao inserir, e a linha
// da arvore le o `machine` que ele escreveu.
QtObject {
    id: root

    readonly property var machineNames: [
        ".git", ".idea", ".kinein", ".vscode", ".cargo", ".ruff_cache", ".mypy_cache",
        ".pytest_cache", ".venv", "__pycache__", "node_modules", "build", "target", "dist"
    ]

    function isMachineEntry(name) {
        return machineNames.indexOf(name) >= 0;
    }

    // Pastas do autor, pastas da maquina, arquivos — cada grupo na ordem em
    // que o core mandou.
    function orderEntries(entries) {
        return entries.filter(function(e) { return e.kind === "directory" && !root.isMachineEntry(e.name); })
            .concat(entries.filter(function(e) { return e.kind === "directory" && root.isMachineEntry(e.name); }))
            .concat(entries.filter(function(e) { return e.kind !== "directory"; }));
    }
}
