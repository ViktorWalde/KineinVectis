import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property var editorSurface: null
    property var diagnosticsController: null
    property var settingsController: null
    // M4.1: auto-close (setting) e format-on-save (setting) via o
    // SettingsController; default seguro quando ainda não carregou.
    readonly property bool autoCloseEnabled:
        settingsController === null ? true : settingsController.autoClosePairs
    property alias filesModel: documents.filesModel
    property alias recentFiles: documents.recentFiles
    property alias completionModel: completionController.completionModel
    property alias usagesModel: usagesItemsModel
    property alias currentTab: documents.currentTab
    property alias externalConflict: documents.currentExternalConflict
    property alias externalDeleted: documents.currentExternalDeleted
    property alias externalMessage: documents.currentExternalMessage
    property string watchError: ""
    property alias loadingEditorText: surfaceBridge.loadingText
    // D1 (docs/24): alias para a property PRÓPRIA do controller, nunca para o
    // `visible` do Item — este EditorController é invisível (é controller), e
    // `Item.visible` de um filho lê a visibilidade EFETIVA, que fica presa em
    // false sob pai invisível. Era essa a causa do popup nunca abrir.
    property alias completionVisible: completionController.popupVisible
    property alias completionIndex: completionController.index
    property bool hoverVisible: false
    property string hoverText: ""
    property bool usagesVisible: false
    property bool renameDialogVisible: false
    property string renameError: ""
    property bool workspaceEditPreviewVisible: false
    property string workspaceEditTransactionId: ""
    property string workspaceEditTitle: ""
    property var workspaceEditFiles: []
    property int workspaceEditCount: 0
    property string workspaceEditError: ""
    property alias actionsModel: actionsItemsModel
    property bool actionsVisible: false
    property int actionsIndex: 0
    property bool goToLineVisible: false
    // D1b (docs/24): Find/Replace no arquivo. Mesma regra do D1 — alias para
    // a property PRÓPRIA do controller, nunca para o `visible` do Item.
    property alias findBarVisible: findController.barVisible
    property alias findReplaceMode: findController.replaceMode
    property alias findQuery: findController.query
    property alias findReplacement: findController.replacement
    property alias findCaseSensitive: findController.caseSensitive
    property alias findWholeWord: findController.wholeWord
    property alias findUseRegex: findController.useRegex
    property alias findInvalidRegex: findController.invalidRegex
    property alias findMatchCount: findController.matchCount
    property alias findCurrentDisplay: findController.currentDisplay
    property int syntaxVersion: 0
    property int semanticVersion: 0
    property string syntaxLanguage: "plain"
    property bool syntaxHasErrors: false
    property var syntaxOutline: []
    property var syntaxLocals: []

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content, string expectedContent)
    // M-S1 (docs/23): autosave/limpeza de rascunho não salvo (rede de segurança).
    signal draftSaveRequested(string path, string content)
    signal draftClearRequested(string path)
    signal formatRequested(string path, string content)
    signal fileChangedNotificationRequested(string path, string content)
    signal semanticTokensRequested(string path, string content, int version)
    signal syntaxTreeRequested(string path, string content, int version)
    signal switchSourceHeaderRequested(string path, string content)
    signal definitionRequested(string path, string content, int line, int column)
    signal hoverRequested(string path, string content, int line, int column)
    signal completionRequested(string path, string content, int line, int column)
    signal referencesRequested(string path, string content, int line, int column)
    signal renameRequested(string path, string content, int line, int column,
                           string newName)
    signal renameDialogOpenRequested(string currentName)
    signal codeActionsRequested(string path, string content, int line, int column)
    signal codeActionApplyRequested(string path, string content, int actionIndex)
    signal workspaceEditApplyRequested(string transactionId)
    signal workspaceEditCancelRequested(string transactionId)
    signal saveSessionRequested(var files, string activeFile)
    signal goToLineDialogOpenRequested(string prefill)
    // D1b: a barra abriu — a UI precisa focar o campo de busca.
    signal findBarOpenRequested()

    visible: false

    ListModel {
        id: usagesItemsModel
    }

    ListModel {
        id: actionsItemsModel
    }

    EditorSurfaceBridge {
        id: surfaceBridge

        editorSurface: root.editorSurface
    }

    EditorDocumentController {
        id: documents

        workspaceRoot: root.workspaceRoot
        surfaceBridge: surfaceBridge
        onReadFileRequested: function(path) {
            root.readFileRequested(path);
        }
        onWriteFileRequested: function(path, content, expectedContent) {
            root.writeFileRequested(path, content, expectedContent);
        }
        onCurrentDocumentChanged: {
            root.refreshSyntaxTree();
            root.refreshSemanticTokens();
        }
    }

    EditorTextController {
        id: textController

        surfaceBridge: surfaceBridge
    }

    EditorCompletionController {
        id: completionController

        surfaceBridge: surfaceBridge
        documentController: documents
        textController: textController
        localItems: root.syntaxLocals
        onCompletionRequested: function(path, content, line, column) {
            root.completionRequested(path, content, line, column);
        }
    }

    EditorFindController {
        id: findController

        surfaceBridge: surfaceBridge
        textController: textController
        // Os matches viram spans de realce no editor (todas as ocorrências,
        // a atual mais forte) — o highlighter é quem pinta.
        onSearchStateChanged: root.refreshSearchHighlight()
    }

    function refreshSearchHighlight() {
        if (!editorReady()) {
            return;
        }
        editorSurface.setSearchMatches(findController.barVisible
                                       ? findController.matches : [],
                                       findController.current);
    }

    // D1b: Ctrl+F / Ctrl+H. Sem arquivo aberto não há o que buscar.
    function openFind() {
        if (!editableFileOpen()) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        findController.open(false);
        findBarOpenRequested();
    }

    function openFindReplace() {
        if (!editableFileOpen()) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        findController.open(true);
        findBarOpenRequested();
    }

    function closeFind() {
        findController.close();
    }

    function setFindQuery(text) {
        findController.setQuery(text);
    }

    function setFindReplacement(text) {
        findController.setReplacement(text);
    }

    // F3 / Shift+F3 funcionam mesmo com a barra fechada: reusam o último
    // termo (igual VS Code). Sem termo, o F3 abre a barra.
    function findNext() {
        if (findController.query === "") {
            openFind();
            return;
        }
        if (!findController.barVisible) {
            findController.barVisible = true;
            findController.recompute(findController.cursorOffset());
        }
        findController.findNext();
    }

    function findPrevious() {
        if (findController.query === "") {
            openFind();
            return;
        }
        if (!findController.barVisible) {
            findController.barVisible = true;
            findController.recompute(findController.cursorOffset());
        }
        findController.findPrevious();
    }

    function toggleFindCase() {
        findController.toggleCaseSensitive();
    }

    function toggleFindWholeWord() {
        findController.toggleWholeWord();
    }

    function toggleFindRegex() {
        findController.toggleRegex();
    }

    function replaceFindCurrent() {
        findController.replaceCurrent();
    }

    function replaceFindAll() {
        findController.replaceAll();
    }

    function editorReady() {
        return surfaceBridge.ready();
    }

    function editorText() {
        return surfaceBridge.text();
    }

    function focusEditor() {
        surfaceBridge.focusEditor();
    }

    function clear() {
        documents.clear();
        completionController.clear();
        findController.close();
        hoverText = "";
        hoverVisible = false;
        usagesVisible = false;
        usagesItemsModel.clear();
        renameDialogVisible = false;
        renameError = "";
        resetWorkspaceEditPreview();
        dismissActions();
        pendingSaveAfterFormat = false;
        pendingSaveAllQueue = [];
        pendingSaveAllItem = null;
        watchError = "";
        syntaxVersion++;
        syntaxLanguage = "plain";
        syntaxHasErrors = false;
        syntaxOutline = [];
        syntaxLocals = [];
    }

    function storeCurrentEditor() {
        documents.storeCurrentEditor();
    }

    function selectTab(index) {
        completionController.dismiss();
        documents.selectTab(index);
        // D1b: o buffer trocou — os offsets dos matches eram do texto ANTIGO.
        // Revarre no arquivo novo (mantendo o termo, como VS Code faz).
        if (findController.barVisible) {
            findController.recompute(0);
        }
    }

    function closeTab(index) {
        // M-S1: fechar a aba é uma decisão explícita → o rascunho não é mais
        // necessário (só sobrevive a CRASH). Limpa antes de fechar.
        const path = index >= 0 && index < filesModel.count
                ? filesModel.get(index).path : "";
        if (path !== "") {
            draftClearRequested(path);
        }
        documents.closeTab(index);
    }

    // M4.1: format-on-save. Ctrl+S formata e SÓ ENTÃO salva. Assíncrono
    // robusto: pendingSaveAfterFormat salva no resolved E no failed do
    // format.text (servidor ausente/timeout não trava o save).
    property bool pendingSaveAfterFormat: false
    property var pendingSaveAllQueue: []
    property var pendingSaveAllItem: null

    function formatOnSaveEnabled() {
        return settingsController !== null && settingsController.formatOnSave;
    }

    function formattableLanguage() {
        const language = editorSurface.language;
        return language === "rust" || language === "cpp";
    }

    function saveCurrentFile() {
        if (pendingSaveAllItem !== null || pendingSaveAllQueue.length > 0) {
            return;
        }
        if (formatOnSaveEnabled() && editableFileOpen() && formattableLanguage()) {
            pendingSaveAfterFormat = true;
            formatCurrentFile();
            return;
        }
        documents.saveCurrentFile();
    }

    function formattablePath(path) {
        const lower = path.toLowerCase();
        return lower.endsWith(".rs") || lower.endsWith(".c")
                || lower.endsWith(".cc") || lower.endsWith(".cpp")
                || lower.endsWith(".cxx") || lower.endsWith(".h")
                || lower.endsWith(".hh") || lower.endsWith(".hpp")
                || lower.endsWith(".hxx");
    }

    function saveAllFiles() {
        if (pendingSaveAllItem !== null || pendingSaveAllQueue.length > 0) {
            return;
        }
        if (!formatOnSaveEnabled()) {
            documents.saveAllFiles();
            return;
        }
        pendingSaveAllQueue = documents.modifiedDocuments();
        continueSaveAll();
    }

    function hasModifiedFiles() {
        return documents.modifiedDocuments().length > 0;
    }

    function continueSaveAll() {
        while (pendingSaveAllQueue.length > 0) {
            const item = pendingSaveAllQueue[0];
            pendingSaveAllQueue = pendingSaveAllQueue.slice(1);
            if (formattablePath(item.path)) {
                pendingSaveAllItem = item;
                formatRequested(item.path, item.content);
                return;
            }
            documents.saveDocumentSnapshot(item.path, item.content, item.content);
        }
        pendingSaveAllItem = null;
    }

    function formatCurrentFile() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        formatRequested(path, surfaceBridge.text());
    }

    function handleFormatResolved(path, text, changed) {
        if (pendingSaveAllItem !== null && pendingSaveAllItem.path === path) {
            const item = pendingSaveAllItem;
            pendingSaveAllItem = null;
            documents.saveDocumentSnapshot(path, item.content, text);
            continueSaveAll();
            return;
        }
        if (editorReady() && path === currentFilePath() && changed) {
            const cursor = editorSurface.cursorPosition;
            editorSurface.text = text;
            editorSurface.cursorPosition = Math.min(cursor, text.length);
        }
        if (pendingSaveAfterFormat) {
            pendingSaveAfterFormat = false;
            documents.saveCurrentFile();
        }
    }

    function applyPathRenameToTabs(from, to) {
        documents.applyPathRenameToTabs(from, to);
    }

    function closeTabsUnderPath(path) {
        documents.closeTabsUnderPath(path);
    }

    function openDiagnostic(file, line, column) {
        documents.openDiagnostic(file, line, column);
    }

    function currentFilePath() {
        return documents.currentFilePath();
    }

    function requestDefinition() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        hoverVisible = false;
        const position = textController.cursorLineColumn();
        definitionRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function requestHover() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        const position = textController.cursorLineColumn();
        hoverRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function requestCompletion() {
        hoverVisible = false;
        completionController.requestCompletion();
    }

    function acceptCompletion() {
        completionController.accept();
    }

    function moveCompletion(delta) {
        completionController.move(delta);
    }

    function refreshSemanticTokens() {
        const path = currentFilePath();
        semanticVersion++;
        if (path === "" || !editorReady()) {
            if (editorReady()) {
                editorSurface.clearSemanticTokens();
            }
            return;
        }
        semanticTokensRequested(path, surfaceBridge.text(), semanticVersion);
    }

    function refreshSyntaxTree() {
        const path = currentFilePath();
        syntaxVersion++;
        if (path === "" || !editorReady()) {
            syntaxLanguage = "plain";
            syntaxHasErrors = false;
            syntaxOutline = [];
            syntaxLocals = [];
            if (editorReady()) {
                editorSurface.setSyntaxSnapshot([], []);
            }
            return;
        }
        syntaxTreeRequested(path, surfaceBridge.text(), syntaxVersion);
    }

    // T1: alterna header/source (clangd). O core valida a linguagem;
    // pedir num arquivo não-C/C++ apenas devolve INVALID_PARAMS.
    function requestSwitchSourceHeader() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        switchSourceHeaderRequested(path, surfaceBridge.text());
    }

    function handleSwitchSourceHeader(path) {
        // path vazio = clangd não achou contraparte; sem primitiva de
        // aviso discreto ainda (radar docs/18), o v1 apenas não navega.
        if (path !== "") {
            documents.openDiagnostic(path, 1, 1);
        }
    }

    function requestUsages() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        hoverVisible = false;
        const position = textController.cursorLineColumn();
        referencesRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function requestCodeActions() {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        const position = textController.cursorLineColumn();
        codeActionsRequested(path, surfaceBridge.text(), position.line, position.column);
    }

    function handleCodeActionsResolved(actions) {
        actionsItemsModel.clear();
        for (let i = 0; i < actions.length; i++) {
            actionsItemsModel.append({
                title: actions[i].title,
                kind: actions[i].kind !== undefined ? actions[i].kind : ""
            });
        }
        actionsIndex = 0;
        actionsVisible = true;
    }

    function moveActions(delta) {
        if (actionsItemsModel.count === 0) {
            return;
        }
        const next = actionsIndex + delta;
        actionsIndex = Math.max(0, Math.min(actionsItemsModel.count - 1, next));
    }

    function applyCodeAction(index) {
        const path = currentFilePath();
        if (!actionsVisible || path === "" || !editorReady()) {
            return;
        }
        if (index < 0 || index >= actionsItemsModel.count) {
            dismissActions();
            return;
        }
        actionsVisible = false;
        codeActionApplyRequested(path, surfaceBridge.text(), index);
    }

    function applySelectedAction() {
        applyCodeAction(actionsIndex);
    }

    function dismissActions() {
        actionsVisible = false;
        actionsItemsModel.clear();
    }

    function handleWorkspaceEditPreview(transactionId, title, files, edits) {
        workspaceEditTransactionId = transactionId;
        workspaceEditTitle = title;
        workspaceEditFiles = files;
        workspaceEditCount = edits;
        workspaceEditError = "";
        workspaceEditPreviewVisible = true;
    }

    function applyWorkspaceEdit() {
        if (workspaceEditTransactionId === "") {
            return;
        }
        workspaceEditError = "";
        workspaceEditApplyRequested(workspaceEditTransactionId);
    }

    function cancelWorkspaceEdit() {
        if (workspaceEditTransactionId === "") {
            resetWorkspaceEditPreview();
            focusEditor();
            return;
        }
        workspaceEditCancelRequested(workspaceEditTransactionId);
    }

    function handleWorkspaceEditApplied(files) {
        resetWorkspaceEditPreview();
        handleRenameApplied(files);
        focusEditor();
    }

    function handleWorkspaceEditCancelled() {
        resetWorkspaceEditPreview();
        focusEditor();
    }

    function resetWorkspaceEditPreview() {
        workspaceEditPreviewVisible = false;
        workspaceEditTransactionId = "";
        workspaceEditTitle = "";
        workspaceEditFiles = [];
        workspaceEditCount = 0;
        workspaceEditError = "";
    }

    function restoreSession(files, activeFile) {
        for (let i = 0; i < files.length; i++) {
            if (files[i] !== activeFile) {
                readFileRequested(files[i]);
            }
        }
        // A ativa vai por ultimo: o core responde em ordem e cada load
        // seleciona a propria aba, entao a ultima carregada fica ativa.
        for (let i = 0; i < files.length; i++) {
            if (files[i] === activeFile) {
                readFileRequested(files[i]);
                break;
            }
        }
    }

    function scheduleSessionSave() {
        if (workspaceRoot === "") {
            return;
        }
        sessionSaveDebounce.restart();
    }

    onCurrentTabChanged: scheduleSessionSave()

    function openRenameDialog() {
        if (currentFilePath() === "") {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        renameError = "";
        renameDialogVisible = true;
        renameDialogOpenRequested(textController.currentWord());
    }

    function confirmRename(name) {
        const path = currentFilePath();
        if (path === "" || !editorReady()) {
            return;
        }
        if (name === "") {
            renameError = qsTr("Informe um novo nome.");
            return;
        }
        for (let i = 0; i < filesModel.count; i++) {
            if (i !== currentTab && filesModel.get(i).modified) {
                renameError =
                        qsTr("Salve as outras abas modificadas antes de renomear.");
                return;
            }
        }
        const position = textController.cursorLineColumn();
        renameDialogVisible = false;
        focusEditor();
        renameRequested(path, surfaceBridge.text(), position.line, position.column, name);
    }

    function editableFileOpen() {
        return currentFilePath() !== "" && editorReady();
    }

    function duplicateLine() {
        if (editableFileOpen()) {
            textController.duplicateLineOrSelection();
        }
    }

    function moveLineUp() {
        if (editableFileOpen()) {
            textController.moveLines(-1);
        }
    }

    function moveLineDown() {
        if (editableFileOpen()) {
            textController.moveLines(1);
        }
    }

    function deleteLine() {
        if (editableFileOpen()) {
            textController.deleteCurrentLine();
        }
    }

    function commentTokenFor(language) {
        if (language === "rust" || language === "cpp" || language === "js") {
            return "//";
        }
        if (language === "python" || language === "shell" || language === "cmake"
                || language === "toml") {
            return "#";
        }
        return "";
    }

    function toggleComment() {
        if (!editableFileOpen()) {
            return;
        }
        textController.toggleLineComment(commentTokenFor(editorSurface.language));
    }

    function openGoToLine() {
        if (!editableFileOpen()) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        goToLineVisible = true;
        goToLineDialogOpenRequested(String(textController.cursorLineColumn().line));
    }

    function confirmGoToLine(value) {
        goToLineVisible = false;
        focusEditor();
        const match = value.trim().match(/^(\d+)(?:[:,](\d+))?$/);
        if (match === null || !editableFileOpen()) {
            return;
        }
        const column = match[2] !== undefined ? parseInt(match[2]) : 1;
        textController.goToLine(parseInt(match[1]), column);
    }

    function cancelGoToLine() {
        goToLineVisible = false;
        focusEditor();
    }

    function indentEditorSelection() {
        textController.indentSelection();
    }

    function unindentEditorSelection() {
        textController.unindentSelection();
    }

    function insertEditorNewline() {
        textController.insertNewline();
    }

    function insertEditorCloserBrace() {
        textController.insertCloserBrace();
    }

    function editorSmartHome(extendSelection) {
        textController.smartHome(extendSelection);
    }

    function expandSelection() {
        if (editableFileOpen()) {
            textController.expandSelection();
        }
    }

    // T6: salta o cursor para o próximo/anterior diagnóstico do arquivo
    // (com wrap). A busca vem do DiagnosticsController; o salto reusa o
    // goToLine do textController.
    function goToNextDiagnostic() {
        jumpToDiagnostic(true);
    }

    function goToPrevDiagnostic() {
        jumpToDiagnostic(false);
    }

    function jumpToDiagnostic(forward) {
        if (!editableFileOpen() || diagnosticsController === null) {
            return;
        }
        const position = textController.cursorLineColumn();
        const target = forward
                ? diagnosticsController.nextDiagnostic(position.line, position.column)
                : diagnosticsController.prevDiagnostic(position.line, position.column);
        if (target !== null) {
            textController.goToLine(target.line, target.column);
            focusEditor();
        }
    }

    function shrinkSelection() {
        if (editableFileOpen()) {
            textController.shrinkSelection();
        }
    }

    function handleTextEdited(text) {
        if (!surfaceBridge.loadingText && documents.markCurrentModified(text)) {
            // Invalida imediatamente respostas iniciadas para o snapshot
            // anterior. Semantic tokens antigos também saem da pintura até o
            // LSP responder; Tree-sitter continua como fallback estrutural.
            semanticVersion++;
            syntaxVersion++;
            editorSurface.clearSemanticTokens();
            hoverVisible = false;
            changeDebounce.restart();
            syntaxDebounce.restart();
            autosaveDebounce.restart(); // M-S1: persiste o buffer sujo em breve.
            completionController.handleTextEdited();
            // D1b: o texto mudou → os offsets dos matches envelheceram.
            // Debounce para não revarrer o arquivo a cada tecla.
            if (findController.barVisible) {
                findRecomputeDebounce.restart();
            }
        }
    }

    // M-S1 (docs/23): rascunhos aguardando o load do disco para sobrepor.
    property var pendingDraftContent: ({})

    function handleFileLoaded(path, content) {
        documents.handleFileLoaded(path, content);
        // Se há rascunho não salvo pendente para este arquivo, sobrepõe o
        // conteúdo do disco (aba fica MODIFICADA) — o savedContent segue o
        // disco, então salvar/reverter continuam corretos.
        if (pendingDraftContent[path] !== undefined) {
            const draft = pendingDraftContent[path];
            const map = pendingDraftContent;
            delete map[path];
            pendingDraftContent = map;
            documents.applyDraftOverlay(path, draft);
        }
    }

    // M-S1: abre cada arquivo com rascunho e sobrepõe o buffer não salvo.
    function restoreDrafts(drafts) {
        for (let i = 0; i < drafts.length; i++) {
            const map = pendingDraftContent;
            map[drafts[i].path] = drafts[i].content;
            pendingDraftContent = map;
            readFileRequested(drafts[i].path);
        }
    }

    function handleFileSaved(path) {
        documents.handleFileSaved(path);
    }

    function handleExternalChanges(changes) {
        documents.handleExternalChanges(changes);
    }

    function handleFileSaveFailed(path, message) {
        documents.handleFileSaveFailed(path, message);
    }

    function handleFileWatchFailed(message) {
        watchError = message;
    }

    function reloadExternalFile() {
        const path = documents.reloadExternalCurrent();
        if (path !== "") {
            draftClearRequested(path);
        }
    }

    function keepLocalFile() {
        documents.keepLocalAfterExternalChange();
    }

    function dismissWatchError() {
        watchError = "";
    }

    function handleHoverResolved(content) {
        const text = content.trim();
        if (text === "") {
            hoverVisible = false;
            hoverText = "";
            return;
        }
        hoverText = text;
        hoverVisible = true;
        hoverHideTimer.restart();
    }

    function handleSemanticTokensResolved(path, version, tokens) {
        if (path !== currentFilePath() || Number(version) !== semanticVersion
                || !editorReady()) {
            return;
        }
        editorSurface.setSemanticTokens(tokens);
    }

    function handleSyntaxTreeResolved(path, version, language, hasErrors,
                                      highlights, foldingRanges, outline, locals) {
        if (path !== currentFilePath() || Number(version) !== syntaxVersion
                || !editorReady()) {
            return;
        }
        syntaxLanguage = language;
        syntaxHasErrors = hasErrors;
        syntaxOutline = outline;
        syntaxLocals = locals;
        editorSurface.setSyntaxSnapshot(highlights, foldingRanges);
    }

    function openOutlineItem(line, column) {
        if (!editableFileOpen()) {
            return;
        }
        textController.goToLine(Number(line), Number(column));
        focusEditor();
    }

    function handleCompletionResolved(items, isIncomplete) {
        completionController.handleResolved(items, isIncomplete);
    }

    function handleReferencesResolved(references) {
        usagesItemsModel.clear();
        for (let i = 0; i < references.length; i++) {
            const usage = references[i];
            const line = usage.line !== undefined ? Number(usage.line) : 1;
            usagesItemsModel.append({
                path: usage.path,
                line: line,
                column: usage.column !== undefined ? Number(usage.column) : 1,
                display: documents.relativeToRoot(usage.path) + ":" + line
            });
        }
        usagesVisible = usagesItemsModel.count > 0;
    }

    function handleRenameApplied(files) {
        documents.handleRenameApplied(files);
    }

    function handleRequestFailed(method, message) {
        if (method === "lsp.hover") {
            hoverVisible = false;
        }
        if (method === "lsp.completion") {
            completionController.handleFailed();
        }
        if (method === "lsp.rename") {
            renameError = message;
            renameDialogVisible = true;
        }
        if (method === "lsp.codeActions" || method === "lsp.applyCodeAction") {
            dismissActions();
        }
        if (method === "lsp.workspaceEdit.apply") {
            workspaceEditError = message;
            workspaceEditPreviewVisible = workspaceEditTransactionId !== "";
        }
        if (method === "lsp.workspaceEdit.cancel") {
            resetWorkspaceEditPreview();
        }
        // Format-on-save: se o format falhou (servidor ausente/timeout),
        // salva assim mesmo — não trava o Ctrl+S do usuário.
        if (method === "format.text" && pendingSaveAfterFormat) {
            pendingSaveAfterFormat = false;
            documents.saveCurrentFile();
        }
        if (method === "format.text" && pendingSaveAllItem !== null) {
            const item = pendingSaveAllItem;
            pendingSaveAllItem = null;
            documents.saveDocumentSnapshot(item.path, item.content, item.content);
            continueSaveAll();
        }
    }

    Timer {
        id: changeDebounce

        interval: 600
        repeat: false
        onTriggered: {
            const path = root.currentFilePath();
            if (path !== "" && root.editorReady()) {
                root.fileChangedNotificationRequested(path, surfaceBridge.text());
                root.refreshSemanticTokens();
            }
        }
    }

    Timer {
        id: syntaxDebounce

        interval: 180
        repeat: false
        onTriggered: root.refreshSyntaxTree()
    }

    Timer {
        id: hoverHideTimer

        interval: 9000
        repeat: false
        onTriggered: root.hoverVisible = false
    }

    // D1b: revarredura dos matches após a edição parar (o replace já
    // recomputa sozinho; isto cobre a digitação normal com a barra aberta).
    Timer {
        id: findRecomputeDebounce

        interval: 200
        repeat: false
        onTriggered: {
            if (findController.barVisible) {
                findController.recompute(findController.cursorOffset());
            }
        }
    }

    // M-S1 (docs/23): autosave do buffer sujo na store local (rede de
    // segurança). Debounce após a última edição; só persiste se modificado.
    Timer {
        id: autosaveDebounce

        interval: 1500
        repeat: false
        onTriggered: {
            const path = root.currentFilePath();
            if (path !== "" && root.editorReady() && documents.currentIsModified()) {
                root.draftSaveRequested(path, surfaceBridge.text());
            }
        }
    }

    Timer {
        id: sessionSaveDebounce

        interval: 1200
        repeat: false
        onTriggered: {
            if (root.workspaceRoot === "") {
                return;
            }
            const files = [];
            for (let i = 0; i < root.filesModel.count; i++) {
                files.push(root.filesModel.get(i).path);
            }
            root.saveSessionRequested(files, root.currentFilePath());
        }
    }

    Connections {
        target: root.filesModel

        function onCountChanged() {
            root.scheduleSessionSave();
        }
    }
}
