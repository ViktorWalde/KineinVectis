import QtQuick

// As REGRAS PURAS dos problemas (Etapa 2, F5/F6-b do roadmaps/43, 2026-09-18):
// o proximo passo de cada um, e o que e' repeticao.
//
// O PROXIMO PASSO:
// a referencia nao lista erros — diz o que fazer com cada um. Regra pura,
// sem UI: dado o problema (fonte, mensagem), o rotulo do botao e para onde
// ele leva. Tres classes, medidas nos textos que o core/ferramentas emitem:
//
//   lsp            o servidor tem code actions -> "Acoes" (o Alt+Enter)
//   sem CDB        clang-tidy/clangd sem compile_commands -> Configurar CMake
//   ferramenta     "nao encontrado", "instale", "not found" -> Ferramentas
//
// O resto nao ganha botao: inventar um passo e' pior que nao ter.
QtObject {
    id: root

    readonly property var sourcesWithCodeActions: ["lsp"]
    readonly property var cdbPatterns: [/compile_commands/i, /\bCDB\b/, /configur/i]
    readonly property var toolPatterns: [/n[aã]o (foi )?encontrad/i, /not found/i, /\binstale\b/i, /ausente/i, /no such file/i]

    function matchesAny(text, patterns) {
        for (let i = 0; i < patterns.length; i++) {
            if (patterns[i].test(text)) {
                return true;
            }
        }
        return false;
    }

    // { label, kind: "codeActions" | "health" | "", target }
    function stepFor(problem) {
        const source = problem.source || "";
        const message = problem.message || "";
        // As fontes com code actions sao os servidores de linguagem.
        if (sourcesWithCodeActions.indexOf(source) >= 0) {
            return { label: qsTr("Ações"), kind: "codeActions", target: "" };
        }
        if (matchesAny(message, cdbPatterns)) {
            return { label: qsTr("Configurar CMake"), kind: "health", target: "cmakeConfigure" };
        }
        if (matchesAny(message, toolPatterns)) {
            return { label: qsTr("Ferramentas"), kind: "health", target: "tools" };
        }
        return { label: "", kind: "", target: "" };
    }

    // REPETICAO (F6-b): o mesmo erro chega pelo build (cargo) E pelo
    // servidor de linguagem (rust-analyzer) — mesmo arquivo, mesma linha, e
    // uma mensagem que comeca com a outra. A lista mostra um so'.
    function isDuplicate(model, diagnostic) {
        const file = diagnostic.file !== undefined ? diagnostic.file : "";
        const line = diagnostic.line !== undefined ? Number(diagnostic.line) : 0;
        const message = (diagnostic.message !== undefined ? diagnostic.message : "").trim();
        if (file === "" || message === "") {
            return false;
        }
        for (let i = 0; i < model.count; i++) {
            const row = model.get(i);
            if (row.file !== file || Number(row.line) !== line) {
                continue;
            }
            const existing = String(row.message).trim();
            if (existing.indexOf(message) === 0 || message.indexOf(existing) === 0) {
                return true;
            }
        }
        return false;
    }
}
