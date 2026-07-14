import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property alias searchModel: searchItemsModel
    property alias everywhereModel: everywhereItemsModel
    property bool caseSensitive: false
    property bool searching: false
    property bool replaceMode: false
    property bool replacing: false
    property string replaceError: ""
    property string replaceSummary: ""
    property bool everywhereVisible: false
    property bool everywhereLoading: false
    property bool everywhereTruncated: false
    property int everywhereIndex: 0
    property string everywhereError: ""
    property var commandList: []
    property bool searchTruncated: false
    property string pendingEverywhereQuery: ""
    property bool hasActiveEditorFile: false
    property string symbolFilter: ""
    property var recentFiles: []
    property bool recentMode: false
    property string everywhereTitle: qsTr("Search Everywhere")
    readonly property bool hasCommands: commandList.length > 0

    signal showTabRequested(string tab)
    signal focusSearchInputRequested()
    signal resetAndFocusEverywhereRequested()
    signal searchInFilesRequested(string query, bool caseSensitive)
    signal replaceInFilesRequested(string query, string replacement, bool caseSensitive)
    signal focusReplaceInputRequested()
    signal findFilesRequested(string query)
    signal documentSymbolsRequested()
    signal workspaceSymbolsRequested(string query)
    signal listCommandsRequested()
    signal readFileRequested(string path)
    signal openAtRequested(string path, int line, int column)
    signal commandAccepted(string commandId)
    signal focusEditorRequested()

    visible: false

    ListModel {
        id: searchItemsModel
    }

    ListModel {
        id: everywhereItemsModel
    }

    function clear() {
        searchItemsModel.clear();
        clearSearchEverywhere();
        caseSensitive = false;
        searching = false;
        replaceMode = false;
        replacing = false;
        replaceError = "";
        replaceSummary = "";
    }

    function clearSearchEverywhere() {
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

    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function openSearchPanel() {
        if (workspaceRoot === "") {
            return;
        }
        showTabRequested("search");
        focusSearchInputRequested();
    }

    function openReplacePanel() {
        if (workspaceRoot === "") {
            return;
        }
        replaceMode = true;
        replaceError = "";
        replaceSummary = "";
        showTabRequested("search");
        focusReplaceInputRequested();
    }

    function runSearch(query) {
        if (query === "" || workspaceRoot === "" || searching) {
            return;
        }
        searching = true;
        searchTruncated = false;
        searchItemsModel.clear();
        searchInFilesRequested(query, caseSensitive);
    }

    function runReplace(query, replacement) {
        if (query === "" || workspaceRoot === "" || replacing) {
            replaceError = query === "" ? qsTr("Informe o texto a substituir.") : "";
            return;
        }
        replacing = true;
        replaceError = "";
        replaceSummary = "";
        replaceInFilesRequested(query, replacement, caseSensitive);
    }

    function rejectReplaceForDirtyEditors() {
        replacing = false;
        replaceError = qsTr("Salve todas as abas modificadas antes de substituir no projeto.");
    }

    function handleReplaceResult(files, replacements) {
        replacing = false;
        replaceError = "";
        // Os offsets do resultado anterior deixaram de representar o disco.
        // O usuario pode executar uma nova busca quando quiser conferir o
        // estado posterior, sem navegar por resultados obsoletos.
        searchItemsModel.clear();
        searchTruncated = false;
        replaceSummary = qsTr("%1 ocorrencia(s) em %2 arquivo(s).")
                .arg(replacements).arg(files.length);
    }

    function toggleCaseAndRun(query) {
        caseSensitive = !caseSensitive;
        runSearch(query);
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
            everywhereItemsModel.append({
                kind: "recent",
                title: baseName(path),
                path: path,
                subtitle: relative,
                commandId: "",
                line: 0,
                column: 0
            });
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function runSymbolSearch(query) {
        const isDocument = query.charAt(0) === "@";
        const needle = query.substring(1).trim();
        symbolFilter = isDocument ? needle.toLowerCase() : "";
        if (workspaceRoot === "" || !hasActiveEditorFile) {
            everywhereLoading = false;
            everywhereError =
                    qsTr("Abra um arquivo com LSP para buscar símbolos.");
            return;
        }
        if (!isDocument && needle === "") {
            everywhereLoading = false;
            everywhereError = qsTr("Digite o nome do símbolo após #.");
            return;
        }
        everywhereLoading = true;
        if (isDocument) {
            documentSymbolsRequested();
        } else {
            workspaceSymbolsRequested(needle);
        }
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
            everywhereItemsModel.append({
                kind: "command",
                title: title,
                path: id,
                subtitle: shortcut !== "" ? category + " · " + shortcut : category,
                commandId: id,
                line: 0,
                column: 0
            });
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

    function handleSearchResults(matches, truncated) {
        searching = false;
        searchItemsModel.clear();
        for (let i = 0; i < matches.length; i++) {
            const match = matches[i];
            searchItemsModel.append({
                path: match.path,
                line: match.line !== undefined ? Number(match.line) : 1,
                column: match.column !== undefined ? Number(match.column) : 1,
                preview: match.preview !== undefined ? match.preview : ""
            });
        }
        searchTruncated = truncated;
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
            everywhereItemsModel.append({
                kind: "file",
                title: match.name !== undefined ? match.name : baseName(match.path),
                path: match.path !== undefined ? match.path : "",
                subtitle: match.path !== undefined ? match.path : "",
                commandId: "",
                line: 0,
                column: 0
            });
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    function handleSymbolsResolved(symbols) {
        everywhereLoading = false;
        if (!everywhereVisible) {
            return;
        }
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
            everywhereItemsModel.append({
                kind: "symbol",
                title: name,
                path: symbol.path !== undefined ? symbol.path : "",
                subtitle: (symbol.kind !== undefined ? symbol.kind + " · " : "")
                          + container + relativeToRoot(symbol.path) + ":" + line,
                commandId: "",
                line: line,
                column: symbol.column !== undefined ? Number(symbol.column) : 1
            });
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function handleCommandsListed(commands) {
        commandList = commands;
        if (everywhereVisible && !recentMode) {
            runSearchEverywhere(pendingEverywhereQuery);
        }
    }

    function handleRequestFailed(method, message) {
        if (method === "fs.search") {
            searching = false;
        }
        if (method === "fs.replace") {
            replacing = false;
            replaceError = message;
        }
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
