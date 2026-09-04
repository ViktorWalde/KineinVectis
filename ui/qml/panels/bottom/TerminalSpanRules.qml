import QtQuick

// Regras PURAS sobre um trecho ("span") de linha do terminal.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). `spanCells` estava escrito DUAS
// vezes — no TerminalViewport e no TerminalSelectionController — e as duas
// copias ja divergiam no detalhe: uma tratava `span.text` ausente, a outra
// devolvia 0. Largura de celula e' a base de TODO o mapeamento pixel->celula:
// se o renderer e a selecao contarem diferente, o texto copiado nao e' o texto
// destacado, e nada no gate reclamaria.
//
// Sem `import KineinVectis` de proposito: o harness de logica carrega o
// TerminalSelectionController por caminho relativo, sem modulo montado. Cor
// depende do tema e por isso NAO esta aqui — vive no StatusColors.
QtObject {
    // Quantas CELULAS o trecho ocupa. Nao e' o tamanho da string: um
    // caractere fora do BMP ocupa duas unidades UTF-16 e uma celula so', e por
    // isso a contagem de reserva usa `Array.from` em vez de `.length`.
    //
    // O core informa `cells` desde o protocolo 0.56; o ramo do `text` e'
    // compatibilidade defensiva com um frame antigo ainda na fila durante a
    // troca do core.
    function cells(span) {
        if (span.cells !== undefined) {
            return Math.max(0, Number(span.cells));
        }
        const text = span.text !== undefined ? span.text : "";
        return Array.from(String(text)).length;
    }
}
