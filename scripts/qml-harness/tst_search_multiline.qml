// A sintaxe `\n` do painel de busca (roadmap 30, etapa 9 — 2026-09-02).
//
// Por que existe: o core passou a achar e substituir texto que atravessa
// linhas, mas o campo do painel e' um `TextInput` de UMA linha — nao da' para
// digitar nem colar uma quebra. A traducao de `\n` mora no controller, e ela
// e' invisivel: se `expandLineBreaks` virar identidade, o build passa, o
// qmllint passa, e a busca simplesmente nunca acha nada com quebra de linha.
// Falha silenciosa (`ARCHITECTURE.md` §4 regra 11).
//
// A segunda metade do teste e' a que costuma ser esquecida: `\\n` tem que
// devolver o LITERAL barra-ene. Quem mexe em C ou Rust procura por `\n` dentro
// do codigo o tempo todo, e uma traducao que come a barra tornaria isso
// impossivel — o campo perderia uma capacidade para ganhar outra.
import QtQuick
import "../../ui/qml/search"

Item {
    id: root

    property var buscas: []
    property var substituicoes: []

    SearchController {
        id: busca

        workspaceRoot: "/tmp/ws"
        onSearchInFilesRequested: function (query, caseSensitive) {
            root.buscas.push(query);
        }
        onReplaceInFilesRequested: function (query, replacement, caseSensitive) {
            root.substituicoes.push({ query: query, replacement: replacement });
        }
    }

    Component.onCompleted: {
        let failures = 0;

        // Nas strings abaixo, `\\n` no fonte QML e' o que o usuario DIGITA:
        // dois caracteres, barra e ene.

        // ---- A traducao, isolada -------------------------------------------
        if (busca.expandLineBreaks("a\\nb") !== "a\nb") failures += 1;
        // Sem escape nenhum, o texto atravessa intocado.
        if (busca.expandLineBreaks("int main()") !== "int main()") failures += 2;
        // `\\n` digitado (tres caracteres: barra, barra, ene) devolve o
        // LITERAL barra-ene.
        if (busca.expandLineBreaks("\\\\n") !== "\\n") failures += 4;
        // Uma passada so': a barra que acabou de ser resolvida nao pode ser
        // reinterpretada junto com o `n` seguinte.
        if (busca.expandLineBreaks("puts(\"\\\\n\");") !== "puts(\"\\n\");") failures += 8;
        // Barra solta no fim nao some nem estoura o indice.
        if (busca.expandLineBreaks("fim\\") !== "fim\\") failures += 16;
        // Escape desconhecido fica como esta': `\t` nao e' sintaxe da caixa.
        if (busca.expandLineBreaks("a\\tb") !== "a\\tb") failures += 32;
        // Duas quebras na mesma consulta.
        if (busca.expandLineBreaks("a\\nb\\nc") !== "a\nb\nc") failures += 64;

        // ---- O caminho ate' o core -----------------------------------------
        busca.runSearch("void f()\\n{");
        if (root.buscas.length !== 1) failures += 128;
        else if (root.buscas[0] !== "void f()\n{") failures += 256;

        // A substituicao traduz OS DOIS lados: quem procura por duas linhas
        // costuma querer devolver duas linhas.
        busca.runReplace("a\\nb", "x\\ny");
        if (root.substituicoes.length !== 1) {
            failures += 512;
        } else {
            if (root.substituicoes[0].query !== "a\nb") failures += 1024;
            if (root.substituicoes[0].replacement !== "x\ny") failures += 2048;
        }

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
