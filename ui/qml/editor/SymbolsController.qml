import QtQuick

// A busca de SIMBOLOS da aba direita (Etapa 3, E3-2): por nome, no projeto
// inteiro e recortada para a pasta do arquivo aberto. O que o autor lembrava da
// "aba escondida que listava as funcoes do arquivo" — agora tambem para
// projetos grandes.
//
// DUAS FONTES desde a L1 (2026-09-26), e nenhuma substitui a outra:
//
//   INDICE  responde desde o primeiro segundo e SEM arquivo aberto, no projeto
//           inteiro (`index.symbols`);
//   LSP     responde depois, so' para o arquivo aberto, e sabe mais
//           (`lsp.documentSymbols`).
//
// A mesclagem (deduplicar por arquivo/linha, LSP vencendo o empate) e' regra
// pura e mora no `SymbolMergeRules`. Aqui ficam o recorte por pasta, o debounce
// e a guarda de qual resposta pertence a este pedido.
Item {
    id: root

    property string query: ""
    // O caminho RELATIVO do arquivo ativo ("" sem arquivo); a pasta e' dele.
    property string activeRelativePath: ""
    readonly property string activeFolder: {
        const i = activeRelativePath.lastIndexOf("/");
        return i < 0 ? "" : activeRelativePath.substring(0, i);
    }
    // O que cada fonte trouxe, antes da mesclagem.
    property var indexResults: []
    property var lspResults: []
    readonly property var results: mergeRules.merge(indexResults, lspResults)
    readonly property bool showSource: mergeRules.sourceWorthShowing(results)

    // O caminho ABSOLUTO do arquivo ativo, para guardar a resposta do LSP. Uma
    // resposta atrasada para outro arquivo nao pinta esta lista.
    property string activeAbsolutePath: ""
    property bool waitingLsp: false
    property int total: 0
    property string indexState: ""
    property bool searching: false
    // Um pedido em voo: so' a resposta dele e' aceita (o Search Everywhere
    // tambem pede index.symbols; cada um guarda o seu).
    property bool waiting: false

    readonly property var folderResults: activeFolder === "" ? []
        : results.filter(s => s.path === activeRelativePath || s.path.indexOf(activeFolder + "/") === 0)
    readonly property bool active: query.trim() !== ""

    SymbolMergeRules {
        id: mergeRules
    }

    signal indexSymbolsRequested(string query)
    signal documentSymbolsRequested()
    signal openRequested(string path, int line, int column)

    visible: false

    Timer {
        id: debounce

        interval: 150
        repeat: false
        onTriggered: root.requestNow()
    }

    function setQuery(text) {
        query = text === undefined ? "" : text;
        if (!active) {
            root.forgetResults();
            return;
        }
        searching = true;
        debounce.restart();
    }

    function requestNow() {
        if (!active) {
            return;
        }
        waiting = true;
        indexSymbolsRequested(query.trim());
        // O LSP so' responde sobre o arquivo ABERTO; sem ele, o indice responde
        // sozinho e a aba continua util.
        if (root.activeAbsolutePath !== "") {
            root.waitingLsp = true;
            documentSymbolsRequested();
        }
    }

    function forgetResults() {
        indexResults = [];
        lspResults = [];
        total = 0;
        searching = false;
        waiting = false;
        waitingLsp = false;
    }

    function handleIndexSymbols(symbols, newTotal, state) {
        if (!waiting) {
            return;
        }
        waiting = false;
        searching = false;
        indexResults = symbols === undefined || symbols === null ? [] : symbols;
        total = newTotal === undefined ? indexResults.length : newTotal;
        indexState = state === undefined ? "" : state;
    }

    // A resposta do LSP, com a guarda que a L1 trouxe: `path` identifica o
    // pedido, e uma resposta para OUTRO arquivo nao entra aqui.
    function handleDocumentSymbols(path, symbols) {
        if (!waitingLsp || path !== root.activeAbsolutePath) {
            return false;
        }
        waitingLsp = false;
        // O LSP devolve caminho ABSOLUTO; a lista da aba fala em relativo ao
        // workspace, e misturar os dois quebraria a deduplicacao por posicao.
        const relativos = [];
        const lista = symbols === undefined || symbols === null ? [] : symbols;
        for (let i = 0; i < lista.length; i++) {
            const copia = ({});
            for (const campo in lista[i]) {
                copia[campo] = lista[i][campo];
            }
            copia.path = root.activeRelativePath;
            relativos.push(copia);
        }
        lspResults = relativos;
        return true;
    }

    function open(symbol) {
        if (symbol && symbol.path !== undefined) {
            openRequested(symbol.path, Number(symbol.line) || 1, 1);
        }
    }

    function clear() {
        query = "";
        root.forgetResults();
    }
}
