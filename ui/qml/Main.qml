// Layout em "ilhas" arredondadas inspirado nas IDEs JetBrains:
// header, explorer em arvore, editor com abas, logs recolhiveis, status bar.
import QtQuick
import QtQuick.Window
import KineinVectis

Window {
    id: root

    width: 1280
    height: 800
    minimumWidth: 800
    minimumHeight: 500
    visible: true
    title: coreClient.workspaceName !== ""
           ? qsTr("%1 - Kinein Vectis").arg(coreClient.workspaceName)
           : qsTr("Kinein Vectis")
    color: Theme.background0

    property int currentTab: -1
    property bool loadingEditorText: false
    property bool showBottomPanel: false
    property string bottomTab: "logs"
    property bool showExplorer: true
    property bool showAssistant: false
    property string activeWorkspaceRoot: ""
    property var toolsList: []
    property string pendingJumpPath: ""
    property int pendingJumpLine: 0
    property int pendingJumpColumn: 0
    property string hoverText: ""
    property bool hoverVisible: false
    property bool completionVisible: false
    property int completionIndex: 0
    property int completionPrefixStart: -1
    property var completionAll: []
    property bool usagesVisible: false
    property bool renameDialogVisible: false
    property string renameError: ""
    property bool createDialogVisible: false
    property string createDialogKind: "file"
    property string createDialogParentPath: ""
    property string createDialogError: ""
    property string explorerSelectedPath: ""
    property string explorerSelectedKind: ""
    property bool searchCaseSensitive: false
    property bool searchTruncated: false
    property bool searching: false
    property bool searchEverywhereVisible: false
    property bool searchEverywhereLoading: false
    property bool searchEverywhereTruncated: false
    property int searchEverywhereIndex: 0
    property string searchEverywhereError: ""
    property var commandList: []
    readonly property string editorIndent: "    "
    property var pendingReloads: ({})
    // Menu de contexto e dialogos de rename/delete do Project panel.
    property bool entryMenuVisible: false
    property real entryMenuX: 0
    property real entryMenuY: 0
    property string entryMenuPath: ""
    property string entryMenuKind: ""
    property string entryMenuName: ""
    property bool entryRenameVisible: false
    property string entryRenamePath: ""
    property string entryRenameKind: ""
    property string entryRenameError: ""
    property bool entryDeleteVisible: false
    property string entryDeletePath: ""
    property string entryDeleteKind: ""
    property string entryDeleteName: ""
    property string entryDeleteError: ""

    function startBuild() {
        if (coreClient.building || coreClient.workspaceRoot === "") {
            return;
        }
        buildOutputModel.clear();
        removeProblemsBySource("build");
        showBottomPanel = true;
        bottomTab = "build";
        coreClient.runBuild();
    }

    property string testSummary: ""

    function startTests() {
        if (coreClient.testing || coreClient.workspaceRoot === "") {
            return;
        }
        testModel.clear();
        testSummary = qsTr("rodando testes...");
        showBottomPanel = true;
        bottomTab = "tests";
        coreClient.runTests("");
    }

    function startQuality() {
        if (coreClient.analyzing || coreClient.workspaceRoot === "") {
            return;
        }
        removeProblemsBySource("quality");
        showBottomPanel = true;
        bottomTab = "problems";
        coreClient.runQuality();
    }

    function removeProblemsBySource(source) {
        for (let i = problemsModel.count - 1; i >= 0; i--) {
            if (problemsModel.get(i).source === source) {
                problemsModel.remove(i);
            }
        }
    }

    function relativeToRoot(path) {
        const root = coreClient.workspaceRoot;
        if (root !== "" && path.indexOf(root + "/") === 0) {
            return path.substring(root.length + 1);
        }
        return path;
    }

    function appendBuildLine(line) {
        buildOutputModel.append({ line: line });
        while (buildOutputModel.count > 2000) {
            buildOutputModel.remove(0);
        }
    }

    function problemColor(severity) {
        if (severity === "error") {
            return Theme.errorSoft;
        }
        if (severity === "warning") {
            return Theme.warningSoft;
        }
        return Theme.infoSoft;
    }

    function testStatusColor(status) {
        if (status === "passed") {
            return Theme.successSoft;
        }
        if (status === "failed") {
            return Theme.errorSoft;
        }
        return Theme.textMuted;
    }

    function openDiagnostic(file, line, column) {
        if (file === "") {
            return;
        }
        const path = file.startsWith("/")
                ? file : coreClient.workspaceRoot + "/" + file;
        pendingJumpPath = path;
        pendingJumpLine = line;
        pendingJumpColumn = column;
        for (let i = 0; i < openFiles.count; i++) {
            if (openFiles.get(i).path === path) {
                selectTab(i);
                jumpToPending();
                return;
            }
        }
        coreClient.readFile(path);
    }

    function jumpToPending() {
        if (pendingJumpPath === "") {
            return;
        }
        const text = editor.text;
        let offset = 0;
        for (let currentLine = 1; currentLine < pendingJumpLine; currentLine++) {
            const next = text.indexOf("\n", offset);
            if (next < 0) {
                break;
            }
            offset = next + 1;
        }
        if (pendingJumpColumn > 0) {
            offset += pendingJumpColumn - 1;
        }
        editor.cursorPosition = Math.min(offset, text.length);
        editor.forceActiveFocus();
        pendingJumpPath = "";
    }

    function cursorLineColumn() {
        const cursor = editor.cursorPosition;
        let line = 1;
        let lineStart = 0;
        let offset = 0;
        while (offset < cursor) {
            const next = editor.text.indexOf("\n", offset);
            if (next < 0 || next >= cursor) {
                break;
            }
            line++;
            lineStart = next + 1;
            offset = next + 1;
        }
        return {
            line: line,
            column: cursor - lineStart + 1
        };
    }

    function requestDefinition() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        hoverVisible = false;
        const position = cursorLineColumn();
        coreClient.requestDefinition(openFiles.get(currentTab).path, editor.text,
                                     position.line, position.column);
    }

    function requestHover() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        const position = cursorLineColumn();
        coreClient.requestHover(openFiles.get(currentTab).path, editor.text,
                                position.line, position.column);
    }

    // ── Completion / rename / usos (Fase 5.2) ─────────────────────────────
    function isWordChar(ch) {
        return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z")
                || (ch >= "0" && ch <= "9") || ch === "_";
    }

    function wordStartAt(position) {
        let start = position;
        while (start > 0 && isWordChar(editor.text.charAt(start - 1))) {
            start--;
        }
        return start;
    }

    function currentWord() {
        const start = wordStartAt(editor.cursorPosition);
        let end = editor.cursorPosition;
        while (end < editor.text.length && isWordChar(editor.text.charAt(end))) {
            end++;
        }
        return editor.text.substring(start, end);
    }

    function requestCompletion() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        hoverVisible = false;
        completionPrefixStart = wordStartAt(editor.cursorPosition);
        const position = cursorLineColumn();
        coreClient.requestCompletion(openFiles.get(currentTab).path, editor.text,
                                     position.line, position.column);
    }

    function refilterCompletions() {
        if (completionPrefixStart < 0
                || completionPrefixStart > editor.cursorPosition) {
            completionVisible = false;
            return;
        }
        const prefix = editor.text.substring(completionPrefixStart,
                                             editor.cursorPosition).toLowerCase();
        completionModel.clear();
        for (let i = 0; i < completionAll.length; i++) {
            const item = completionAll[i];
            const insertText = item.insertText !== undefined
                    ? item.insertText : item.label;
            const label = item.label !== undefined ? item.label : insertText;
            if (prefix === ""
                    || insertText.toLowerCase().indexOf(prefix) === 0
                    || label.toLowerCase().indexOf(prefix) === 0) {
                completionModel.append({
                    label: label,
                    insertText: insertText,
                    detail: item.detail !== undefined ? item.detail : "",
                    kind: item.kind !== undefined ? item.kind : ""
                });
            }
        }
        completionIndex = 0;
        completionVisible = completionModel.count > 0;
    }

    function acceptCompletion() {
        if (!completionVisible || completionIndex < 0
                || completionIndex >= completionModel.count) {
            return;
        }
        const insertText = completionModel.get(completionIndex).insertText;
        const start = completionPrefixStart >= 0
                ? completionPrefixStart : editor.cursorPosition;
        const end = editor.cursorPosition;
        completionVisible = false;
        completionPrefixStart = -1;
        editor.remove(start, end);
        editor.insert(start, insertText);
        editor.cursorPosition = start + insertText.length;
        editor.forceActiveFocus();
    }

    function refreshSemanticTokens() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        coreClient.requestSemanticTokens(openFiles.get(currentTab).path,
                                         editor.text);
    }

    function requestUsages() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        hoverVisible = false;
        const position = cursorLineColumn();
        coreClient.requestReferences(openFiles.get(currentTab).path, editor.text,
                                     position.line, position.column);
    }

    function openRenameDialog() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        completionVisible = false;
        hoverVisible = false;
        renameError = "";
        renameInput.text = currentWord();
        renameDialogVisible = true;
        renameInput.forceActiveFocus();
        renameInput.selectAll();
    }

    function confirmRename() {
        if (currentTab < 0 || currentTab >= openFiles.count) {
            return;
        }
        const name = renameInput.text.trim();
        if (name === "") {
            renameError = qsTr("Informe um novo nome.");
            return;
        }
        for (let i = 0; i < openFiles.count; i++) {
            if (i !== currentTab && openFiles.get(i).modified) {
                renameError =
                        qsTr("Salve as outras abas modificadas antes de renomear.");
                return;
            }
        }
        const position = cursorLineColumn();
        renameDialogVisible = false;
        editor.forceActiveFocus();
        coreClient.requestRename(openFiles.get(currentTab).path, editor.text,
                                 position.line, position.column, name);
    }

    function sendAssistantMessage(body) {
        const text = body.trim();
        if (text === "") {
            return;
        }
        assistantModel.append({ role: "user", body: text });
        assistantModel.append({
            role: "system",
            body: qsTr("Nenhum provider de IA esta configurado ainda. Os providers "
                       + "(Ollama local, GPT CLI, Claude CLI) chegam na Fase 7 do "
                       + "roadmap, sempre com confirmacao explicita antes de "
                       + "qualquer codigo sair da maquina.")
        });
        assistantInput.text = "";
    }

    function toggleBottomTab(tab) {
        if (showBottomPanel && bottomTab === tab) {
            showBottomPanel = false;
            return;
        }
        bottomTab = tab;
        showBottomPanel = true;
        if (tab === "tools" && toolsList.length === 0) {
            coreClient.detectTools();
        }
    }

    function toolStatusColor(status) {
        if (status === "detected") {
            return Theme.successSoft;
        }
        if (status === "failed") {
            return Theme.warningSoft;
        }
        return Theme.errorSoft;
    }

    // Botao quadrado da sidebar de icones, estilo tool window JetBrains.
    component SideButton: Rectangle {
        id: sideButton

        property string glyph: ""
        property string tip: ""
        property bool active: false

        signal activated()

        width: 30
        height: 30
        radius: Theme.radius
        color: active ? Theme.accentDim
                      : (sideButtonArea.containsMouse ? Theme.surface2 : "transparent")

        Text {
            anchors.centerIn: parent
            text: sideButton.glyph
            color: sideButton.active ? Theme.accent : Theme.textSecondary
            font.pixelSize: 14
        }

        MouseArea {
            id: sideButtonArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: sideButton.activated()
        }
    }

    function kindLabel(kind) {
        const labels = {
            rustCargo: "Rust/Cargo",
            cmake: "CMake",
            maven: "Maven",
            gradle: "Gradle",
            python: "Python",
            unknown: qsTr("Projeto")
        };
        return labels[kind] !== undefined ? labels[kind] : kind;
    }

    function baseName(path) {
        return path.substring(path.lastIndexOf("/") + 1);
    }

    function parentDir(path) {
        const slash = path.lastIndexOf("/");
        return slash > 0 ? path.substring(0, slash) : path;
    }

    // ── Arvore do explorer ────────────────────────────────────────────────
    function rowIndexForPath(path) {
        for (let i = 0; i < treeModel.count; i++) {
            if (treeModel.get(i).path === path) {
                return i;
            }
        }
        return -1;
    }

    function collapseRow(index) {
        const depth = treeModel.get(index).depth;
        while (index + 1 < treeModel.count && treeModel.get(index + 1).depth > depth) {
            treeModel.remove(index + 1);
        }
        treeModel.setProperty(index, "expanded", false);
    }

    function insertEntries(startIndex, entries, depth, parentPath) {
        for (let i = 0; i < entries.length; i++) {
            treeModel.insert(startIndex + i, {
                path: parentPath + "/" + entries[i].name,
                name: entries[i].name,
                kind: entries[i].kind,
                depth: depth,
                expanded: false
            });
        }
    }

    function selectedCreateParent() {
        if (explorerSelectedPath === "") {
            return coreClient.workspaceRoot;
        }
        return explorerSelectedKind === "directory"
                ? explorerSelectedPath : parentDir(explorerSelectedPath);
    }

    function openCreateDialog(kind) {
        if (coreClient.workspaceRoot === "") {
            return;
        }
        createDialogKind = kind;
        createDialogParentPath = selectedCreateParent();
        createDialogError = "";
        createNameInput.text = "";
        createDialogVisible = true;
        createNameInput.forceActiveFocus();
    }

    function confirmCreateEntry() {
        const name = createNameInput.text.trim();
        if (name === "") {
            createDialogError = qsTr("Informe um nome.");
            return;
        }
        if (name.indexOf("/") >= 0) {
            createDialogError = qsTr("Use apenas um nome, sem barras.");
            return;
        }
        const path = createDialogParentPath + "/" + name;
        if (createDialogKind === "directory") {
            coreClient.createDirectory(path);
        } else {
            coreClient.createFile(path, "");
        }
    }

    // ── Menu de contexto / rename / delete do Project panel ────────────────
    function openEntryMenu(path, kind, name, sceneX, sceneY) {
        root.entryMenuPath = path;
        root.entryMenuKind = kind;
        root.entryMenuName = name;
        root.entryMenuX = Math.max(0, Math.min(sceneX, root.width - 172));
        root.entryMenuY = Math.max(0, Math.min(sceneY, root.height - 96));
        root.entryMenuVisible = true;
    }

    function openEntryRename() {
        if (root.entryMenuPath === "" || coreClient.workspaceRoot === "") {
            return;
        }
        root.entryMenuVisible = false;
        root.entryRenamePath = root.entryMenuPath;
        root.entryRenameKind = root.entryMenuKind;
        root.entryRenameError = "";
        entryRenameInput.text = root.entryMenuName;
        root.entryRenameVisible = true;
        entryRenameInput.forceActiveFocus();
        entryRenameInput.selectAll();
    }

    function confirmEntryRename() {
        const name = entryRenameInput.text.trim();
        if (name === "") {
            root.entryRenameError = qsTr("Informe um nome.");
            return;
        }
        if (name.indexOf("/") >= 0) {
            root.entryRenameError = qsTr("Use apenas um nome, sem barras.");
            return;
        }
        if (name === root.baseName(root.entryRenamePath)) {
            root.entryRenameVisible = false;
            editor.forceActiveFocus();
            return;
        }
        const target = root.parentDir(root.entryRenamePath) + "/" + name;
        coreClient.renamePath(root.entryRenamePath, target);
    }

    function openEntryDelete() {
        if (root.entryMenuPath === "" || coreClient.workspaceRoot === "") {
            return;
        }
        root.entryMenuVisible = false;
        root.entryDeletePath = root.entryMenuPath;
        root.entryDeleteKind = root.entryMenuKind;
        root.entryDeleteName = root.entryMenuName;
        root.entryDeleteError = "";
        root.entryDeleteVisible = true;
    }

    function confirmEntryDelete() {
        root.entryDeleteError = "";
        coreClient.deletePath(root.entryDeletePath);
    }

    // Reaponta as abas abertas apos um rename/move (arquivo ou pasta pai).
    function applyPathRenameToTabs(from, to) {
        for (let i = 0; i < openFiles.count; i++) {
            const current = openFiles.get(i).path;
            let updated = "";
            if (current === from) {
                updated = to;
            } else if (current.indexOf(from + "/") === 0) {
                updated = to + current.substring(from.length);
            } else {
                continue;
            }
            openFiles.setProperty(i, "path", updated);
            openFiles.setProperty(i, "name", root.baseName(updated));
            if (i === root.currentTab) {
                editorHighlighter.filePath = updated;
            }
        }
    }

    // Fecha abas do arquivo removido (ou de arquivos sob a pasta removida).
    function closeTabsUnderPath(path) {
        for (let i = openFiles.count - 1; i >= 0; i--) {
            const current = openFiles.get(i).path;
            if (current === path || current.indexOf(path + "/") === 0) {
                root.closeTab(i);
            }
        }
    }

    // ── Abas do editor ────────────────────────────────────────────────────
    function storeCurrentEditor() {
        if (currentTab >= 0 && currentTab < openFiles.count) {
            openFiles.setProperty(currentTab, "content", editor.text);
        }
    }

    function selectTab(index) {
        if (index === currentTab) {
            return;
        }
        storeCurrentEditor();
        completionVisible = false;
        completionPrefixStart = -1;
        currentTab = index;
        loadingEditorText = true;
        editor.text = index >= 0 ? openFiles.get(index).content : "";
        loadingEditorText = false;
        editorHighlighter.filePath = index >= 0 ? openFiles.get(index).path : "";
        refreshSemanticTokens();
    }

    function closeTab(index) {
        openFiles.remove(index);
        if (openFiles.count === 0) {
            currentTab = -1;
            loadingEditorText = true;
            editor.text = "";
            loadingEditorText = false;
            editorHighlighter.filePath = "";
            return;
        }
        const next = Math.min(index, openFiles.count - 1);
        currentTab = -1;
        selectTab(next);
    }

    function saveCurrentFile() {
        if (currentTab < 0) {
            return;
        }
        storeCurrentEditor();
        coreClient.writeFile(openFiles.get(currentTab).path, editor.text);
    }

    function lineStartAt(position) {
        const before = Math.max(0, position - 1);
        const previousBreak = editor.text.lastIndexOf("\n", before);
        return previousBreak < 0 ? 0 : previousBreak + 1;
    }

    function lineEndAt(position) {
        const nextBreak = editor.text.indexOf("\n", position);
        return nextBreak < 0 ? editor.text.length : nextBreak;
    }

    function lineIndentAt(lineStart) {
        let end = lineStart;
        while (end < editor.text.length) {
            const ch = editor.text.charAt(end);
            if (ch !== " " && ch !== "\t") {
                break;
            }
            end++;
        }
        return editor.text.substring(lineStart, end);
    }

    function selectedLineStarts() {
        const selectionStart = Math.min(editor.selectionStart, editor.selectionEnd);
        const selectionEnd = Math.max(editor.selectionStart, editor.selectionEnd);
        const effectiveEnd = selectionEnd > selectionStart
                ? Math.max(selectionStart, selectionEnd - 1) : editor.cursorPosition;
        const starts = [];
        let lineStart = root.lineStartAt(selectionStart);
        const lastLineStart = root.lineStartAt(effectiveEnd);
        while (lineStart <= lastLineStart) {
            starts.push(lineStart);
            const lineEnd = root.lineEndAt(lineStart);
            if (lineEnd >= editor.text.length) {
                break;
            }
            lineStart = lineEnd + 1;
        }
        return starts;
    }

    function indentEditorSelection() {
        const starts = selectedLineStarts();
        const hadSelection = editor.selectionStart !== editor.selectionEnd;
        const selectionStart = Math.min(editor.selectionStart, editor.selectionEnd);
        const selectionEnd = Math.max(editor.selectionStart, editor.selectionEnd);
        const cursor = editor.cursorPosition;
        for (let i = starts.length - 1; i >= 0; i--) {
            editor.insert(starts[i], editorIndent);
        }
        if (hadSelection) {
            editor.select(selectionStart + editorIndent.length,
                          selectionEnd + starts.length * editorIndent.length);
        } else {
            editor.cursorPosition = cursor + editorIndent.length;
        }
    }

    function unindentEditorSelection() {
        const starts = selectedLineStarts();
        const hadSelection = editor.selectionStart !== editor.selectionEnd;
        const selectionStart = Math.min(editor.selectionStart, editor.selectionEnd);
        const selectionEnd = Math.max(editor.selectionStart, editor.selectionEnd);
        const cursor = editor.cursorPosition;
        let removedBeforeStart = 0;
        let removedBeforeEnd = 0;
        let removedBeforeCursor = 0;
        for (let i = starts.length - 1; i >= 0; i--) {
            const start = starts[i];
            let removeCount = 0;
            if (editor.text.charAt(start) === "\t") {
                removeCount = 1;
            } else {
                while (removeCount < editorIndent.length
                       && editor.text.charAt(start + removeCount) === " ") {
                    removeCount++;
                }
            }
            if (removeCount === 0) {
                continue;
            }
            editor.remove(start, start + removeCount);
            if (start < selectionStart) {
                removedBeforeStart += removeCount;
            }
            if (start < selectionEnd) {
                removedBeforeEnd += removeCount;
            }
            if (start < cursor) {
                removedBeforeCursor += removeCount;
            }
        }
        if (hadSelection) {
            editor.select(Math.max(0, selectionStart - removedBeforeStart),
                          Math.max(0, selectionEnd - removedBeforeEnd));
        } else {
            editor.cursorPosition = Math.max(root.lineStartAt(cursor), cursor - removedBeforeCursor);
        }
    }

    function insertEditorNewline() {
        const cursor = editor.cursorPosition;
        const lineStart = root.lineStartAt(cursor);
        const beforeCursor = editor.text.substring(lineStart, cursor);
        let indent = root.lineIndentAt(lineStart);
        const trimmed = beforeCursor.replace(/[ \t]+$/, "");
        if (trimmed.endsWith("{") || trimmed.endsWith("(")
                || trimmed.endsWith("[") || trimmed.endsWith(":")) {
            indent += editorIndent;
        }
        editor.insert(cursor, "\n" + indent);
        editor.cursorPosition = cursor + 1 + indent.length;
    }

    function appendRunLine(line, kind) {
        runModel.append({ line: line, kind: kind });
        if (runModel.count > 2000) {
            runModel.remove(0);
        }
    }

    property string terminalText: ""

    function appendTerminalData(data) {
        terminalText += data;
        if (terminalText.length > 100000) {
            terminalText = terminalText.substring(terminalText.length - 80000);
        }
    }

    function openTerminalPanel() {
        if (coreClient.workspaceRoot === "") {
            return;
        }
        showBottomPanel = true;
        bottomTab = "terminal";
        if (!coreClient.terminalActive) {
            coreClient.terminalOpen();
        }
        shellInput.forceActiveFocus();
    }

    function submitShellInput() {
        if (!coreClient.terminalActive) {
            coreClient.terminalOpen();
            return;
        }
        coreClient.runInput(shellInput.text + "\n");
        shellInput.text = "";
    }

    function startRun(command) {
        if (coreClient.workspaceRoot === "" || coreClient.running) {
            return;
        }
        showBottomPanel = true;
        bottomTab = "run";
        coreClient.runStart(command);
    }

    function stopRun() {
        if (coreClient.running) {
            coreClient.runStop();
        }
    }

    function submitRunInput() {
        const text = runInput.text;
        if (text === "") {
            return;
        }
        runInput.text = "";
        if (coreClient.running) {
            appendRunLine("> " + text, "stdin");
            coreClient.runStdin(text + "\n");
            return;
        }
        startRun(text);
    }

    function openSearchPanel() {
        if (coreClient.workspaceRoot === "") {
            return;
        }
        showBottomPanel = true;
        bottomTab = "search";
        searchInput.forceActiveFocus();
        searchInput.selectAll();
    }

    function runSearch() {
        const query = searchInput.text;
        if (query === "" || coreClient.workspaceRoot === "" || searching) {
            return;
        }
        searching = true;
        searchModel.clear();
        searchTruncated = false;
        coreClient.searchInFiles(query, searchCaseSensitive);
    }

    function openSearchEverywhere() {
        searchEverywhereVisible = true;
        searchEverywhereLoading = false;
        searchEverywhereTruncated = false;
        searchEverywhereIndex = 0;
        searchEverywhereError = "";
        everywhereModel.clear();
        searchEverywhereInput.text = "";
        if (commandList.length === 0) {
            coreClient.listCommands();
        }
        root.appendSearchEverywhereCommands("");
        searchEverywhereInput.forceActiveFocus();
    }

    function runSearchEverywhere() {
        const query = searchEverywhereInput.text.trim();
        searchEverywhereError = "";
        everywhereModel.clear();
        searchEverywhereIndex = 0;
        root.appendSearchEverywhereCommands(query);
        if (query === "" || coreClient.workspaceRoot === "") {
            searchEverywhereLoading = false;
            searchEverywhereTruncated = false;
            return;
        }
        searchEverywhereLoading = true;
        coreClient.findFiles(query);
    }

    function appendSearchEverywhereCommands(query) {
        const needle = query.toLowerCase();
        for (let i = 0; i < commandList.length; i++) {
            const command = commandList[i];
            if (command.requiresWorkspace === true && coreClient.workspaceRoot === "") {
                continue;
            }
            const id = command.id !== undefined ? command.id : "";
            const title = command.title !== undefined ? command.title : id;
            const category = command.category !== undefined ? command.category : qsTr("Comando");
            const description = command.description !== undefined ? command.description : "";
            const shortcut = command.defaultShortcut !== undefined
                    ? command.defaultShortcut : "";
            const haystack = (id + " " + title + " " + category + " "
                              + description + " " + shortcut).toLowerCase();
            if (needle !== "" && haystack.indexOf(needle) < 0) {
                continue;
            }
            everywhereModel.append({
                kind: "command",
                title: title,
                path: id,
                subtitle: shortcut !== "" ? category + " · " + shortcut : category,
                commandId: id
            });
        }
    }

    function acceptSearchEverywhere() {
        if (!searchEverywhereVisible || searchEverywhereIndex < 0
                || searchEverywhereIndex >= everywhereModel.count) {
            return;
        }
        const item = everywhereModel.get(searchEverywhereIndex);
        searchEverywhereVisible = false;
        if (item.kind === "command") {
            root.executeSearchCommand(item.commandId);
            return;
        }
        coreClient.readFile(coreClient.workspaceRoot + "/" + item.path);
    }

    function executeSearchCommand(commandId) {
        if (commandId === "workspace.open") {
            root.requestOpenFolder();
        } else if (commandId === "workspace.close") {
            coreClient.closeWorkspace();
        } else if (commandId === "tools.detect" || commandId === "tools.status") {
            root.toggleBottomTab("tools");
            coreClient.detectTools();
        } else if (commandId === "build.run") {
            root.startBuild();
        } else if (commandId === "fs.search") {
            root.openSearchPanel();
        } else if (commandId === "fs.findFiles" || commandId === "command.list") {
            root.openSearchEverywhere();
        } else if (commandId === "run.start") {
            root.startRun("");
        } else if (commandId === "run.stop") {
            root.stopRun();
        } else if (commandId === "terminal.open") {
            root.openTerminalPanel();
        } else if (commandId === "lsp.definition") {
            root.requestDefinition();
        } else if (commandId === "lsp.hover") {
            root.requestHover();
        } else if (commandId === "lsp.completion") {
            root.requestCompletion();
        } else if (commandId === "lsp.references") {
            root.requestUsages();
        } else if (commandId === "lsp.rename") {
            root.openRenameDialog();
        } else if (commandId === "fs.createFile") {
            root.openCreateDialog("file");
        } else if (commandId === "fs.createDirectory") {
            root.openCreateDialog("directory");
        }
    }

    function requestOpenFolder() {
        folderPicker.open(coreClient.workspaceRoot !== ""
                          ? coreClient.workspaceRoot : coreClient.homeDir);
    }

    function clearWorkspaceUiState() {
        treeModel.clear();
        openFiles.clear();
        buildOutputModel.clear();
        removeProblemsBySource("build");
        removeProblemsBySource("lsp");
        removeProblemsBySource("quality");
        pendingJumpPath = "";
        pendingJumpLine = 0;
        pendingJumpColumn = 0;
        hoverText = "";
        hoverVisible = false;
        completionVisible = false;
        completionPrefixStart = -1;
        completionAll = [];
        completionModel.clear();
        usagesVisible = false;
        usagesModel.clear();
        renameDialogVisible = false;
        renameError = "";
        createDialogVisible = false;
        createDialogError = "";
        createDialogParentPath = "";
        entryMenuVisible = false;
        entryMenuPath = "";
        entryRenameVisible = false;
        entryRenameError = "";
        entryDeleteVisible = false;
        entryDeleteError = "";
        explorerSelectedPath = "";
        explorerSelectedKind = "";
        searchModel.clear();
        searchInput.text = "";
        searchTruncated = false;
        searching = false;
        runModel.clear();
        runInput.text = "";
        testModel.clear();
        testSummary = "";
        terminalText = "";
        shellInput.text = "";
        pendingReloads = {};
        currentTab = -1;
        loadingEditorText = true;
        editor.text = "";
        loadingEditorText = false;
        editorHighlighter.filePath = "";
    }

    ListModel {
        id: treeModel
    }

    ListModel {
        id: completionModel
    }

    ListModel {
        id: usagesModel
    }

    ListModel {
        id: openFiles
    }

    ListModel {
        id: assistantModel
    }

    ListModel {
        id: buildOutputModel
    }

    ListModel {
        id: problemsModel
    }

    ListModel {
        id: searchModel
    }

    ListModel {
        id: everywhereModel
    }

    ListModel {
        id: runModel
    }

    ListModel {
        id: testModel
    }

    CoreClient {
        id: coreClient
    }

    FolderPickerDialog {
        id: folderPicker

        anchors.fill: parent
        homePath: coreClient.homeDir
        onBrowseRequested: function(path) {
            coreClient.browseWorkspaceFolders(path);
        }
        onOpenRequested: function(path) {
            coreClient.openWorkspace(path);
        }
        onCreateFolderRequested: function(parent, name) {
            coreClient.createWorkspaceFolder(parent, name);
        }
        onCreateProjectRequested: function(parent, name, templateId) {
            coreClient.createWorkspaceProject(parent, name, templateId);
        }
    }

    Component.onCompleted: {
        coreClient.start();
        assistantModel.append({
            role: "system",
            body: qsTr("Assistente KW ainda esta offline. Os providers de IA "
                       + "(Ollama local, GPT CLI, Claude CLI) chegam na Fase 7 do "
                       + "roadmap. Politica: nenhum codigo sai da maquina sem a sua "
                       + "confirmacao explicita.")
        });
    }

    Connections {
        target: coreClient

        function onStatusChanged() {
            if (coreClient.connected && root.commandList.length === 0) {
                coreClient.listCommands();
            }
        }

        function onDirListed(path, entries) {
            if (path === coreClient.workspaceRoot) {
                treeModel.clear();
                root.insertEntries(0, entries, 0, path);
                return;
            }
            const index = root.rowIndexForPath(path);
            if (index < 0) {
                return;
            }
            if (treeModel.get(index).expanded) {
                root.collapseRow(index);
            }
            treeModel.setProperty(index, "expanded", true);
            root.insertEntries(index + 1, entries, treeModel.get(index).depth + 1, path);
        }

        function onFileLoaded(path, content) {
            // Recarga silenciosa pos-rename: atualiza a aba sem trocar de
            // arquivo nem perder a posicao do cursor.
            if (root.pendingReloads[path] === true) {
                const reloads = root.pendingReloads;
                delete reloads[path];
                root.pendingReloads = reloads;
                for (let i = 0; i < openFiles.count; i++) {
                    if (openFiles.get(i).path === path) {
                        openFiles.setProperty(i, "content", content);
                        openFiles.setProperty(i, "savedContent", content);
                        openFiles.setProperty(i, "modified", false);
                        if (i === root.currentTab) {
                            const cursor = Math.min(editor.cursorPosition,
                                                    content.length);
                            root.loadingEditorText = true;
                            editor.text = content;
                            root.loadingEditorText = false;
                            editor.cursorPosition = cursor;
                        }
                        return;
                    }
                }
                return;
            }
            for (let i = 0; i < openFiles.count; i++) {
                if (openFiles.get(i).path === path) {
                    openFiles.setProperty(i, "content", content);
                    openFiles.setProperty(i, "savedContent", content);
                    openFiles.setProperty(i, "modified", false);
                    root.selectTab(i);
                    root.loadingEditorText = true;
                    editor.text = content;
                    root.loadingEditorText = false;
                    editorHighlighter.filePath = path;
                    if (root.pendingJumpPath === path) {
                        root.jumpToPending();
                    }
                    root.refreshSemanticTokens();
                    return;
                }
            }
            openFiles.append({
                path: path,
                name: root.baseName(path),
                content: content,
                savedContent: content,
                modified: false
            });
            root.selectTab(openFiles.count - 1);
            if (root.pendingJumpPath === path) {
                root.jumpToPending();
            }
            root.refreshSemanticTokens();
        }

        function onFileSaved(path) {
            for (let i = 0; i < openFiles.count; i++) {
                if (openFiles.get(i).path === path) {
                    openFiles.setProperty(i, "savedContent", openFiles.get(i).content);
                    openFiles.setProperty(i, "modified", false);
                }
            }
        }

        function onFileCreated(path) {
            root.createDialogVisible = false;
            root.createDialogError = "";
            root.explorerSelectedPath = path;
            root.explorerSelectedKind = "file";
            coreClient.listDir(root.parentDir(path));
            coreClient.readFile(path);
        }

        function onDirectoryCreated(path) {
            root.createDialogVisible = false;
            root.createDialogError = "";
            root.explorerSelectedPath = path;
            root.explorerSelectedKind = "directory";
            coreClient.listDir(root.parentDir(path));
        }

        function onPathRenamed(from, to) {
            root.entryRenameVisible = false;
            root.entryRenameError = "";
            root.applyPathRenameToTabs(from, to);
            root.explorerSelectedPath = to;
            coreClient.listDir(root.parentDir(to));
            editor.forceActiveFocus();
        }

        function onPathDeleted(path) {
            root.entryDeleteVisible = false;
            root.entryDeleteError = "";
            root.closeTabsUnderPath(path);
            if (root.explorerSelectedPath === path
                    || root.explorerSelectedPath.indexOf(path + "/") === 0) {
                root.explorerSelectedPath = "";
                root.explorerSelectedKind = "";
            }
            coreClient.listDir(root.parentDir(path));
            editor.forceActiveFocus();
        }

        function onWorkspaceBrowseListed(path, parent, entries) {
            folderPicker.setListing(path, parent, entries);
        }

        function onWorkspaceFolderCreated(path) {
            folderPicker.selectAfterRefresh(path);
            coreClient.browseWorkspaceFolders(folderPicker.currentPath);
        }

        function onWorkspaceChanged() {
            folderPicker.close();
            const newRoot = coreClient.workspaceRoot;
            if (newRoot === "") {
                root.clearWorkspaceUiState();
                root.activeWorkspaceRoot = "";
                return;
            }
            if (root.activeWorkspaceRoot !== "" && root.activeWorkspaceRoot !== newRoot) {
                root.clearWorkspaceUiState();
            }
            root.activeWorkspaceRoot = newRoot;
        }

        function onToolsListed(tools) {
            root.toolsList = tools;
        }

        function onBuildStarted(command) {
            root.appendBuildLine("$ " + command);
        }

        function onBuildOutput(line) {
            root.appendBuildLine(line);
        }

        function onBuildDiagnostic(diagnostic) {
            problemsModel.append({
                severity: diagnostic.severity !== undefined
                          ? diagnostic.severity : "error",
                message: diagnostic.message !== undefined ? diagnostic.message : "",
                file: diagnostic.file !== undefined ? diagnostic.file : "",
                line: diagnostic.line !== undefined ? Number(diagnostic.line) : 0,
                column: diagnostic.column !== undefined ? Number(diagnostic.column) : 0,
                source: "build"
            });
        }

        function onLspDiagnostics(path, diagnostics) {
            const rel = root.relativeToRoot(path);
            for (let i = problemsModel.count - 1; i >= 0; i--) {
                const row = problemsModel.get(i);
                if (row.source === "lsp" && row.file === rel) {
                    problemsModel.remove(i);
                }
            }
            for (let j = 0; j < diagnostics.length; j++) {
                const d = diagnostics[j];
                problemsModel.append({
                    severity: d.severity !== undefined ? d.severity : "error",
                    message: d.message !== undefined ? d.message : "",
                    file: rel,
                    line: d.line !== undefined ? Number(d.line) : 0,
                    column: d.column !== undefined ? Number(d.column) : 0,
                    source: "lsp"
                });
            }
        }

        function onLspDefinitionResolved(path, line, column) {
            root.openDiagnostic(path, line, column);
        }

        function onLspHoverResolved(content) {
            const text = content.trim();
            if (text === "") {
                root.hoverVisible = false;
                root.hoverText = "";
                return;
            }
            root.hoverText = text;
            root.hoverVisible = true;
            hoverHideTimer.restart();
        }

        function onLspSemanticTokensResolved(tokens) {
            editorHighlighter.setSemanticTokens(tokens);
        }

        function onLspCompletionResolved(items) {
            root.completionAll = items;
            root.refilterCompletions();
        }

        function onLspReferencesResolved(references) {
            usagesModel.clear();
            for (let i = 0; i < references.length; i++) {
                const usage = references[i];
                const line = usage.line !== undefined ? Number(usage.line) : 1;
                usagesModel.append({
                    path: usage.path,
                    line: line,
                    column: usage.column !== undefined ? Number(usage.column) : 1,
                    display: root.relativeToRoot(usage.path) + ":" + line
                });
            }
            root.usagesVisible = usagesModel.count > 0;
        }

        function onLspRenameApplied(files, edits) {
            const reloads = {};
            for (let i = 0; i < openFiles.count; i++) {
                const path = openFiles.get(i).path;
                if (files.indexOf(path) >= 0) {
                    reloads[path] = true;
                }
            }
            root.pendingReloads = reloads;
            for (const path in reloads) {
                coreClient.readFile(path);
            }
        }

        function onSearchResults(matches, truncated) {
            root.searching = false;
            root.searchTruncated = truncated;
            searchModel.clear();
            for (let i = 0; i < matches.length; i++) {
                const match = matches[i];
                searchModel.append({
                    path: match.path,
                    line: match.line !== undefined ? Number(match.line) : 1,
                    column: match.column !== undefined ? Number(match.column) : 1,
                    preview: match.preview !== undefined ? match.preview : ""
                });
            }
        }

        function onFileSearchResults(matches, truncated) {
            root.searchEverywhereLoading = false;
            root.searchEverywhereTruncated = truncated;
            for (let i = everywhereModel.count - 1; i >= 0; i--) {
                if (everywhereModel.get(i).kind === "file") {
                    everywhereModel.remove(i);
                }
            }
            for (let i = 0; i < matches.length; i++) {
                const match = matches[i];
                everywhereModel.append({
                    kind: "file",
                    title: match.name !== undefined ? match.name : root.baseName(match.path),
                    path: match.path !== undefined ? match.path : "",
                    subtitle: match.path !== undefined ? match.path : "",
                    commandId: ""
                });
            }
            root.searchEverywhereIndex = everywhereModel.count > 0 ? 0 : -1;
        }

        function onCommandsListed(commands) {
            root.commandList = commands;
            if (root.searchEverywhereVisible) {
                root.runSearchEverywhere();
            }
        }

        function onTerminalData(data) {
            root.appendTerminalData(data);
        }

        function onTerminalClosed(exitCode) {
            root.appendTerminalData(qsTr("\n== sessao encerrada — Enter para reabrir ==\n"));
        }

        function onRunStarted(command) {
            root.appendRunLine("$ " + command, "command");
        }

        function onRunOutput(line, stream) {
            root.appendRunLine(line, stream);
        }

        function onRunFinished(success, exitCode) {
            root.appendRunLine(success
                                    ? qsTr("== processo finalizado (codigo %1) ==").arg(exitCode)
                                    : qsTr("== processo encerrou com falha (codigo %1) ==")
                                          .arg(exitCode),
                                    success ? "info" : "stderr");
        }

        function onBuildFinished(success, exitCode, diagnostics) {
            root.appendBuildLine(success
                                 ? qsTr("== build concluido com sucesso ==")
                                 : qsTr("== build falhou (codigo %1) ==").arg(exitCode));
            if (!success && problemsModel.count > 0) {
                root.bottomTab = "problems";
            }
        }

        function onTestCase(name, status) {
            testModel.append({ name: name, status: status });
            while (testModel.count > 5000) {
                testModel.remove(0);
            }
        }

        function onQualityDiagnostic(diagnostic) {
            problemsModel.append({
                severity: diagnostic.severity !== undefined
                          ? diagnostic.severity : "warning",
                message: diagnostic.message !== undefined ? diagnostic.message : "",
                file: diagnostic.file !== undefined ? diagnostic.file : "",
                line: diagnostic.line !== undefined ? Number(diagnostic.line) : 0,
                column: diagnostic.column !== undefined ? Number(diagnostic.column) : 0,
                source: "quality"
            });
        }

        function onQualityFinished(success, exitCode, diagnostics) {
            root.bottomTab = "problems";
        }

        function onTestFinished(success, passed, failed, ignored) {
            root.testSummary = (success
                                ? qsTr("passou: %1")
                                : qsTr("FALHOU — passou: %1")).arg(passed)
                    + qsTr("  falhou: %1  ignorado: %2").arg(failed).arg(ignored);
        }

        function onRequestFailed(method, message) {
            if (method === "workspace.open" || method === "workspace.browse"
                    || method === "workspace.createFolder"
                    || method === "workspace.createProject") {
                folderPicker.showError(message);
            }
            if (method === "lsp.hover") {
                root.hoverVisible = false;
            }
            if (method === "lsp.completion") {
                root.completionVisible = false;
            }
            if (method === "fs.search") {
                root.searching = false;
            }
            if (method === "fs.findFiles") {
                root.searchEverywhereLoading = false;
                root.searchEverywhereError = message;
                root.searchEverywhereVisible = true;
            }
            if (method === "fs.createFile" || method === "fs.createDirectory") {
                root.createDialogError = message;
                root.createDialogVisible = true;
            }
            if (method === "fs.rename") {
                root.entryRenameError = message;
                root.entryRenameVisible = true;
            }
            if (method === "fs.delete") {
                root.entryDeleteError = message;
                root.entryDeleteVisible = true;
            }
            if (method === "run.start" || method === "run.stdin"
                    || method === "run.stop") {
                root.appendRunLine(message, "stderr");
            }
            if (method === "lsp.rename") {
                root.renameError = message;
                root.renameDialogVisible = true;
            }
        }
    }

    Shortcut {
        sequence: StandardKey.Save
        onActivated: root.saveCurrentFile()
    }

    Shortcut {
        sequence: "Ctrl+F9"
        onActivated: root.startBuild()
    }

    Shortcut {
        sequence: "Ctrl+Shift+F9"
        onActivated: root.startTests()
    }

    Shortcut {
        sequence: "Ctrl+Shift+L"
        onActivated: root.startQuality()
    }

    Shortcut {
        sequence: "Ctrl+B"
        onActivated: root.requestDefinition()
    }

    Shortcut {
        sequence: "Ctrl+Q"
        onActivated: root.requestHover()
    }

    Shortcut {
        sequence: "Ctrl+Space"
        onActivated: root.requestCompletion()
    }

    Shortcut {
        sequence: "Shift+F6"
        onActivated: root.openRenameDialog()
    }

    Shortcut {
        sequence: "Alt+F7"
        onActivated: root.requestUsages()
    }

    Shortcut {
        sequence: "Ctrl+Shift+F"
        onActivated: root.openSearchPanel()
    }

    Shortcut {
        sequence: "Ctrl+Shift+N"
        onActivated: root.openSearchEverywhere()
    }

    Shortcut {
        sequence: "Ctrl+Shift+A"
        onActivated: root.openSearchEverywhere()
    }

    Shortcut {
        sequence: "Shift+F10"
        onActivated: root.startRun("")
    }

    Shortcut {
        sequence: "Ctrl+F2"
        onActivated: root.stopRun()
    }

    Shortcut {
        sequence: "Alt+F12"
        onActivated: root.openTerminalPanel()
    }

    // Auto-completion: dispara sozinho ~250ms depois de digitar parte de um
    // identificador ou um acesso de membro (. ou :), estilo JetBrains.
    Timer {
        id: completionDebounce

        interval: 250
        repeat: false
        onTriggered: {
            if (root.currentTab < 0 || !editor.activeFocus
                    || root.completionVisible) {
                return;
            }
            const position = editor.cursorPosition;
            if (position <= 0) {
                return;
            }
            const previous = editor.text.charAt(position - 1);
            if (root.isWordChar(previous) || previous === "."
                    || previous === ":") {
                root.requestCompletion();
            }
        }
    }

    // Debounce: envia o buffer ao LSP ~meio segundo depois da ultima tecla.
    Timer {
        id: changeDebounce

        interval: 600
        repeat: false
        onTriggered: {
            if (root.currentTab >= 0 && root.currentTab < openFiles.count) {
                coreClient.notifyFileChanged(openFiles.get(root.currentTab).path,
                                             editor.text);
                root.refreshSemanticTokens();
            }
        }
    }

    Timer {
        id: hoverHideTimer

        interval: 9000
        repeat: false
        onTriggered: root.hoverVisible = false
    }

    Timer {
        id: searchEverywhereDebounce

        interval: 180
        repeat: false
        onTriggered: root.runSearchEverywhere()
    }

    Rectangle {
        id: searchEverywhereDialog

        visible: root.searchEverywhereVisible
        z: 90
        anchors.centerIn: parent
        width: Math.min(620, root.width - 80)
        height: Math.min(420, root.height - 120)
        radius: Theme.radius
        color: Theme.background2
        border.color: Theme.accent
        border.width: 1

        Column {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium
            spacing: Theme.spacingSmall

            Text {
                text: qsTr("Search Everywhere")
                color: Theme.textPrimary
                font.pixelSize: 13
                font.bold: true
            }

            Rectangle {
                width: parent.width
                height: 34
                radius: Theme.radius
                color: Theme.background0
                border.color: searchEverywhereInput.activeFocus
                              ? Theme.accent : Theme.borderSoft
                border.width: 1

                TextInput {
                    id: searchEverywhereInput

                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    selectedTextColor: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 13
                    clip: true
                    selectByMouse: true
                    onTextChanged: searchEverywhereDebounce.restart()
                    onAccepted: root.acceptSearchEverywhere()
                    Keys.onEscapePressed: {
                        root.searchEverywhereVisible = false;
                        editor.forceActiveFocus();
                    }
                    Keys.onDownPressed: {
                        root.searchEverywhereIndex = Math.min(
                                    root.searchEverywhereIndex + 1,
                                    everywhereModel.count - 1);
                    }
                    Keys.onUpPressed: {
                        root.searchEverywhereIndex = Math.max(
                                    root.searchEverywhereIndex - 1, 0);
                    }
                }
            }

            Text {
                width: parent.width
                visible: root.searchEverywhereError !== ""
                text: root.searchEverywhereError
                color: Theme.errorSoft
                font.pixelSize: 10
                wrapMode: Text.WordWrap
            }

            Text {
                width: parent.width
                text: {
                    if (root.searchEverywhereLoading) {
                        return qsTr("Buscando arquivos...");
                    }
                    if (root.searchEverywhereTruncated) {
                        return qsTr("Mostrando os primeiros resultados.");
                    }
                    if (searchEverywhereInput.text.trim() === "") {
                        return qsTr("Digite para buscar arquivos por nome.");
                    }
                    return qsTr("%1 arquivos").arg(everywhereModel.count);
                }
                color: Theme.textMuted
                font.pixelSize: 10
            }

            ListView {
                id: searchEverywhereList

                width: parent.width
                height: parent.height - y
                clip: true
                model: everywhereModel
                currentIndex: root.searchEverywhereIndex

                delegate: Rectangle {
                    required property int index
                    required property string path
                    required property string title
                    required property string subtitle
                    required property string kind

                    width: searchEverywhereList.width
                    height: 38
                    radius: Theme.radius
                    color: index === root.searchEverywhereIndex
                           ? Theme.accentDim
                           : (searchEverywhereArea.containsMouse
                              ? Theme.surface2 : "transparent")

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.rightMargin: Theme.spacingSmall
                        spacing: 2

                        Text {
                            width: parent.width
                            text: title
                            color: Theme.textPrimary
                            font.pixelSize: 12
                            font.bold: true
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            text: subtitle !== "" ? subtitle : path
                            color: Theme.textMuted
                            font.family: Theme.monoFont
                            font.pixelSize: 10
                            elide: Text.ElideMiddle
                        }
                    }

                    MouseArea {
                        id: searchEverywhereArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onEntered: root.searchEverywhereIndex = parent.index
                        onClicked: {
                            root.searchEverywhereIndex = parent.index;
                            root.acceptSearchEverywhere();
                        }
                    }
                }
            }
        }
    }

    // ── Header ────────────────────────────────────────────────────────────
    Rectangle {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 44
        color: Theme.background1

        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingMedium
            spacing: Theme.spacingMedium

            Image {
                width: 26
                height: 26
                anchors.verticalCenter: parent.verticalCenter
                source: "qrc:/KineinVectis/assets/app-icon.png"
                fillMode: Image.PreserveAspectFit
                smooth: true
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Kinein Vectis")
                color: Theme.textPrimary
                font.pixelSize: 15
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: openFolderText.width + 2 * Theme.spacingMedium
                height: 28
                radius: Theme.radius
                color: openFolderArea.containsMouse ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: openFolderText

                    anchors.centerIn: parent
                    text: qsTr("Abrir pasta...")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                }

                MouseArea {
                    id: openFolderArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.requestOpenFolder()
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: buildButtonText.width + 2 * Theme.spacingMedium
                height: 28
                radius: Theme.radius
                visible: coreClient.workspaceRoot !== ""
                enabled: !coreClient.building && coreClient.connected
                opacity: enabled ? 1.0 : 0.6
                color: coreClient.building
                       ? Theme.surface1
                       : (buildArea.pressed ? Theme.accentDim : Theme.accent)

                Text {
                    id: buildButtonText

                    anchors.centerIn: parent
                    text: coreClient.building ? qsTr("Compilando...") : qsTr("Compilar")
                    color: coreClient.building ? Theme.textSecondary : Theme.background0
                    font.pixelSize: 12
                    font.bold: true
                }

                MouseArea {
                    id: buildArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.startBuild()
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: testButtonText.width + 2 * Theme.spacingMedium
                height: 28
                radius: Theme.radius
                visible: coreClient.workspaceRoot !== ""
                enabled: !coreClient.testing && coreClient.connected
                opacity: enabled ? 1.0 : 0.6
                color: testArea.pressed ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: testButtonText

                    anchors.centerIn: parent
                    text: coreClient.testing ? qsTr("Testando...") : qsTr("Testes")
                    color: coreClient.testing ? Theme.textSecondary : Theme.textPrimary
                    font.pixelSize: 12
                    font.bold: true
                }

                MouseArea {
                    id: testArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.startTests()
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: qualityButtonText.width + 2 * Theme.spacingMedium
                height: 28
                radius: Theme.radius
                visible: coreClient.workspaceRoot !== ""
                enabled: !coreClient.analyzing && coreClient.connected
                opacity: enabled ? 1.0 : 0.6
                color: qualityArea.pressed ? Theme.surface2 : Theme.surface1
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: qualityButtonText

                    anchors.centerIn: parent
                    text: coreClient.analyzing ? qsTr("Analisando...") : qsTr("Análise")
                    color: coreClient.analyzing ? Theme.textSecondary : Theme.textPrimary
                    font.pixelSize: 12
                    font.bold: true
                }

                MouseArea {
                    id: qualityArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.startQuality()
                }
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: runButtonText.width + 2 * Theme.spacingMedium
                height: 28
                radius: Theme.radius
                visible: coreClient.workspaceRoot !== ""
                enabled: coreClient.connected
                opacity: enabled ? 1.0 : 0.6
                color: coreClient.running
                       ? (runArea.pressed ? Theme.surface1 : Theme.surface2)
                       : (runArea.pressed ? Theme.accentDim : Theme.accent)

                Text {
                    id: runButtonText

                    anchors.centerIn: parent
                    text: coreClient.running ? qsTr("■ Parar") : qsTr("▶ Iniciar")
                    color: coreClient.running ? Theme.textPrimary : Theme.background0
                    font.pixelSize: 12
                    font.bold: true
                }

                MouseArea {
                    id: runArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: coreClient.running ? root.stopRun() : root.startRun("")
                }
            }
        }
    }

    // ── Conteudo em ilhas ─────────────────────────────────────────────────
    Item {
        id: content

        anchors.top: header.bottom
        anchors.bottom: statusBar.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.panelGap

        Row {
            anchors.fill: parent
            spacing: Theme.panelGap

            // Ilha: sidebar de icones (tool windows)
            Rectangle {
                id: sideBar

                width: 42
                height: parent.height
                radius: Theme.radiusLarge
                color: Theme.background1
                border.color: Theme.borderSoft
                border.width: 1

                Column {
                    anchors.top: parent.top
                    anchors.topMargin: Theme.spacingSmall
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: Theme.spacingSmall

                    SideButton {
                        glyph: "▤"
                        tip: qsTr("Projeto")
                        active: root.showExplorer && coreClient.workspaceRoot !== ""
                        enabled: coreClient.workspaceRoot !== ""
                        opacity: enabled ? 1.0 : 0.4
                        onActivated: root.showExplorer = !root.showExplorer
                    }

                    SideButton {
                        glyph: "⚒"
                        tip: qsTr("Ferramentas")
                        active: root.showBottomPanel && root.bottomTab === "tools"
                        onActivated: root.toggleBottomTab("tools")
                    }

                    SideButton {
                        glyph: "≣"
                        tip: qsTr("Log da IDE")
                        active: root.showBottomPanel && root.bottomTab === "logs"
                        onActivated: root.toggleBottomTab("logs")
                    }

                    SideButton {
                        glyph: "✦"
                        tip: qsTr("Assistente KW")
                        active: root.showAssistant
                        onActivated: root.showAssistant = !root.showAssistant
                    }
                }
            }

            // Ilha: explorer em arvore
            Rectangle {
                id: explorerPanel

                width: visible ? 260 : 0
                height: parent.height
                visible: coreClient.workspaceRoot !== "" && root.showExplorer
                radius: Theme.radiusLarge
                color: Theme.background1
                border.color: Theme.borderSoft
                border.width: 1

                Column {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    Row {
                        width: parent.width
                        spacing: Theme.spacingSmall

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: coreClient.workspaceName
                            color: Theme.textPrimary
                            font.pixelSize: 13
                            font.bold: true
                        }

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            width: kindText.width + 10
                            height: 16
                            radius: 8
                            color: Theme.accentDim

                            Text {
                                id: kindText

                                anchors.centerIn: parent
                                text: root.kindLabel(coreClient.workspaceKind)
                                color: Theme.accent
                                font.pixelSize: 9
                                font.bold: true
                            }
                        }

                        Item {
                            width: parent.width - x - refreshChip.width
                                   - newFileChip.width - newFolderChip.width
                                   - closeProjectChip.width - 3 * Theme.spacingSmall
                            height: 1
                        }

                        Rectangle {
                            id: newFileChip

                            anchors.verticalCenter: parent.verticalCenter
                            width: 22
                            height: 22
                            radius: Theme.radius
                            color: newFileArea.containsMouse ? Theme.surface2 : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "+"
                                color: Theme.textSecondary
                                font.pixelSize: 15
                                font.bold: true
                            }

                            MouseArea {
                                id: newFileArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.openCreateDialog("file")
                            }
                        }

                        Rectangle {
                            id: newFolderChip

                            anchors.verticalCenter: parent.verticalCenter
                            width: 22
                            height: 22
                            radius: Theme.radius
                            color: newFolderArea.containsMouse ? Theme.surface2 : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "▣"
                                color: Theme.textSecondary
                                font.pixelSize: 12
                            }

                            MouseArea {
                                id: newFolderArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.openCreateDialog("directory")
                            }
                        }

                        Rectangle {
                            id: refreshChip

                            anchors.verticalCenter: parent.verticalCenter
                            width: 22
                            height: 22
                            radius: Theme.radius
                            color: refreshArea.containsMouse ? Theme.surface2 : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "⟳"
                                color: Theme.textSecondary
                                font.pixelSize: 13
                            }

                            MouseArea {
                                id: refreshArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: coreClient.listDir(coreClient.workspaceRoot)
                            }
                        }

                        Rectangle {
                            id: closeProjectChip

                            anchors.verticalCenter: parent.verticalCenter
                            width: 22
                            height: 22
                            radius: Theme.radius
                            color: closeProjectArea.containsMouse
                                   ? Theme.surface2 : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "x"
                                color: closeProjectArea.containsMouse
                                       ? Theme.errorSoft : Theme.textSecondary
                                font.pixelSize: 13
                            }

                            MouseArea {
                                id: closeProjectArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: coreClient.closeWorkspace()
                            }
                        }
                    }

                    ListView {
                        id: explorerView

                        width: parent.width
                        height: parent.height - y
                        clip: true
                        model: treeModel

                        delegate: Rectangle {
                            id: treeRow

                            required property int index
                            required property string path
                            required property string name
                            required property string kind
                            required property int depth
                            required property bool expanded

                            width: explorerView.width
                            height: 24
                            radius: Theme.radius
                            color: treeRow.path === root.explorerSelectedPath
                                   ? Theme.accentDim
                                   : (entryArea.containsMouse ? Theme.surface2 : "transparent")

                            Row {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall + treeRow.depth * 14
                                spacing: Theme.spacingSmall

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 12
                                    text: treeRow.kind === "directory"
                                          ? (treeRow.expanded ? "▾" : "▸") : "·"
                                    color: treeRow.kind === "directory"
                                           ? Theme.accent : Theme.textMuted
                                    font.pixelSize: 12
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: treeRow.name
                                    color: treeRow.kind === "directory"
                                           ? Theme.textPrimary : Theme.textSecondary
                                    font.pixelSize: 12
                                }
                            }

                            MouseArea {
                                id: entryArea

                                anchors.fill: parent
                                hoverEnabled: true
                                acceptedButtons: Qt.LeftButton | Qt.RightButton
                                cursorShape: Qt.PointingHandCursor
                                onClicked: function(mouse) {
                                    root.explorerSelectedPath = treeRow.path;
                                    root.explorerSelectedKind = treeRow.kind;
                                    if (mouse.button === Qt.RightButton) {
                                        const pt = entryArea.mapToItem(null, mouse.x,
                                                                       mouse.y);
                                        root.openEntryMenu(treeRow.path, treeRow.kind,
                                                           treeRow.name, pt.x, pt.y);
                                        return;
                                    }
                                    if (treeRow.kind === "directory") {
                                        if (treeRow.expanded) {
                                            root.collapseRow(treeRow.index);
                                        } else {
                                            coreClient.listDir(treeRow.path);
                                        }
                                    } else if (treeRow.kind === "file") {
                                        coreClient.readFile(treeRow.path);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Coluna: editor + painel inferior
            Column {
                width: parent.width - sideBar.width - Theme.panelGap
                       - (explorerPanel.visible
                          ? explorerPanel.width + Theme.panelGap : 0)
                       - (assistantPanel.visible
                          ? assistantPanel.width + Theme.panelGap : 0)
                height: parent.height
                spacing: Theme.panelGap

                // Ilha: editor com abas
                Rectangle {
                    id: editorPanel

                    width: parent.width
                    height: parent.height - (bottomPanel.visible
                            ? bottomPanel.height + Theme.panelGap : 0)
                    radius: Theme.radiusLarge
                    color: Theme.background1
                    border.color: Theme.borderSoft
                    border.width: 1

                    Item {
                        id: tabBar

                        anchors.top: parent.top
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        height: openFiles.count > 0 ? 30 : 0
                        visible: openFiles.count > 0

                        Row {
                            anchors.left: parent.left
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: Theme.spacingSmall

                            Repeater {
                                model: openFiles

                                delegate: Rectangle {
                                    required property int index
                                    required property string name
                                    required property bool modified

                                    width: tabLabel.width + closeLabel.width
                                           + 3 * Theme.spacingSmall
                                    height: 26
                                    radius: Theme.radius
                                    color: index === root.currentTab
                                           ? Theme.surface2 : Theme.surface1
                                    border.color: index === root.currentTab
                                                  ? Theme.accent : Theme.borderSoft
                                    border.width: 1

                                    Text {
                                        id: tabLabel

                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.left: parent.left
                                        anchors.leftMargin: Theme.spacingSmall
                                        text: (modified ? "● " : "") + name
                                        color: index === root.currentTab
                                               ? Theme.textPrimary : Theme.textSecondary
                                        font.pixelSize: 12
                                    }

                                    Text {
                                        id: closeLabel

                                        anchors.verticalCenter: parent.verticalCenter
                                        anchors.right: parent.right
                                        anchors.rightMargin: Theme.spacingSmall
                                        text: "×"
                                        color: closeArea.containsMouse
                                               ? Theme.errorSoft : Theme.textMuted
                                        font.pixelSize: 14

                                        MouseArea {
                                            id: closeArea

                                            anchors.fill: parent
                                            hoverEnabled: true
                                            cursorShape: Qt.PointingHandCursor
                                            onClicked: root.closeTab(index)
                                        }
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        anchors.rightMargin: closeLabel.width
                                                             + Theme.spacingSmall
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.selectTab(index)
                                    }
                                }
                            }
                        }

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.right: parent.right
                            width: saveText.width + 2 * Theme.spacingMedium
                            height: 24
                            radius: Theme.radius
                            visible: root.currentTab >= 0
                            color: saveArea.pressed ? Theme.accentDim : Theme.accent

                            Text {
                                id: saveText

                                anchors.centerIn: parent
                                text: qsTr("Salvar")
                                color: Theme.background0
                                font.pixelSize: 11
                                font.bold: true
                            }

                            MouseArea {
                                id: saveArea

                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.saveCurrentFile()
                            }
                        }
                    }

                    Rectangle {
                        id: editorArea

                        anchors.top: tabBar.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        radius: Theme.radius
                        color: Theme.background0

                        Text {
                            anchors.centerIn: parent
                            visible: root.currentTab < 0
                            width: Math.min(parent.width - 40, 480)
                            horizontalAlignment: Text.AlignHCenter
                            wrapMode: Text.WordWrap
                            text: coreClient.workspaceRoot === ""
                                  ? qsTr("Abra uma pasta para comecar "
                                         + "(botao \"Abrir pasta...\" acima).")
                                  : qsTr("Clique em um arquivo no explorer para abrir.")
                            color: Theme.textMuted
                            font.pixelSize: 14
                        }

                        Flickable {
                            id: editorFlick

                            anchors.fill: parent
                            anchors.margins: Theme.spacingSmall
                            visible: root.currentTab >= 0
                            clip: true
                            contentWidth: editor.contentWidth + 20
                            contentHeight: editor.contentHeight + 20
                            boundsBehavior: Flickable.StopAtBounds

                            function ensureVisible(rect) {
                                if (contentX >= rect.x) {
                                    contentX = rect.x;
                                } else if (contentX + width <= rect.x + rect.width) {
                                    contentX = rect.x + rect.width - width;
                                }
                                if (contentY >= rect.y) {
                                    contentY = rect.y;
                                } else if (contentY + height <= rect.y + rect.height) {
                                    contentY = rect.y + rect.height - height;
                                }
                            }

                            TextEdit {
                                id: editor

                                EditorHighlighter {
                                    id: editorHighlighter

                                    document: editor.textDocument
                                }

                                width: Math.max(editorFlick.width, contentWidth)
                                height: Math.max(editorFlick.height, contentHeight)
                                color: Theme.textPrimary
                                selectionColor: Theme.accentDim
                                selectedTextColor: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 13
                                wrapMode: TextEdit.NoWrap
                                selectByMouse: true
                                tabStopDistance: 4 * 8
                                onCursorRectangleChanged:
                                    editorFlick.ensureVisible(cursorRectangle)
                                onTextChanged: {
                                    if (!root.loadingEditorText && root.currentTab >= 0) {
                                        root.hoverVisible = false;
                                        const saved =
                                            openFiles.get(root.currentTab).savedContent;
                                        openFiles.setProperty(root.currentTab, "modified",
                                                              text !== saved);
                                        changeDebounce.restart();
                                        if (root.completionVisible) {
                                            root.refilterCompletions();
                                        } else {
                                            completionDebounce.restart();
                                        }
                                    }
                                }
                                Keys.onPressed: function(event) {
                                    if (root.completionVisible) {
                                        if (event.key === Qt.Key_Down) {
                                            root.completionIndex = Math.min(
                                                root.completionIndex + 1,
                                                completionModel.count - 1);
                                            event.accepted = true;
                                            return;
                                        }
                                        if (event.key === Qt.Key_Up) {
                                            root.completionIndex = Math.max(
                                                root.completionIndex - 1, 0);
                                            event.accepted = true;
                                            return;
                                        }
                                        if (event.key === Qt.Key_Return
                                                || event.key === Qt.Key_Enter
                                                || event.key === Qt.Key_Tab) {
                                            root.acceptCompletion();
                                            event.accepted = true;
                                            return;
                                        }
                                        if (event.key === Qt.Key_Escape) {
                                            root.completionVisible = false;
                                            event.accepted = true;
                                            return;
                                        }
                                    }
                                    if (event.key === Qt.Key_Escape) {
                                        if (root.usagesVisible) {
                                            root.usagesVisible = false;
                                            event.accepted = true;
                                            return;
                                        }
                                        if (root.hoverVisible) {
                                            root.hoverVisible = false;
                                            event.accepted = true;
                                        }
                                    }
                                    if (event.key === Qt.Key_Tab
                                            && (event.modifiers & Qt.ShiftModifier)) {
                                        root.unindentEditorSelection();
                                        event.accepted = true;
                                        return;
                                    }
                                    if (event.key === Qt.Key_Tab) {
                                        root.indentEditorSelection();
                                        event.accepted = true;
                                        return;
                                    }
                                    if (event.key === Qt.Key_Backtab) {
                                        root.unindentEditorSelection();
                                        event.accepted = true;
                                        return;
                                    }
                                    if ((event.key === Qt.Key_Return
                                            || event.key === Qt.Key_Enter)
                                            && event.modifiers === Qt.NoModifier) {
                                        root.insertEditorNewline();
                                        event.accepted = true;
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        id: hoverPopup

                        visible: root.hoverVisible && root.hoverText !== ""
                        z: 20
                        anchors.top: tabBar.bottom
                        anchors.right: parent.right
                        anchors.topMargin: 2 * Theme.spacingSmall
                        anchors.rightMargin: 2 * Theme.spacingSmall
                        width: Math.min(460, editorPanel.width - 4 * Theme.spacingSmall)
                        height: Math.min(180, hoverTextItem.contentHeight
                                         + 2 * Theme.spacingMedium)
                        radius: Theme.radius
                        color: Theme.background2
                        border.color: Theme.accentDim
                        border.width: 1
                        clip: true

                        Flickable {
                            anchors.fill: parent
                            anchors.margins: Theme.spacingMedium
                            contentWidth: width
                            contentHeight: hoverTextItem.contentHeight
                            clip: true

                            Text {
                                id: hoverTextItem

                                width: parent.width
                                text: root.hoverText
                                color: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                wrapMode: Text.WrapAnywhere
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            acceptedButtons: Qt.RightButton
                            onClicked: root.hoverVisible = false
                        }
                    }

                    // Popup de completion (Ctrl+Space), ancorado ao cursor.
                    Rectangle {
                        id: completionPopup

                        visible: root.completionVisible && root.currentTab >= 0
                        z: 30
                        width: Math.min(400, editorPanel.width - 4 * Theme.spacingSmall)
                        height: Math.min(224, completionModel.count * 24
                                         + 2 * Theme.spacingSmall)
                        x: {
                            const scrollX = editorFlick.contentX;
                            const rect = editor.cursorRectangle;
                            const point = editor.mapToItem(editorPanel, rect.x, rect.y);
                            return Math.max(Theme.spacingSmall,
                                            Math.min(point.x, editorPanel.width - width
                                                     - Theme.spacingSmall));
                        }
                        y: {
                            const scrollY = editorFlick.contentY;
                            const rect = editor.cursorRectangle;
                            const point = editor.mapToItem(editorPanel, rect.x, rect.y);
                            const below = point.y + rect.height + 4;
                            if (below + height > editorPanel.height - Theme.spacingSmall) {
                                return Math.max(Theme.spacingSmall, point.y - height - 4);
                            }
                            return below;
                        }
                        radius: Theme.radius
                        color: Theme.background2
                        border.color: Theme.accentDim
                        border.width: 1
                        clip: true

                        ListView {
                            id: completionList

                            anchors.fill: parent
                            anchors.margins: Theme.spacingSmall
                            clip: true
                            model: completionModel
                            currentIndex: root.completionIndex
                            onCurrentIndexChanged:
                                positionViewAtIndex(currentIndex, ListView.Contain)

                            delegate: Rectangle {
                                required property int index
                                required property string label
                                required property string detail
                                required property string kind

                                width: completionList.width
                                height: 24
                                radius: Theme.radius
                                color: index === root.completionIndex
                                       ? Theme.surface2 : "transparent"

                                Row {
                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.left: parent.left
                                    anchors.leftMargin: Theme.spacingSmall
                                    anchors.right: parent.right
                                    anchors.rightMargin: Theme.spacingSmall
                                    spacing: Theme.spacingSmall

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 52
                                        text: parent.parent.kind
                                        color: Theme.accent
                                        font.pixelSize: 9
                                        elide: Text.ElideRight
                                    }

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: parent.parent.label
                                        color: Theme.textPrimary
                                        font.family: Theme.monoFont
                                        font.pixelSize: 12
                                    }

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: parent.width - x
                                        text: parent.parent.detail
                                        color: Theme.textMuted
                                        font.pixelSize: 10
                                        elide: Text.ElideRight
                                    }
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        root.completionIndex = parent.index;
                                        root.acceptCompletion();
                                    }
                                }
                            }
                        }
                    }

                    // Popup de usos (Alt+F7), estilo find usages compacto.
                    Rectangle {
                        id: usagesPopup

                        visible: root.usagesVisible
                        z: 25
                        anchors.top: tabBar.bottom
                        anchors.right: parent.right
                        anchors.topMargin: 2 * Theme.spacingSmall
                        anchors.rightMargin: 2 * Theme.spacingSmall
                        width: Math.min(420, editorPanel.width - 4 * Theme.spacingSmall)
                        height: Math.min(260, usagesModel.count * 22 + 34
                                         + 2 * Theme.spacingSmall)
                        radius: Theme.radius
                        color: Theme.background2
                        border.color: Theme.accentDim
                        border.width: 1
                        clip: true

                        Row {
                            id: usagesHeader

                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.margins: Theme.spacingSmall
                            height: 20

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: qsTr("Usos (%1)").arg(usagesModel.count)
                                color: Theme.textPrimary
                                font.pixelSize: 11
                                font.bold: true
                            }

                            Item {
                                width: parent.width - x - usagesClose.width
                                height: 1
                            }

                            Rectangle {
                                id: usagesClose

                                anchors.verticalCenter: parent.verticalCenter
                                width: 18
                                height: 18
                                radius: Theme.radius
                                color: usagesCloseArea.containsMouse
                                       ? Theme.surface2 : "transparent"

                                Text {
                                    anchors.centerIn: parent
                                    text: "×"
                                    color: Theme.textSecondary
                                    font.pixelSize: 12
                                }

                                MouseArea {
                                    id: usagesCloseArea

                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.usagesVisible = false
                                }
                            }
                        }

                        ListView {
                            id: usagesList

                            anchors.top: usagesHeader.bottom
                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.margins: Theme.spacingSmall
                            anchors.topMargin: 2
                            clip: true
                            model: usagesModel

                            delegate: Rectangle {
                                required property string path
                                required property int line
                                required property int column
                                required property string display

                                width: usagesList.width
                                height: 22
                                radius: Theme.radius
                                color: usageArea.containsMouse
                                       ? Theme.surface2 : "transparent"

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.left: parent.left
                                    anchors.leftMargin: Theme.spacingSmall
                                    anchors.right: parent.right
                                    anchors.rightMargin: Theme.spacingSmall
                                    text: parent.display
                                    color: Theme.accent
                                    font.family: Theme.monoFont
                                    font.pixelSize: 11
                                    elide: Text.ElideMiddle
                                }

                                MouseArea {
                                    id: usageArea

                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.openDiagnostic(parent.path,
                                                                   parent.line,
                                                                   parent.column)
                                }
                            }
                        }
                    }

                    Rectangle {
                        id: createDialog

                        visible: root.createDialogVisible
                        z: 40
                        anchors.centerIn: parent
                        width: Math.min(360, editorPanel.width - 4 * Theme.spacingSmall)
                        height: createColumn.height + 2 * Theme.spacingMedium
                        radius: Theme.radius
                        color: Theme.background2
                        border.color: Theme.accent
                        border.width: 1

                        Column {
                            id: createColumn

                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingSmall

                            Text {
                                text: root.createDialogKind === "directory"
                                      ? qsTr("Nova pasta") : qsTr("Novo arquivo")
                                color: Theme.textPrimary
                                font.pixelSize: 12
                                font.bold: true
                            }

                            Text {
                                width: parent.width
                                text: root.relativeToRoot(root.createDialogParentPath)
                                color: Theme.textMuted
                                font.pixelSize: 10
                                elide: Text.ElideMiddle
                            }

                            Rectangle {
                                width: parent.width
                                height: 30
                                radius: Theme.radius
                                color: Theme.background0
                                border.color: createNameInput.activeFocus
                                              ? Theme.accent : Theme.borderSoft
                                border.width: 1

                                TextInput {
                                    id: createNameInput

                                    anchors.fill: parent
                                    anchors.margins: Theme.spacingSmall
                                    verticalAlignment: TextInput.AlignVCenter
                                    color: Theme.textPrimary
                                    selectionColor: Theme.accentDim
                                    selectedTextColor: Theme.textPrimary
                                    font.family: Theme.monoFont
                                    font.pixelSize: 12
                                    clip: true
                                    selectByMouse: true
                                    onAccepted: root.confirmCreateEntry()
                                    Keys.onEscapePressed: {
                                        root.createDialogVisible = false;
                                        editor.forceActiveFocus();
                                    }
                                }
                            }

                            Text {
                                width: parent.width
                                visible: root.createDialogError !== ""
                                text: root.createDialogError
                                color: Theme.errorSoft
                                font.pixelSize: 10
                                wrapMode: Text.WordWrap
                            }

                            Row {
                                anchors.right: parent.right
                                spacing: Theme.spacingSmall

                                Rectangle {
                                    width: createCancelText.width + 2 * Theme.spacingMedium
                                    height: 24
                                    radius: Theme.radius
                                    color: createCancelArea.containsMouse
                                           ? Theme.surface2 : Theme.surface1
                                    border.color: Theme.borderSoft
                                    border.width: 1

                                    Text {
                                        id: createCancelText

                                        anchors.centerIn: parent
                                        text: qsTr("Cancelar")
                                        color: Theme.textSecondary
                                        font.pixelSize: 11
                                    }

                                    MouseArea {
                                        id: createCancelArea

                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            root.createDialogVisible = false;
                                            editor.forceActiveFocus();
                                        }
                                    }
                                }

                                Rectangle {
                                    width: createConfirmText.width + 2 * Theme.spacingMedium
                                    height: 24
                                    radius: Theme.radius
                                    color: createConfirmArea.pressed
                                           ? Theme.accentDim : Theme.accent

                                    Text {
                                        id: createConfirmText

                                        anchors.centerIn: parent
                                        text: qsTr("Criar")
                                        color: Theme.background0
                                        font.pixelSize: 11
                                        font.bold: true
                                    }

                                    MouseArea {
                                        id: createConfirmArea

                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.confirmCreateEntry()
                                    }
                                }
                            }
                        }
                    }

                    // Dialogo de rename (Shift+F6), primeira refatoracao visual.
                    Rectangle {
                        id: renameDialog

                        visible: root.renameDialogVisible
                        z: 40
                        anchors.centerIn: parent
                        width: Math.min(360, editorPanel.width - 4 * Theme.spacingSmall)
                        height: renameColumn.height + 2 * Theme.spacingMedium
                        radius: Theme.radius
                        color: Theme.background2
                        border.color: Theme.accent
                        border.width: 1

                        Column {
                            id: renameColumn

                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.margins: Theme.spacingMedium
                            spacing: Theme.spacingSmall

                            Text {
                                text: qsTr("Renomear simbolo")
                                color: Theme.textPrimary
                                font.pixelSize: 12
                                font.bold: true
                            }

                            Rectangle {
                                width: parent.width
                                height: 30
                                radius: Theme.radius
                                color: Theme.background0
                                border.color: renameInput.activeFocus
                                              ? Theme.accent : Theme.borderSoft
                                border.width: 1

                                TextInput {
                                    id: renameInput

                                    anchors.fill: parent
                                    anchors.margins: Theme.spacingSmall
                                    verticalAlignment: TextInput.AlignVCenter
                                    color: Theme.textPrimary
                                    selectionColor: Theme.accentDim
                                    selectedTextColor: Theme.textPrimary
                                    font.family: Theme.monoFont
                                    font.pixelSize: 12
                                    clip: true
                                    selectByMouse: true
                                    onAccepted: root.confirmRename()
                                    Keys.onEscapePressed: {
                                        root.renameDialogVisible = false;
                                        editor.forceActiveFocus();
                                    }
                                }
                            }

                            Text {
                                width: parent.width
                                visible: root.renameError !== ""
                                text: root.renameError
                                color: Theme.errorSoft
                                font.pixelSize: 10
                                wrapMode: Text.WordWrap
                            }

                            Row {
                                anchors.right: parent.right
                                spacing: Theme.spacingSmall

                                Rectangle {
                                    width: renameCancelText.width + 2 * Theme.spacingMedium
                                    height: 24
                                    radius: Theme.radius
                                    color: renameCancelArea.containsMouse
                                           ? Theme.surface2 : Theme.surface1
                                    border.color: Theme.borderSoft
                                    border.width: 1

                                    Text {
                                        id: renameCancelText

                                        anchors.centerIn: parent
                                        text: qsTr("Cancelar")
                                        color: Theme.textSecondary
                                        font.pixelSize: 11
                                    }

                                    MouseArea {
                                        id: renameCancelArea

                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            root.renameDialogVisible = false;
                                            editor.forceActiveFocus();
                                        }
                                    }
                                }

                                Rectangle {
                                    width: renameConfirmText.width + 2 * Theme.spacingMedium
                                    height: 24
                                    radius: Theme.radius
                                    color: renameConfirmArea.pressed
                                           ? Theme.accentDim : Theme.accent

                                    Text {
                                        id: renameConfirmText

                                        anchors.centerIn: parent
                                        text: qsTr("Renomear")
                                        color: Theme.background0
                                        font.pixelSize: 11
                                        font.bold: true
                                    }

                                    MouseArea {
                                        id: renameConfirmArea

                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: root.confirmRename()
                                    }
                                }
                            }
                        }
                    }
                }

                // Ilha: painel inferior (Logs / Ferramentas)
                Rectangle {
                    id: bottomPanel

                    width: parent.width
                    height: 170
                    visible: root.showBottomPanel
                    radius: Theme.radiusLarge
                    color: Theme.background2
                    border.color: Theme.borderSoft
                    border.width: 1

                    Row {
                        id: bottomTabs

                        anchors.top: parent.top
                        anchors.left: parent.left
                        anchors.margins: Theme.spacingSmall
                        spacing: Theme.spacingSmall

                        Repeater {
                            model: [
                                { key: "build", label: qsTr("Build") },
                                { key: "problems", label: qsTr("Problemas") },
                                { key: "tests", label: qsTr("Testes") },
                                { key: "run", label: qsTr("Executar") },
                                { key: "terminal", label: qsTr("Terminal") },
                                { key: "search", label: qsTr("Busca") },
                                { key: "logs", label: qsTr("IDE") },
                                { key: "tools", label: qsTr("Ferramentas") }
                            ]

                            delegate: Rectangle {
                                required property var modelData

                                width: bottomTabLabel.width + 2 * Theme.spacingSmall
                                height: 20
                                radius: Theme.radius
                                color: root.bottomTab === modelData.key
                                       ? Theme.accentDim : "transparent"
                                border.color: Theme.borderSoft
                                border.width: 1

                                Text {
                                    id: bottomTabLabel

                                    anchors.centerIn: parent
                                    text: parent.modelData.key === "problems"
                                          && problemsModel.count > 0
                                          ? qsTr("Problemas (%1)").arg(problemsModel.count)
                                          : parent.modelData.label
                                    color: root.bottomTab === parent.modelData.key
                                           ? Theme.accent : Theme.textSecondary
                                    font.pixelSize: 10
                                    font.bold: true
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.toggleBottomTab(parent.modelData.key)
                                }
                            }
                        }

                        Rectangle {
                            width: refreshToolsLabel.width + 2 * Theme.spacingSmall
                            height: 20
                            radius: Theme.radius
                            visible: root.bottomTab === "tools"
                            color: refreshToolsArea.containsMouse
                                   ? Theme.surface2 : "transparent"
                            border.color: Theme.borderSoft
                            border.width: 1

                            Text {
                                id: refreshToolsLabel

                                anchors.centerIn: parent
                                text: qsTr("⟳ redetectar")
                                color: Theme.textSecondary
                                font.pixelSize: 10
                            }

                            MouseArea {
                                id: refreshToolsArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: coreClient.detectTools()
                            }
                        }
                    }

                    ListView {
                        id: buildView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "build"
                        clip: true
                        model: buildOutputModel
                        onCountChanged: positionViewAtEnd()

                        delegate: Text {
                            required property string line

                            width: buildView.width
                            text: line
                            color: Theme.textSecondary
                            font.family: Theme.monoFont
                            font.pixelSize: 11
                            wrapMode: Text.WrapAnywhere
                        }
                    }

                    Item {
                        id: testsView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "tests"

                        Text {
                            id: testSummaryLabel

                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            visible: root.testSummary !== ""
                            text: root.testSummary
                            color: Theme.textSecondary
                            font.pixelSize: 11
                            font.bold: true
                        }

                        ListView {
                            id: testCasesView

                            anchors.top: root.testSummary !== ""
                                         ? testSummaryLabel.bottom : parent.top
                            anchors.topMargin: root.testSummary !== ""
                                               ? Theme.spacingSmall : 0
                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            clip: true
                            spacing: 1
                            model: testModel
                            onCountChanged: positionViewAtEnd()

                            Text {
                                anchors.centerIn: parent
                                visible: testModel.count === 0 && !coreClient.testing
                                text: qsTr("Nenhum teste rodado. Use ▶ Testes (Ctrl+Shift+F9).")
                                color: Theme.textMuted
                                font.pixelSize: 11
                            }

                            delegate: Row {
                                required property string name
                                required property string status

                                width: testCasesView.width
                                height: 18
                                spacing: Theme.spacingSmall

                                Rectangle {
                                    width: 7
                                    height: 7
                                    radius: 4
                                    anchors.verticalCenter: parent.verticalCenter
                                    color: root.testStatusColor(parent.status)
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: parent.name
                                    color: parent.status === "failed"
                                           ? Theme.textPrimary : Theme.textSecondary
                                    font.family: Theme.monoFont
                                    font.pixelSize: 11
                                    elide: Text.ElideRight
                                    width: testCasesView.width - 16
                                }
                            }
                        }
                    }

                    ListView {
                        id: problemsView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "problems"
                        clip: true
                        spacing: 2
                        model: problemsModel

                        Text {
                            anchors.centerIn: parent
                            visible: problemsModel.count === 0
                            text: qsTr("Nenhum problema. Rode um build (Ctrl+F9) ou "
                                       + "uma análise (Ctrl+Shift+L).")
                            color: Theme.textMuted
                            font.pixelSize: 11
                        }

                        delegate: Rectangle {
                            required property string severity
                            required property string message
                            required property string file
                            required property int line
                            required property int column

                            width: problemsView.width
                            height: problemRow.height + Theme.spacingSmall
                            radius: Theme.radius
                            color: problemArea.containsMouse
                                   ? Theme.surface2 : "transparent"

                            Row {
                                id: problemRow

                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall
                                spacing: Theme.spacingSmall
                                width: parent.width - 2 * Theme.spacingSmall

                                Rectangle {
                                    width: 8
                                    height: 8
                                    radius: 4
                                    anchors.verticalCenter: parent.verticalCenter
                                    color: root.problemColor(parent.parent.severity)
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: parent.parent.file !== ""
                                    text: parent.parent.file + ":" + parent.parent.line
                                    color: Theme.accent
                                    font.family: Theme.monoFont
                                    font.pixelSize: 11
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: parent.width - x
                                    text: parent.parent.message
                                    color: Theme.textPrimary
                                    font.pixelSize: 11
                                    elide: Text.ElideRight
                                }
                            }

                            MouseArea {
                                id: problemArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.openDiagnostic(parent.file, parent.line,
                                                               parent.column)
                            }
                        }
                    }

                    Item {
                        id: shellView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "terminal"
                        onVisibleChanged: {
                            if (visible && !coreClient.terminalActive
                                    && coreClient.workspaceRoot !== "") {
                                coreClient.terminalOpen();
                            }
                            if (visible) {
                                shellInput.forceActiveFocus();
                            }
                        }

                        Flickable {
                            id: shellFlick

                            anchors.top: parent.top
                            anchors.bottom: shellInputBox.top
                            anchors.bottomMargin: Theme.spacingSmall
                            anchors.left: parent.left
                            anchors.right: parent.right
                            clip: true
                            contentWidth: width
                            contentHeight: shellText.contentHeight + 8
                            boundsBehavior: Flickable.StopAtBounds
                            onContentHeightChanged: {
                                if (contentHeight > height) {
                                    contentY = contentHeight - height;
                                }
                            }

                            TextEdit {
                                id: shellText

                                width: shellFlick.width
                                text: root.terminalText
                                readOnly: true
                                color: Theme.textSecondary
                                selectionColor: Theme.accentDim
                                selectedTextColor: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                wrapMode: TextEdit.WrapAnywhere
                                selectByMouse: true
                            }

                            Text {
                                anchors.centerIn: parent
                                visible: root.terminalText === ""
                                text: qsTr("Seu shell ($SHELL) abre aqui na raiz do"
                                           + " workspace (Alt+F12).")
                                color: Theme.textMuted
                                font.pixelSize: 11
                            }
                        }

                        Rectangle {
                            id: shellInputBox

                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            height: 24
                            radius: Theme.radius
                            color: Theme.background0
                            border.color: shellInput.activeFocus
                                          ? Theme.accent : Theme.borderSoft
                            border.width: 1

                            TextInput {
                                id: shellInput

                                anchors.fill: parent
                                anchors.margins: Theme.spacingSmall
                                verticalAlignment: TextInput.AlignVCenter
                                color: Theme.textPrimary
                                selectionColor: Theme.accentDim
                                selectedTextColor: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                clip: true
                                selectByMouse: true
                                onAccepted: root.submitShellInput()
                            }
                        }
                    }

                    Item {
                        id: runView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "run"

                        ListView {
                            id: runOutputView

                            anchors.top: parent.top
                            anchors.bottom: runInputBox.top
                            anchors.bottomMargin: Theme.spacingSmall
                            anchors.left: parent.left
                            anchors.right: parent.right
                            clip: true
                            model: runModel
                            onCountChanged: positionViewAtEnd()

                            Text {
                                anchors.centerIn: parent
                                visible: runModel.count === 0
                                text: qsTr("Digite um comando e pressione Enter,"
                                           + " ou use ▶ Iniciar (Shift+F10).")
                                color: Theme.textMuted
                                font.pixelSize: 11
                            }

                            delegate: Text {
                                required property string line
                                required property string kind

                                width: runOutputView.width
                                text: line
                                color: {
                                    if (kind === "command") {
                                        return Theme.accent;
                                    }
                                    if (kind === "stderr") {
                                        return Theme.errorSoft;
                                    }
                                    if (kind === "stdin" || kind === "info") {
                                        return Theme.textMuted;
                                    }
                                    return Theme.textSecondary;
                                }
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                wrapMode: Text.WrapAnywhere
                            }
                        }

                        Rectangle {
                            id: runInputBox

                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            height: 24
                            radius: Theme.radius
                            color: Theme.background0
                            border.color: runInput.activeFocus
                                          ? Theme.accent : Theme.borderSoft
                            border.width: 1

                            Text {
                                id: runPrompt

                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall
                                anchors.verticalCenter: parent.verticalCenter
                                text: coreClient.running ? ">" : "$"
                                color: coreClient.running
                                       ? Theme.warningSoft : Theme.accent
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                font.bold: true
                            }

                            TextInput {
                                id: runInput

                                anchors.left: runPrompt.right
                                anchors.leftMargin: Theme.spacingSmall
                                anchors.right: parent.right
                                anchors.rightMargin: Theme.spacingSmall
                                anchors.verticalCenter: parent.verticalCenter
                                verticalAlignment: TextInput.AlignVCenter
                                color: Theme.textPrimary
                                selectionColor: Theme.accentDim
                                selectedTextColor: Theme.textPrimary
                                font.family: Theme.monoFont
                                font.pixelSize: 11
                                clip: true
                                selectByMouse: true
                                onAccepted: root.submitRunInput()

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: runInput.text === ""
                                    text: coreClient.running
                                          ? qsTr("enviar para o stdin do processo")
                                          : qsTr("comando no workspace (ex.: cargo test)")
                                    color: Theme.textMuted
                                    font.pixelSize: 11
                                }
                            }
                        }
                    }

                    Item {
                        id: searchView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "search"

                        Row {
                            id: searchControls

                            width: parent.width
                            spacing: Theme.spacingSmall

                            Rectangle {
                                width: parent.width - caseChip.width
                                       - searchStatus.width - 2 * Theme.spacingSmall
                                height: 24
                                radius: Theme.radius
                                color: Theme.background0
                                border.color: searchInput.activeFocus
                                              ? Theme.accent : Theme.borderSoft
                                border.width: 1

                                TextInput {
                                    id: searchInput

                                    anchors.fill: parent
                                    anchors.margins: Theme.spacingSmall
                                    verticalAlignment: TextInput.AlignVCenter
                                    color: Theme.textPrimary
                                    selectionColor: Theme.accentDim
                                    selectedTextColor: Theme.textPrimary
                                    font.family: Theme.monoFont
                                    font.pixelSize: 11
                                    clip: true
                                    selectByMouse: true
                                    onAccepted: root.runSearch()

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        visible: searchInput.text === ""
                                        text: qsTr("Buscar texto no workspace (Enter)")
                                        color: Theme.textMuted
                                        font.pixelSize: 11
                                    }
                                }
                            }

                            Rectangle {
                                id: caseChip

                                width: caseChipLabel.width + 2 * Theme.spacingSmall
                                height: 24
                                radius: Theme.radius
                                color: root.searchCaseSensitive
                                       ? Theme.accentDim : "transparent"
                                border.color: Theme.borderSoft
                                border.width: 1

                                Text {
                                    id: caseChipLabel

                                    anchors.centerIn: parent
                                    text: qsTr("Aa")
                                    color: root.searchCaseSensitive
                                           ? Theme.accent : Theme.textSecondary
                                    font.pixelSize: 10
                                    font.bold: true
                                }

                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: {
                                        root.searchCaseSensitive =
                                                !root.searchCaseSensitive;
                                        root.runSearch();
                                    }
                                }
                            }

                            Text {
                                id: searchStatus

                                anchors.verticalCenter: parent.verticalCenter
                                text: {
                                    if (root.searching) {
                                        return qsTr("buscando...");
                                    }
                                    if (searchModel.count === 0) {
                                        return "";
                                    }
                                    if (root.searchTruncated) {
                                        return qsTr("%1+ resultados")
                                                .arg(searchModel.count);
                                    }
                                    return qsTr("%1 resultados").arg(searchModel.count);
                                }
                                color: Theme.textMuted
                                font.pixelSize: 10
                            }
                        }

                        ListView {
                            id: searchResultsView

                            anchors.top: searchControls.bottom
                            anchors.topMargin: Theme.spacingSmall
                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            clip: true
                            spacing: 2
                            model: searchModel

                            Text {
                                anchors.centerIn: parent
                                visible: searchModel.count === 0 && !root.searching
                                text: qsTr("Digite um termo e pressione Enter"
                                           + " (Ctrl+Shift+F).")
                                color: Theme.textMuted
                                font.pixelSize: 11
                            }

                            delegate: Rectangle {
                                required property string path
                                required property int line
                                required property int column
                                required property string preview

                                width: searchResultsView.width
                                height: searchRow.height + Theme.spacingSmall
                                radius: Theme.radius
                                color: searchArea.containsMouse
                                       ? Theme.surface2 : "transparent"

                                Row {
                                    id: searchRow

                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.left: parent.left
                                    anchors.leftMargin: Theme.spacingSmall
                                    spacing: Theme.spacingSmall
                                    width: parent.width - 2 * Theme.spacingSmall

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: parent.parent.path + ":"
                                              + parent.parent.line
                                        color: Theme.accent
                                        font.family: Theme.monoFont
                                        font.pixelSize: 11
                                    }

                                    Text {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: parent.width - x
                                        text: parent.parent.preview
                                        color: Theme.textPrimary
                                        font.family: Theme.monoFont
                                        font.pixelSize: 11
                                        elide: Text.ElideRight
                                    }
                                }

                                MouseArea {
                                    id: searchArea

                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.openDiagnostic(parent.path,
                                                                   parent.line,
                                                                   parent.column)
                                }
                            }
                        }
                    }

                    ListView {
                        id: logsView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "logs"
                        clip: true
                        model: coreClient.logLines
                        onCountChanged: positionViewAtEnd()

                        delegate: Text {
                            required property string modelData

                            width: logsView.width
                            text: modelData
                            color: Theme.textSecondary
                            font.family: Theme.monoFont
                            font.pixelSize: 11
                            wrapMode: Text.WrapAnywhere
                        }
                    }

                    ListView {
                        id: toolsView

                        anchors.top: bottomTabs.bottom
                        anchors.bottom: parent.bottom
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.margins: Theme.spacingSmall
                        visible: root.bottomTab === "tools"
                        clip: true
                        model: root.toolsList

                        delegate: Rectangle {
                            required property var modelData

                            width: toolsView.width
                            height: 26
                            radius: Theme.radius
                            color: "transparent"

                            Row {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Theme.spacingSmall
                                spacing: Theme.spacingMedium

                                Rectangle {
                                    width: 8
                                    height: 8
                                    radius: 4
                                    anchors.verticalCenter: parent.verticalCenter
                                    color: root.toolStatusColor(modelData.status)
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 130
                                    text: modelData.displayName
                                    color: Theme.textPrimary
                                    font.pixelSize: 11
                                    font.bold: true
                                    elide: Text.ElideRight
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: modelData.version !== undefined
                                          ? modelData.version
                                          : (modelData.message !== undefined
                                             ? modelData.message : "")
                                    color: Theme.textSecondary
                                    font.pixelSize: 11
                                    font.family: Theme.monoFont
                                }

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: modelData.suggestedInstall !== undefined
                                    text: modelData.suggestedInstall !== undefined
                                          ? modelData.suggestedInstall : ""
                                    color: Theme.accent
                                    font.pixelSize: 11
                                    font.family: Theme.monoFont
                                }
                            }
                        }
                    }
                }
            }

            // Ilha: Assistente KW (casca; providers chegam na Fase 7)
            Rectangle {
                id: assistantPanel

                width: visible ? 300 : 0
                height: parent.height
                visible: root.showAssistant
                radius: Theme.radiusLarge
                color: Theme.background1
                border.color: Theme.borderSoft
                border.width: 1

                Column {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingSmall
                    spacing: Theme.spacingSmall

                    Row {
                        id: assistantHeader

                        width: parent.width
                        height: 26
                        spacing: Theme.spacingSmall

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: "✦"
                            color: Theme.accent
                            font.pixelSize: 13
                        }

                        Text {
                            anchors.verticalCenter: parent.verticalCenter
                            text: qsTr("Assistente KW")
                            color: Theme.textPrimary
                            font.pixelSize: 13
                            font.bold: true
                        }

                        Rectangle {
                            anchors.verticalCenter: parent.verticalCenter
                            width: assistantBadge.width + 10
                            height: 16
                            radius: 8
                            color: Theme.surface2

                            Text {
                                id: assistantBadge

                                anchors.centerIn: parent
                                text: qsTr("offline")
                                color: Theme.textMuted
                                font.pixelSize: 9
                                font.bold: true
                            }
                        }

                        Item {
                            width: parent.width - x - assistantClose.width
                            height: 1
                        }

                        Rectangle {
                            id: assistantClose

                            anchors.verticalCenter: parent.verticalCenter
                            width: 22
                            height: 22
                            radius: Theme.radius
                            color: assistantCloseArea.containsMouse
                                   ? Theme.surface2 : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "×"
                                color: Theme.textSecondary
                                font.pixelSize: 13
                            }

                            MouseArea {
                                id: assistantCloseArea

                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.showAssistant = false
                            }
                        }
                    }

                    ListView {
                        id: assistantView

                        width: parent.width
                        height: parent.height - assistantHeader.height
                                - assistantInputBox.height - 2 * Theme.spacingSmall
                        clip: true
                        spacing: Theme.spacingSmall
                        model: assistantModel
                        onCountChanged: positionViewAtEnd()

                        delegate: Rectangle {
                            required property string role
                            required property string body

                            width: assistantView.width
                            height: messageText.height + 2 * Theme.spacingSmall
                            radius: Theme.radius
                            color: role === "user" ? Theme.surface2 : Theme.surface1
                            border.color: role === "user"
                                          ? Theme.accentDim : Theme.borderSoft
                            border.width: 1

                            Text {
                                id: messageText

                                anchors.top: parent.top
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.margins: Theme.spacingSmall
                                text: parent.body
                                color: parent.role === "user"
                                       ? Theme.textPrimary : Theme.textSecondary
                                font.pixelSize: 11
                                wrapMode: Text.WordWrap
                            }
                        }
                    }

                    Rectangle {
                        id: assistantInputBox

                        width: parent.width
                        height: 34
                        radius: Theme.radius
                        color: Theme.background0
                        border.color: assistantInput.activeFocus
                                      ? Theme.accent : Theme.borderSoft
                        border.width: 1

                        TextInput {
                            id: assistantInput

                            anchors.left: parent.left
                            anchors.right: assistantSend.left
                            anchors.top: parent.top
                            anchors.bottom: parent.bottom
                            anchors.margins: Theme.spacingSmall
                            verticalAlignment: TextInput.AlignVCenter
                            color: Theme.textPrimary
                            selectionColor: Theme.accentDim
                            selectedTextColor: Theme.textPrimary
                            font.pixelSize: 11
                            clip: true
                            selectByMouse: true
                            onAccepted: root.sendAssistantMessage(text)
                        }

                        Rectangle {
                            id: assistantSend

                            anchors.verticalCenter: parent.verticalCenter
                            anchors.right: parent.right
                            anchors.rightMargin: 4
                            width: 26
                            height: 26
                            radius: Theme.radius
                            color: assistantSendArea.pressed
                                   ? Theme.accentDim : Theme.accent

                            Text {
                                anchors.centerIn: parent
                                text: "➤"
                                color: Theme.background0
                                font.pixelSize: 11
                                font.bold: true
                            }

                            MouseArea {
                                id: assistantSendArea

                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: root.sendAssistantMessage(assistantInput.text)
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Status bar ────────────────────────────────────────────────────────
    Rectangle {
        id: statusBar

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: 26
        color: Theme.background1

        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingMedium
            spacing: Theme.spacingMedium

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.workspaceRoot !== ""
                text: root.kindLabel(coreClient.workspaceKind) + "  ·  "
                      + coreClient.workspaceRoot
                color: Theme.textMuted
                font.pixelSize: 10
                font.family: Theme.monoFont
            }
        }

        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingMedium
            spacing: Theme.spacingMedium

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: logsToggleText.width + 2 * Theme.spacingSmall
                height: 18
                radius: Theme.radius
                color: root.showBottomPanel && root.bottomTab === "logs"
                       ? Theme.accentDim
                       : (logsToggleArea.containsMouse ? Theme.surface2 : "transparent")
                border.color: Theme.borderSoft
                border.width: 1

                Text {
                    id: logsToggleText

                    anchors.centerIn: parent
                    text: qsTr("IDE")
                    color: root.showBottomPanel && root.bottomTab === "logs"
                           ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                }

                MouseArea {
                    id: logsToggleArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.toggleBottomTab("logs")
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.building
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("compilando...")
                    color: Theme.accent
                    font.pixelSize: 10
                }

                Text {
                    id: cancelBuildLabel

                    anchors.verticalCenter: parent.verticalCenter
                    text: "×"
                    color: cancelBuildArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 12

                    MouseArea {
                        id: cancelBuildArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: coreClient.cancelBuild()
                    }
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.testing
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("testando...")
                    color: Theme.accent
                    font.pixelSize: 10
                }

                Text {
                    id: cancelTestsLabel

                    anchors.verticalCenter: parent.verticalCenter
                    text: "×"
                    color: cancelTestsArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 12

                    MouseArea {
                        id: cancelTestsArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: coreClient.cancelTests()
                    }
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.analyzing
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("analisando...")
                    color: Theme.accent
                    font.pixelSize: 10
                }

                Text {
                    id: cancelQualityLabel

                    anchors.verticalCenter: parent.verticalCenter
                    text: "×"
                    color: cancelQualityArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 12

                    MouseArea {
                        id: cancelQualityArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: coreClient.cancelQuality()
                    }
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.scanningEnvironment
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("scan de ambiente...")
                    color: Theme.accent
                    font.pixelSize: 10
                }

                Text {
                    id: cancelEnvironmentLabel

                    anchors.verticalCenter: parent.verticalCenter
                    text: "×"
                    color: cancelEnvironmentArea.containsMouse ? Theme.errorSoft : Theme.textMuted
                    font.pixelSize: 12

                    MouseArea {
                        id: cancelEnvironmentArea

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: coreClient.cancelEnvironmentScan()
                    }
                }
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                visible: coreClient.running
                text: qsTr("executando...")
                color: Theme.accent
                font.pixelSize: 10
            }

            Rectangle {
                width: 7
                height: 7
                radius: 4
                anchors.verticalCenter: parent.verticalCenter
                color: coreClient.connected ? Theme.successSoft : Theme.errorSoft
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: coreClient.connected
                      ? qsTr("core conectado · IPC %1").arg(coreClient.protocolVersion)
                      : qsTr("core %1").arg(coreClient.status)
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }
    }

    // ── Menu de contexto do Project panel (rename/delete) ──────────────────
    Item {
        anchors.fill: parent
        visible: root.entryMenuVisible
        z: 100

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: root.entryMenuVisible = false
        }

        Rectangle {
            x: root.entryMenuX
            y: root.entryMenuY
            width: 168
            height: entryMenuColumn.height + 2 * Theme.spacingSmall
            radius: Theme.radius
            color: Theme.background2
            border.color: Theme.borderSoft
            border.width: 1

            Column {
                id: entryMenuColumn

                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingSmall
                spacing: 2

                Rectangle {
                    width: parent.width
                    height: 26
                    radius: Theme.radius
                    color: entryRenameHover.containsMouse ? Theme.surface2 : "transparent"

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        text: qsTr("Renomear")
                        color: Theme.textPrimary
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: entryRenameHover

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.openEntryRename()
                    }
                }

                Rectangle {
                    width: parent.width
                    height: 26
                    radius: Theme.radius
                    color: entryDeleteHover.containsMouse ? Theme.surface2 : "transparent"

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        text: qsTr("Excluir")
                        color: Theme.errorSoft
                        font.pixelSize: 12
                    }

                    MouseArea {
                        id: entryDeleteHover

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.openEntryDelete()
                    }
                }
            }
        }
    }

    // ── Dialogo de rename do Project panel ─────────────────────────────────
    Item {
        anchors.fill: parent
        visible: root.entryRenameVisible
        z: 101

        MouseArea {
            anchors.fill: parent
        }

        Rectangle {
            anchors.centerIn: parent
            width: Math.min(360, root.width - 4 * Theme.spacingMedium)
            height: entryRenameColumn.height + 2 * Theme.spacingMedium
            radius: Theme.radius
            color: Theme.background2
            border.color: Theme.accent
            border.width: 1

            Column {
                id: entryRenameColumn

                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingSmall

                Text {
                    text: root.entryRenameKind === "directory"
                          ? qsTr("Renomear pasta") : qsTr("Renomear arquivo")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: root.relativeToRoot(root.entryRenamePath)
                    color: Theme.textMuted
                    font.pixelSize: 10
                    elide: Text.ElideMiddle
                }

                Rectangle {
                    width: parent.width
                    height: 30
                    radius: Theme.radius
                    color: Theme.background0
                    border.color: entryRenameInput.activeFocus
                                  ? Theme.accent : Theme.borderSoft
                    border.width: 1

                    TextInput {
                        id: entryRenameInput

                        anchors.fill: parent
                        anchors.margins: Theme.spacingSmall
                        verticalAlignment: TextInput.AlignVCenter
                        color: Theme.textPrimary
                        selectionColor: Theme.accentDim
                        selectedTextColor: Theme.textPrimary
                        font.family: Theme.monoFont
                        font.pixelSize: 12
                        clip: true
                        selectByMouse: true
                        onAccepted: root.confirmEntryRename()
                        Keys.onEscapePressed: {
                            root.entryRenameVisible = false;
                            editor.forceActiveFocus();
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: root.entryRenameError !== ""
                    text: root.entryRenameError
                    color: Theme.errorSoft
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                }

                Row {
                    anchors.right: parent.right
                    spacing: Theme.spacingSmall

                    Rectangle {
                        width: entryRenameCancelText.width + 2 * Theme.spacingMedium
                        height: 24
                        radius: Theme.radius
                        color: entryRenameCancelArea.containsMouse
                               ? Theme.surface2 : Theme.surface1
                        border.color: Theme.borderSoft
                        border.width: 1

                        Text {
                            id: entryRenameCancelText

                            anchors.centerIn: parent
                            text: qsTr("Cancelar")
                            color: Theme.textSecondary
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: entryRenameCancelArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.entryRenameVisible = false;
                                editor.forceActiveFocus();
                            }
                        }
                    }

                    Rectangle {
                        width: entryRenameConfirmText.width + 2 * Theme.spacingMedium
                        height: 24
                        radius: Theme.radius
                        color: entryRenameConfirmArea.pressed
                               ? Theme.accentDim : Theme.accent

                        Text {
                            id: entryRenameConfirmText

                            anchors.centerIn: parent
                            text: qsTr("Renomear")
                            color: Theme.background0
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: entryRenameConfirmArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.confirmEntryRename()
                        }
                    }
                }
            }
        }
    }

    // ── Dialogo de confirmacao de exclusao ─────────────────────────────────
    Item {
        anchors.fill: parent
        visible: root.entryDeleteVisible
        z: 102

        MouseArea {
            anchors.fill: parent
        }

        Rectangle {
            anchors.centerIn: parent
            width: Math.min(380, root.width - 4 * Theme.spacingMedium)
            height: entryDeleteColumn.height + 2 * Theme.spacingMedium
            radius: Theme.radius
            color: Theme.background2
            border.color: Theme.errorSoft
            border.width: 1

            Column {
                id: entryDeleteColumn

                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingSmall

                Text {
                    text: root.entryDeleteKind === "directory"
                          ? qsTr("Excluir pasta") : qsTr("Excluir arquivo")
                    color: Theme.textPrimary
                    font.pixelSize: 12
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: root.entryDeleteKind === "directory"
                          ? qsTr("A pasta \"%1\" e todo o seu conteudo serao removidos "
                                 + "do disco. Esta acao nao pode ser desfeita.")
                            .arg(root.entryDeleteName)
                          : qsTr("O arquivo \"%1\" sera removido do disco. "
                                 + "Esta acao nao pode ser desfeita.")
                            .arg(root.entryDeleteName)
                    color: Theme.textSecondary
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                }

                Text {
                    width: parent.width
                    visible: root.entryDeleteError !== ""
                    text: root.entryDeleteError
                    color: Theme.errorSoft
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                }

                Row {
                    anchors.right: parent.right
                    spacing: Theme.spacingSmall

                    Rectangle {
                        width: entryDeleteCancelText.width + 2 * Theme.spacingMedium
                        height: 24
                        radius: Theme.radius
                        color: entryDeleteCancelArea.containsMouse
                               ? Theme.surface2 : Theme.surface1
                        border.color: Theme.borderSoft
                        border.width: 1

                        Text {
                            id: entryDeleteCancelText

                            anchors.centerIn: parent
                            text: qsTr("Cancelar")
                            color: Theme.textSecondary
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: entryDeleteCancelArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.entryDeleteVisible = false;
                                editor.forceActiveFocus();
                            }
                        }
                    }

                    Rectangle {
                        width: entryDeleteConfirmText.width + 2 * Theme.spacingMedium
                        height: 24
                        radius: Theme.radius
                        color: entryDeleteConfirmArea.pressed
                               ? Qt.darker(Theme.errorSoft, 1.4) : Theme.errorSoft

                        Text {
                            id: entryDeleteConfirmText

                            anchors.centerIn: parent
                            text: qsTr("Excluir")
                            color: Theme.background0
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: entryDeleteConfirmArea

                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.confirmEntryDelete()
                        }
                    }
                }
            }
        }
    }

}
