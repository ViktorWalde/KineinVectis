import QtQuick

// A busca de SIMBOLOS da aba direita (Etapa 3, E3-2): por nome, no projeto
// inteiro (index.symbols — o indice, sem LSP, instantaneo) e recortada
// para a pasta do arquivo aberto. O que o autor lembrava da "aba escondida
// que listava as funcoes do arquivo" — agora tambem para projetos grandes.
// NAO decide nada sobre simbolos: o indice diz; aqui so' o recorte por
// pasta e o debounce. Pede por sinal; recebe pelo roteador.
Item {
    id: root

    property string query: ""
    // O caminho RELATIVO do arquivo ativo ("" sem arquivo); a pasta e' dele.
    property string activeRelativePath: ""
    readonly property string activeFolder: {
        const i = activeRelativePath.lastIndexOf("/");
        return i < 0 ? "" : activeRelativePath.substring(0, i);
    }
    property var results: []
    property int total: 0
    property string indexState: ""
    property bool searching: false
    // Um pedido em voo: so' a resposta dele e' aceita (o Search Everywhere
    // tambem pede index.symbols; cada um guarda o seu).
    property bool waiting: false

    readonly property var folderResults: activeFolder === "" ? []
        : results.filter(s => s.path === activeRelativePath || s.path.indexOf(activeFolder + "/") === 0)
    readonly property bool active: query.trim() !== ""

    signal indexSymbolsRequested(string query)
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
            results = [];
            total = 0;
            searching = false;
            waiting = false;
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
    }

    function handleIndexSymbols(symbols, newTotal, state) {
        if (!waiting) {
            return;
        }
        waiting = false;
        searching = false;
        results = symbols === undefined || symbols === null ? [] : symbols;
        total = newTotal === undefined ? results.length : newTotal;
        indexState = state === undefined ? "" : state;
    }

    function open(symbol) {
        if (symbol && symbol.path !== undefined) {
            openRequested(symbol.path, Number(symbol.line) || 1, 1);
        }
    }

    function clear() {
        query = "";
        results = [];
        total = 0;
        searching = false;
        waiting = false;
    }
}
