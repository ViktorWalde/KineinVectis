import QtQuick

// A ESCADA DE SELECAO: expandir e encolher por unidade sintatica.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03). E' o unico pedaco do
// EditorTextController que tem MEMORIA — `expandHistory` e `lastExpansion` —
// e a memoria e' fragil de proposito. Estado com invariante propria dentro de
// um controller de 573 linhas que so' transforma texto e' vocabulario
// misturado; aqui ele fica sozinho, com a invariante escrita ao lado.
//
// O DESENHO (fatia E3, docs-privada/diario/18). Nao ha maquina de estados: a
// cada Ctrl+W o codigo gera os candidatos que CONTEM a selecao atual — palavra,
// linha sem indentacao, linha inteira, cada par ()/[]/{} que a envolve, e o
// documento — e escolhe o MENOR deles. A escada do JetBrains emerge disso.
//
// A GUARDA DO HISTORICO e' barata e proposital: o historico so' vale enquanto a
// selecao atual for exatamente a ultima expansao registrada SOBRE UM TEXTO DO
// MESMO TAMANHO. Qualquer edicao no meio, ou uma selecao feita com o mouse,
// invalida a pilha em vez de encolher para um lugar que nao existe mais.
//
// LIMITACAO REGISTRADA: a varredura de pares nao entende string nem
// comentario, entao um "(" dentro de uma string conta como abridor. Tolerante
// a texto desbalanceado (fechador sem par no topo da pilha e' ignorado).
Item {
    id: root

    visible: false

    property var surfaceBridge: null
    // As consultas de linha/palavra; ver EditorTextGeometry.qml.
    property var geometry: null

    // Degraus ja subidos, do mais recente para o mais antigo.
    property var expandHistory: []
    // A selecao que a ultima expansao PRODUZIU, com o tamanho do texto na
    // hora. E' o que valida o historico.
    property var lastExpansion: null

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready();
    }

    function reset() {
        expandHistory = [];
        lastExpansion = null;
    }

    // Pares ()/[]/{} que envolvem a selecao, numa varredura unica com pilha.
    // Devolve DOIS candidatos por par: o conteudo e o conteudo com os
    // delimitadores — e' isso que faz Ctrl+W parar duas vezes num parenteses.
    function enclosingPairRanges(selectionStart, selectionEnd, text) {
        const closerToOpener = { ")": "(", "]": "[", "}": "{" };
        const stack = [];
        const ranges = [];
        for (let i = 0; i < text.length; i++) {
            const character = text.charAt(i);
            if (character === "(" || character === "[" || character === "{") {
                stack.push({ character: character, index: i });
            } else if (closerToOpener[character] !== undefined) {
                if (stack.length > 0 && stack[stack.length - 1].character
                        === closerToOpener[character]) {
                    const opener = stack.pop();
                    if (opener.index + 1 <= selectionStart && i >= selectionEnd) {
                        ranges.push({ start: opener.index + 1, end: i });
                        ranges.push({ start: opener.index, end: i + 1 });
                    }
                }
            }
        }
        return ranges;
    }

    function expand() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart,
                                        surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart,
                                      surface.selectionEnd);
        const text = geometry.text();
        if (lastExpansion === null || lastExpansion.start !== selectionStart
                || lastExpansion.end !== selectionEnd
                || lastExpansion.length !== text.length) {
            expandHistory = [];
        }
        const candidates = [];
        if (selectionStart === selectionEnd) {
            const wordStart = geometry.wordStartAt(selectionStart);
            const wordEnd = geometry.wordEndAt(selectionEnd);
            if (wordStart < wordEnd) {
                candidates.push({ start: wordStart, end: wordEnd });
            }
        }
        const lineStart = geometry.lineStartAt(selectionStart);
        const lineEnd = geometry.lineEndAt(selectionEnd);
        candidates.push({
            start: lineStart + geometry.lineIndentAt(lineStart).length,
            end: lineEnd });
        candidates.push({ start: lineStart, end: lineEnd });
        const pairs = enclosingPairRanges(selectionStart, selectionEnd, text);
        for (let i = 0; i < pairs.length; i++) {
            candidates.push(pairs[i]);
        }
        candidates.push({ start: 0, end: text.length });
        let best = null;
        for (let i = 0; i < candidates.length; i++) {
            const candidate = candidates[i];
            if (candidate.start > selectionStart
                    || candidate.end < selectionEnd
                    || (candidate.start === selectionStart
                        && candidate.end === selectionEnd)) {
                continue;
            }
            if (best === null
                    || candidate.end - candidate.start < best.end - best.start) {
                best = candidate;
            }
        }
        if (best === null) {
            return;
        }
        surface.select(best.start, best.end);
        // O TextEdit pode clampar (ex.: não seleciona o "\n" final do
        // documento); registrar a seleção REAL mantém o histórico do
        // shrink válido. Sem mudança efetiva, não vira degrau.
        const appliedStart = Math.min(surface.selectionStart,
                                      surface.selectionEnd);
        const appliedEnd = Math.max(surface.selectionStart,
                                    surface.selectionEnd);
        if (appliedStart === selectionStart && appliedEnd === selectionEnd) {
            return;
        }
        expandHistory.push({ start: selectionStart, end: selectionEnd });
        lastExpansion = { start: appliedStart, end: appliedEnd,
                          length: text.length };
    }

    // Volta um degrau. Sem historico valido, nao faz nada — nunca adivinha.
    function shrink() {
        if (!ready()) {
            return;
        }
        const surface = surfaceBridge.editorSurface;
        const selectionStart = Math.min(surface.selectionStart,
                                        surface.selectionEnd);
        const selectionEnd = Math.max(surface.selectionStart,
                                      surface.selectionEnd);
        const text = geometry.text();
        if (lastExpansion === null || lastExpansion.start !== selectionStart
                || lastExpansion.end !== selectionEnd
                || lastExpansion.length !== text.length
                || expandHistory.length === 0) {
            return;
        }
        const previous = expandHistory.pop();
        surface.select(previous.start, previous.end);
        lastExpansion = { start: previous.start, end: previous.end,
                          length: text.length };
    }
}
