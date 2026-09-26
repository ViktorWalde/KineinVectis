pragma ComponentBehavior: Bound
import QtQuick

// SEARCH EVERYWHERE: a caixa única que acha arquivo, símbolo e comando.
//
// # Por que isto saiu do `SearchController` em 2026-09-02
//
// O `SearchController` guardava DUAS superfícies com o mesmo nome — e o nome
// era a única coisa que elas tinham em comum:
//
// ```text
// BUSCA NO PROJETO      painel de baixo. Persistente, com resultado navegavel,
//                       e com SUBSTITUICAO destrutiva atras dele.
//                       -> SearchController
//
// SEARCH EVERYWHERE     dialogo modal. Efemero, teclado-primeiro, mistura
//                       arquivo + simbolo + comando numa lista so.
//                       -> este arquivo
// ```
//
// A separação não é estética: a de baixo tem um `fs.replace` atrás dela e
// precisa de confirmação; esta abre com `Ctrl+Shift+N`, some no `Enter` e nunca
// escreve nada. Guardá-las juntas fazia o arquivo passar do limite da catraca e,
// pior, misturava o estado de uma com o da outra num `clear()` só.
//
// # Os três modos, e como o usuário os escolhe
//
// ```text
// (vazio ou texto)   arquivos por nome (fd, no core) + comandos da paleta
// @nome              simbolos do DOCUMENTO ativo (lsp.documentSymbols)
// #nome              simbolos do WORKSPACE       (lsp.workspaceSymbols)
// recentMode         arquivos recentes, sem tocar no core
// ```
//
// O prefixo é lido na primeira letra da consulta e nunca é enviado ao core: ele
// é sintaxe da CAIXA, não do protocolo.
//
// # Por que há debounce, e por que ele é curto
//
// Cada tecla dispararia um `fs.findFiles` (que roda `fd`) e um `lsp.*Symbols`.
// 180 ms é o intervalo que a digitação normal não atravessa e que a pausa
// atravessa — o mesmo do reparse sintático, e pelo mesmo motivo.
Item {
    id: root

    property string workspaceRoot: ""
    property alias everywhereModel: everywhereItemsModel
    property bool everywhereVisible: false
    property bool everywhereLoading: false
    property bool everywhereTruncated: false
    property int everywhereIndex: 0
    property string everywhereError: ""
    property var commandList: []
    property string pendingEverywhereQuery: ""
    // A busca por `@` exige documento aberto: sem arquivo ativo não há
    // `documentSymbols` a pedir, e dizer isso é melhor que devolver vazio.
    property bool hasActiveEditorFile: false
    property string symbolFilter: ""
    property var recentFiles: []
    property bool recentMode: false
    property string everywhereTitle: qsTr("Search Everywhere")
    readonly property bool hasCommands: commandList.length > 0

    signal resetAndFocusEverywhereRequested()
    signal findFilesRequested(string query)
    signal documentSymbolsRequested()
    signal workspaceSymbolsRequested(string query)
    // O indice proprio do projeto (pilar 0 do roadmaps/42) responde em
    // milissegundos e sem LSP; o LSP, quando responder, substitui.
    signal indexSymbolsRequested(string query)
    property bool lspSymbolsAnswered: false

    // De QUAL pedido a proxima resposta pode ser; ver EverywhereSymbolOrigin.
    EverywhereSymbolOrigin {
        id: origin
    }

    signal listCommandsRequested()
    signal readFileRequested(string path)
    signal openAtRequested(string path, int line, int column)
    signal commandAccepted(string commandId)
    signal focusEditorRequested()

    visible: false

    ListModel {
        id: everywhereItemsModel
    }

    // A LINHA DA LISTA tem o mesmo formato para as quatro fontes — recente,
    // comando, arquivo e simbolo —, e ele estava escrito quatro vezes, com os
    // mesmos zeros de `line`/`column` em cada uma. Um campo novo exigiria
    // lembrar dos quatro.
    function appendItem(kind, title, path, subtitle, commandId, line, column) {
        everywhereItemsModel.append({
            kind: kind,
            title: title,
            path: path,
            subtitle: subtitle,
            commandId: commandId,
            line: line,
            column: column
        });
    }

    function clear() {
        everywhereItemsModel.clear();
        everywhereVisible = false;
        everywhereLoading = false;
        everywhereTruncated = false;
        everywhereIndex = 0;
        everywhereError = "";
        pendingEverywhereQuery = "";
        recentMode = false;
        everywhereTitle = qsTr("Search Everywhere");
    }

    // As duas eram COPIA do `PathRules` (2026-09-26): mesmo `baseName`, mesmo
    // recorte pela raiz. Reusar o que ja' existe tira a chance de as duas
    // versoes divergirem sem ninguem notar.
    PathRules {
        id: pathRules
    }

    function baseName(path) {
        return pathRules.baseName(path);
    }

    function relativeToRoot(path) {
        return pathRules.relativeTo(workspaceRoot, path);
    }

    function openSearchEverywhere() {
        recentMode = false;
        everywhereTitle = qsTr("Search Everywhere");
        everywhereVisible = true;
        everywhereLoading = false;
        everywhereTruncated = false;
        everywhereIndex = 0;
        everywhereError = "";
        pendingEverywhereQuery = "";
        everywhereItemsModel.clear();
        if (commandList.length === 0) {
            listCommandsRequested();
        }
        appendSearchEverywhereCommands("");
        resetAndFocusEverywhereRequested();
    }

    function openRecentFiles() {
        if (workspaceRoot === "") {
            return;
        }
        recentMode = true;
        everywhereTitle = qsTr("Arquivos recentes");
        everywhereVisible = true;
        everywhereLoading = false;
        everywhereTruncated = false;
        everywhereError = "";
        pendingEverywhereQuery = "";
        appendRecentFiles("");
        resetAndFocusEverywhereRequested();
    }

    function scheduleSearchEverywhere(query) {
        pendingEverywhereQuery = query;
        searchEverywhereDebounce.restart();
    }

    function runSearchEverywhere(query) {
        everywhereError = "";
        everywhereItemsModel.clear();
        everywhereIndex = 0;
        everywhereTruncated = false;
        if (recentMode) {
            appendRecentFiles(query);
            return;
        }
        if (query.length > 0 && (query.charAt(0) === "@" || query.charAt(0) === "#")) {
            runSymbolSearch(query);
            return;
        }
        appendSearchEverywhereCommands(query);
        if (query === "" || workspaceRoot === "") {
            everywhereLoading = false;
            return;
        }
        everywhereLoading = true;
        findFilesRequested(query);
    }

    function appendRecentFiles(query) {
        everywhereItemsModel.clear();
        everywhereLoading = false;
        const needle = query.toLowerCase();
        for (let i = 0; i < recentFiles.length; i++) {
            const path = recentFiles[i];
            const relative = relativeToRoot(path);
            if (needle !== "" && relative.toLowerCase().indexOf(needle) < 0) {
                continue;
            }
            root.appendItem("recent", baseName(path), path, relative, "", 0, 0);
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function runSymbolSearch(query) {
        const plano = origin.planFor(query, workspaceRoot, hasActiveEditorFile);
        symbolFilter = query.charAt(0) === "@" ? plano.needle.toLowerCase() : "";
        if (plano.error !== "") {
            everywhereLoading = false;
            everywhereError = plano.error;
            return;
        }
        everywhereLoading = true;
        if (origin.ask(plano)) {
            // Os simbolos DO DOCUMENTO sao do LSP do arquivo aberto.
            documentSymbolsRequested();
            return;
        }
        // `#nome` no PROJETO: o indice proprio responde desde o primeiro
        // segundo e sem arquivo aberto (pilar 0 do roadmaps/42); o LSP, quando
        // ha' um arquivo aberto para ancora-lo, responde depois e substitui.
        lspSymbolsAnswered = false;
        indexSymbolsRequested(plano.needle);
        if (hasActiveEditorFile) {
            workspaceSymbolsRequested(plano.needle);
        }
    }

    // Os simbolos do INDICE: caminhos relativos a raiz viram absolutos (o
    // clique abre o arquivo), e so' entram enquanto o LSP nao respondeu — a
    // resposta dele e' mais rica e vence quando chega.
    function handleIndexSymbols(symbols, total, state) {
        if (lspSymbolsAnswered || !everywhereVisible) {
            return;
        }
        const absolutos = [];
        for (let i = 0; i < symbols.length; i++) {
            const s = symbols[i];
            const caminho = s.path !== undefined && s.path.indexOf("/") !== 0 && workspaceRoot !== ""
                    ? workspaceRoot + "/" + s.path : s.path;
            absolutos.push({ name: s.name, kind: s.kind, container: s.container,
                             line: s.line, column: 1, path: caminho });
        }
        applySymbols(absolutos);
        // O LSP ainda pode responder: a caixa continua "carregando" so' se o
        // indice nao achou nada (senao a lista ja' e' util).
        everywhereLoading = absolutos.length === 0;
    }

    function appendSearchEverywhereCommands(query) {
        const needle = query.toLowerCase();
        for (let i = 0; i < commandList.length; i++) {
            const command = commandList[i];
            if (command.requiresWorkspace === true && workspaceRoot === "") {
                continue;
            }
            const id = command.id !== undefined ? command.id : "";
            const title = command.title !== undefined ? command.title : id;
            const category = command.category !== undefined
                    ? command.category : qsTr("Comando");
            const description = command.description !== undefined
                    ? command.description : "";
            const shortcut = command.defaultShortcut !== undefined
                    ? command.defaultShortcut : "";
            const haystack = (id + " " + title + " " + category + " "
                              + description + " " + shortcut).toLowerCase();
            if (needle !== "" && haystack.indexOf(needle) < 0) {
                continue;
            }
            root.appendItem("command", title, id,
                            shortcut !== "" ? category + " · " + shortcut : category, id, 0, 0);
        }
    }

    function acceptSearchEverywhere() {
        if (!everywhereVisible || everywhereIndex < 0
                || everywhereIndex >= everywhereItemsModel.count) {
            return;
        }
        const item = everywhereItemsModel.get(everywhereIndex);
        everywhereVisible = false;
        if (item.kind === "command") {
            commandAccepted(item.commandId);
            return;
        }
        if (item.kind === "symbol") {
            openAtRequested(item.path, item.line, item.column);
            return;
        }
        if (item.kind === "recent") {
            readFileRequested(item.path);
            return;
        }
        readFileRequested(workspaceRoot + "/" + item.path);
    }

    function moveEverywhereDown() {
        everywhereIndex = Math.min(everywhereIndex + 1,
                                   everywhereItemsModel.count - 1);
    }

    function moveEverywhereUp() {
        everywhereIndex = Math.max(everywhereIndex - 1, 0);
    }

    function handleFileSearchResults(matches, truncated) {
        everywhereLoading = false;
        everywhereTruncated = truncated;
        for (let i = everywhereItemsModel.count - 1; i >= 0; i--) {
            if (everywhereItemsModel.get(i).kind === "file") {
                everywhereItemsModel.remove(i);
            }
        }
        for (let i = 0; i < matches.length; i++) {
            const match = matches[i];
            const caminho = match.path !== undefined ? match.path : "";
            root.appendItem("file", match.name !== undefined ? match.name : baseName(caminho),
                            caminho, caminho, "", 0, 0);
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    // De qual pedido e' cada resposta: ver EverywhereSymbolOrigin.qml.
    function noteDocumentAnchor(path) {
        origin.noteAnchor(path);
    }

    function handleDocumentSymbols(path, symbols) {
        return origin.acceptsDocument(path) && root.acceptSymbols(symbols);
    }

    function handleWorkspaceSymbols(query, symbols) {
        return origin.acceptsWorkspace(query) && root.acceptSymbols(symbols);
    }

    function acceptSymbols(symbols) {
        lspSymbolsAnswered = true;
        everywhereLoading = false;
        if (!everywhereVisible) {
            return false;
        }
        applySymbols(symbols);
        return true;
    }

    function applySymbols(symbols) {
        everywhereItemsModel.clear();
        for (let i = 0; i < symbols.length; i++) {
            const symbol = symbols[i];
            const name = symbol.name !== undefined ? symbol.name : "";
            if (symbolFilter !== ""
                    && name.toLowerCase().indexOf(symbolFilter) < 0) {
                continue;
            }
            const container = symbol.container !== undefined
                    ? symbol.container + " · " : "";
            const line = symbol.line !== undefined ? Number(symbol.line) : 1;
            root.appendItem("symbol", name, symbol.path !== undefined ? symbol.path : "",
                            (symbol.kind !== undefined ? symbol.kind + " · " : "")
                            + container + relativeToRoot(symbol.path) + ":" + line,
                            "", line,
                            symbol.column !== undefined ? Number(symbol.column) : 1);
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function handleCommandsListed(commands) {
        commandList = commands;
        if (everywhereVisible && !recentMode) {
            runSearchEverywhere(pendingEverywhereQuery);
        }
    }

    // A recusa do core vira texto NA CAIXA, e não linha de log: "abra um
    // arquivo com LSP" é a resposta à pergunta que o usuário acabou de fazer.
    function handleRequestFailed(method, message) {
        if (method === "fs.findFiles") {
            everywhereLoading = false;
            everywhereError = message;
            everywhereVisible = true;
        }
        if (method === "lsp.documentSymbols" || method === "lsp.workspaceSymbols") {
            everywhereLoading = false;
            everywhereError = message;
        }
    }

    Timer {
        id: searchEverywhereDebounce

        interval: 180
        repeat: false
        onTriggered: root.runSearchEverywhere(root.pendingEverywhereQuery)
    }
}
