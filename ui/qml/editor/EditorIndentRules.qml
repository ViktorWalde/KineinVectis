import QtQuick

// O FALLBACK LOCAL DA INDENTACAO (extraido do EditorTextController, 2026-09-25).
//
// Ele e' o caminho NORMAL, e nao o plano B: roda sempre, na hora da tecla, sem
// esperar ninguem. A §7 da especificacao poe a latencia da tecla como regua, e
// nada sincrono pode entrar nesse caminho.
//
// A correcao por GRAMATICA chega depois, pelo `syntaxTree.indent`, e so'
// corrige quando diverge. Com isso: arquivo sem gramatica (Markdown, `.log`)
// funciona como sempre funcionou; arvore com erro — que e' o estado normal
// enquanto se digita — nao trava a tecla; e core lento ou morto degrada para o
// comportamento de hoje, sem tela de erro.
//
// Mora num objeto proprio por dois motivos. E' REGRA, e nao desenho. E o
// harness QML carrega um espelho plano das fontes, sem os tipos registrados em
// C++ — regra separada do widget e' regra testavel.
QtObject {
    id: root

    // O texto de UM nivel, decidido pela configuracao do editor. A regra nao
    // opina sobre espaco ou tabulacao: ela conta niveis.
    property string unit: "    "

    // O que fazer com a nova linha: o texto a inserir depois do `\n`, e quantos
    // caracteres o cursor anda a partir da quebra.
    //
    // `content` e' o buffer inteiro; `cursor` e' o offset da quebra.
    function forNewline(content, cursor) {
        const lineStart = content.lastIndexOf("\n", Math.max(0, cursor - 1)) + 1;
        const beforeCursor = content.substring(lineStart, cursor);
        const baseIndent = root.leadingWhitespace(beforeCursor);
        const trimmed = beforeCursor.replace(/[ \t]+$/, "");
        const lineContent = beforeCursor.substring(baseIndent.length);

        // Comentario de linha continua comentado: quebrar um `//` e deixar a
        // proxima linha como codigo muda o significado do arquivo.
        if (lineContent.indexOf("//") === 0) {
            return { "insert": baseIndent + "// ", "closerIndent": "" };
        }

        // Dentro de um `/* */` aberto, a continuacao alinha com o asterisco.
        const searchFrom = Math.max(0, cursor - 1);
        const blockStart = content.lastIndexOf("/*", searchFrom);
        const blockEnd = content.lastIndexOf("*/", searchFrom);
        if (blockStart >= 0 && blockStart > blockEnd) {
            const inner = lineContent.replace(/^[ \t]+/, "");
            return {
                "insert": baseIndent + (inner.indexOf("*") === 0 ? "* " : " * "),
                "closerIndent": ""
            };
        }

        // Abriu um bloco e o fechador ja' esta' logo a' frente (o auto-close
        // poe): a linha nova fica no meio, e o fechador desce alinhado.
        if (root.opensBlock(trimmed) && root.closerAhead(content, cursor)) {
            return { "insert": baseIndent + root.unit, "closerIndent": baseIndent };
        }

        return {
            "insert": baseIndent + (root.opensBlock(trimmed) ? root.unit : ""),
            "closerIndent": ""
        };
    }

    // A indentacao da linha, como TEXTO: quem decide espaco ou tabulacao e' a
    // configuracao, e devolver "dois niveis" obrigaria os dois lados a
    // concordar sobre o que e' um nivel.
    function leadingWhitespace(line) {
        const match = /^[ \t]*/.exec(line);
        return match === null ? "" : match[0];
    }

    // O que a heuristica local sabe: a linha termina abrindo algo.
    //
    // O `:` e' de Python, e em C++ ele tambem acerta (`case x:`, `public:`).
    // Onde ela erra — `match` do Rust, continuacao depois de `&&`, `{` dentro
    // de string — quem corrige e' a gramatica.
    function opensBlock(trimmedLine) {
        return trimmedLine.endsWith("{") || trimmedLine.endsWith("(")
                || trimmedLine.endsWith("[") || trimmedLine.endsWith(":");
    }

    function closerAhead(content, cursor) {
        const next = content.charAt(cursor);
        return next === "}" || next === ")" || next === "]";
    }

    // Quantos niveis de `unit` cabem numa indentacao existente. Serve para
    // comparar o que o fallback fez com o que a gramatica respondeu.
    function levelsOf(indentText) {
        if (root.unit === "" || indentText === "") {
            return 0;
        }
        let levels = 0;
        let rest = indentText;
        while (rest.startsWith(root.unit)) {
            levels += 1;
            rest = rest.substring(root.unit.length);
        }
        return levels;
    }

    // O texto de `levels` niveis.
    function indentFor(levels) {
        let text = "";
        for (let i = 0; i < levels; i++) {
            text += root.unit;
        }
        return text;
    }
}
