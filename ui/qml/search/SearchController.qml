import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property alias searchModel: searchItemsModel
    property alias everywhereModel: everywhereItemsModel
    property bool caseSensitive: false
    property bool searching: false
    property bool everywhereVisible: false
    property bool everywhereLoading: false
    property bool everywhereTruncated: false
    property int everywhereIndex: 0
    property string everywhereError: ""
    property var commandList: []
    property bool searchTruncated: false
    property string pendingEverywhereQuery: ""
    readonly property bool hasCommands: commandList.length > 0

    signal showTabRequested(string tab)
    signal focusSearchInputRequested()
    signal resetAndFocusEverywhereRequested()
    signal searchInFilesRequested(string query, bool caseSensitive)
    signal findFilesRequested(string query)
    signal listCommandsRequested()
    signal readFileRequested(string path)
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
    }

    function clearSearchEverywhere() {
        everywhereItemsModel.clear();
        everywhereVisible = false;
        everywhereLoading = false;
        everywhereTruncated = false;
        everywhereIndex = 0;
        everywhereError = "";
        pendingEverywhereQuery = "";
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

    function runSearch(query) {
        if (query === "" || workspaceRoot === "" || searching) {
            return;
        }
        searching = true;
        searchTruncated = false;
        searchItemsModel.clear();
        searchInFilesRequested(query, caseSensitive);
    }

    function toggleCaseAndRun(query) {
        caseSensitive = !caseSensitive;
        runSearch(query);
    }

    function openSearchEverywhere() {
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

    function scheduleSearchEverywhere(query) {
        pendingEverywhereQuery = query;
        searchEverywhereDebounce.restart();
    }

    function runSearchEverywhere(query) {
        everywhereError = "";
        everywhereItemsModel.clear();
        everywhereIndex = 0;
        appendSearchEverywhereCommands(query);
        if (query === "" || workspaceRoot === "") {
            everywhereLoading = false;
            everywhereTruncated = false;
            return;
        }
        everywhereLoading = true;
        findFilesRequested(query);
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
                commandId: id
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
                commandId: ""
            });
        }
        everywhereIndex = everywhereItemsModel.count > 0 ? 0 : -1;
    }

    function handleCommandsListed(commands) {
        commandList = commands;
        if (everywhereVisible) {
            runSearchEverywhere(pendingEverywhereQuery);
        }
    }

    function handleRequestFailed(method, message) {
        if (method === "fs.search") {
            searching = false;
        }
        if (method === "fs.findFiles") {
            everywhereLoading = false;
            everywhereError = message;
            everywhereVisible = true;
        }
    }

    Timer {
        id: searchEverywhereDebounce

        interval: 180
        repeat: false
        onTriggered: root.runSearchEverywhere(root.pendingEverywhereQuery)
    }
}
