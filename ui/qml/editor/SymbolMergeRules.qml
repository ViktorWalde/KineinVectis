import QtQuick

// MESCLAR AS DUAS FONTES DE SIMBOLO: o indice e o LSP (L1, 2026-09-26).
//
// A §11.1 do roadmap 48 pede duas coisas: **deduplicar por arquivo/posicao** e
// **manter a fonte visivel** quando isso ajudar a explicar divergencia.
//
// Por que as duas fontes existem, e por que nenhuma substitui a outra:
//
//   INDICE  responde desde o primeiro segundo e SEM arquivo aberto, no projeto
//           inteiro. Sabe nome e posicao.
//   LSP     responde depois, so' para o arquivo aberto, e sabe mais: tipo,
//           escopo, container.
//
// Dai a regra do empate: na MESMA posicao, o LSP vence, porque ele diz mais
// sobre o mesmo simbolo. E dai tambem por que o indice aparece sozinho: sem ele,
// a aba ficaria vazia ate' o servidor subir — o pilar 0 do roadmap 42.
//
// Regra pura: entram duas listas, sai uma. Nenhum acesso a disco, nenhuma
// decisao de tela.
QtObject {
    id: root

    // A CHAVE de deduplicacao. Nao e' o nome: dois simbolos podem ter o mesmo
    // nome em arquivos diferentes, e o mesmo simbolo aparece nas duas fontes.
    //
    // A coluna entra normalizada porque as fontes contam diferente — o indice
    // conta BYTES e o LSP conta unidades UTF-16. Sem isso, uma linha com acento
    // antes do simbolo vira duas entradas.
    function keyFor(symbol) {
        if (symbol === null || symbol === undefined) {
            return "";
        }
        const path = symbol.path === undefined ? "" : String(symbol.path);
        const line = symbol.line === undefined ? 0 : Number(symbol.line);
        return path + ":" + line;
    }

    // Marca a origem em cada item, para a tela poder dizer de onde veio.
    function tagged(symbols, source) {
        const out = [];
        const lista = symbols === undefined || symbols === null ? [] : symbols;
        for (let i = 0; i < lista.length; i++) {
            const original = lista[i];
            const copia = ({ "source": source });
            for (const campo in original) {
                copia[campo] = original[campo];
            }
            copia.source = source;
            out.push(copia);
        }
        return out;
    }

    // A ordem de saida e' a do INDICE primeiro, porque e' a que o autor ja'
    // estava vendo quando o LSP respondeu: reordenar a lista sob o cursor dele
    // seria pior que mostrar menos.
    function merge(indexSymbols, lspSymbols) {
        const doLsp = root.tagged(lspSymbols, "lsp");
        const porChave = ({});
        for (let i = 0; i < doLsp.length; i++) {
            porChave[root.keyFor(doLsp[i])] = true;
        }
        const saida = [];
        const doIndice = root.tagged(indexSymbols, "índice");
        for (let i = 0; i < doIndice.length; i++) {
            // Mesma posicao: o LSP vence, e o do indice nao entra.
            if (porChave[root.keyFor(doIndice[i])] !== true) {
                saida.push(doIndice[i]);
            }
        }
        for (let i = 0; i < doLsp.length; i++) {
            saida.push(doLsp[i]);
        }
        return saida;
    }

    // `true` quando vale mostrar a fonte na tela: se todas vieram do mesmo
    // lugar, dizer de onde nao explica nada e so' ocupa espaco.
    function sourceWorthShowing(merged) {
        const lista = merged === undefined || merged === null ? [] : merged;
        let temIndice = false;
        let temLsp = false;
        for (let i = 0; i < lista.length; i++) {
            if (lista[i].source === "lsp") {
                temLsp = true;
            } else {
                temIndice = true;
            }
        }
        return temIndice && temLsp;
    }
}
