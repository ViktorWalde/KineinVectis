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
    property alias usagesModel: language.usagesModel
    property alias currentTab: documents.currentTab
    property alias currentDocId: documents.currentDocId
    property alias externalConflict: documents.currentExternalConflict
    property alias externalDeleted: documents.currentExternalDeleted
    property alias externalMessage: documents.currentExternalMessage
    property string watchError: ""
    property alias loadingEditorText: surfaceBridge.loadingText
    property alias cursorSummary: surfaceBridge.cursorSummary
    // D1 (DocsPublic/roadmaps/24): alias para a property PRÓPRIA do controller, nunca para o
    // `visible` do Item — este EditorController é invisível (é controller), e
    // `Item.visible` de um filho lê a visibilidade EFETIVA, que fica presa em
    // false sob pai invisível. Era essa a causa do popup nunca abrir.
    property alias completionVisible: completionController.popupVisible
    property alias completionIndex: completionController.index
    property alias hoverVisible: language.hoverVisible
    property alias hoverText: language.hoverText
    property alias usagesVisible: language.usagesVisible
    property alias renameDialogVisible: language.renameDialogVisible
    property alias renameError: language.renameError
    property alias workspaceEditPreviewVisible: language.workspaceEditPreviewVisible
    property alias workspaceEditTransactionId: language.workspaceEditTransactionId
    property alias workspaceEditTitle: language.workspaceEditTitle
    property alias workspaceEditFiles: language.workspaceEditFiles
    property alias workspaceEditCount: language.workspaceEditCount
    property alias workspaceEditError: language.workspaceEditError
    property alias actionsModel: language.actionsModel
    property alias actionsVisible: language.actionsVisible
    property alias actionsIndex: language.actionsIndex
    property bool goToLineVisible: false
    // D1b (DocsPublic/roadmaps/24): Find/Replace no arquivo. Mesma regra do D1 — alias para
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
    property alias syntaxVersion: highlight.syntaxVersion
    property alias semanticVersion: highlight.semanticVersion
    property alias syntaxLanguage: highlight.syntaxLanguage
    property alias syntaxHasErrors: highlight.syntaxHasErrors
    property alias syntaxOutline: highlight.syntaxOutline
    property alias syntaxLocals: highlight.syntaxLocals
    // O dono da camada de linguagem, exposto por nome: os roteadores IPC falam
    // com ELE, nao com um repasse daqui. Fachada de pass-through foi
    // justamente o que engordou este arquivo ate 1.070 linhas.
    readonly property alias language: language
    readonly property alias highlight: highlight

    signal readFileRequested(string path)
    signal writeFileRequested(string path, string content, string expectedContent)
    // M-S1 (DocsPublic/seguranca/23): autosave/limpeza de rascunho não salvo (rede de segurança).
    signal draftSaveRequested(string path, string content)
    signal draftClearRequested(string path)
    signal formatRequested(string path, string content)
    signal fileChangedNotificationRequested(string path, string content)
    signal completionRequested(string path, string content, int line, int column)
    signal saveSessionRequested(var files, string activeFile)
    signal goToLineDialogOpenRequested(string prefill)
    // D1b: a barra abriu — a UI precisa focar o campo de busca.
    signal findBarOpenRequested()

    visible: false

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

    // Camada de linguagem (LSP + Tree-sitter). Extraida em 2026-09-02: era o
    // que este arquivo AINDA implementava inline, enquanto ja delegava
    // documentos, texto, completion e find.
    EditorLanguageController {
        id: language

        surfaceBridge: surfaceBridge
        documentController: documents
        textController: textController
        completionController: completionController
        editorSurface: root.editorSurface
        onFocusEditorRequested: root.focusEditor()
    }

    // Realce: Tree-sitter e semantic tokens, com os dois relogios de versao.
    // Separado do `language` porque persegue a DIGITACAO, e nao um gesto.
    EditorHighlightController {
        id: highlight

        surfaceBridge: surfaceBridge
        documentController: documents
        editorSurface: root.editorSurface
    }

    // Formatacao e a maquina de estados do format-on-save.
    // O que o editor lembra entre sessoes: abas abertas e rascunho nao salvo.
    EditorPersistenceController {
        id: persistence

        workspaceRoot: root.workspaceRoot
        surfaceBridge: surfaceBridge
        documentController: documents
        filesModel: documents.filesModel
        autoSaveEnabled: root.settingsController === null ? true : root.settingsController.autoSave
        onReadFileRequested: path => root.readFileRequested(path)
        onDraftSaveRequested: (path, content) => root.draftSaveRequested(path, content)
        onSaveSessionRequested: (files, activeFile) => root.saveSessionRequested(files, activeFile)
        // Autosave (F3): o mesmo caminho do Ctrl+S, so' com buffer sujo.
        onAutoSaveRequested: if (documents.currentIsModified()) root.saveCurrentFile()
    }

    EditorFormatController {
        id: format

        surfaceBridge: surfaceBridge
        documentController: documents
        completionController: completionController
        settingsController: root.settingsController
        editorSurface: root.editorSurface
        onFormatRequested: function(path, content) {
            root.formatRequested(path, content);
        }
        onDismissOverlaysRequested: {
            completionController.dismiss();
            language.hoverVisible = false;
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

    // Cada dono limpa o SEU estado; este arquivo so limpa o que e dele
    // (format-on-save pendente e o aviso do watcher).
    function clear() {
        documents.clear();
        completionController.clear();
        findController.close();
        language.clear();
        highlight.clear();
        format.clear();
        watchError = "";
    }

    function storeCurrentEditor() {
        documents.storeCurrentEditor();
    }

    function selectDocument(docId) {
        completionController.dismiss();
        persistence.flushAutoSave(); // F3: a aba que sai vai ao disco antes.
        documents.selectDocument(docId);
        // D1b: o buffer trocou — os offsets dos matches eram do texto ANTIGO.
        // Revarre no arquivo novo (mantendo o termo, como VS Code faz).
        if (findController.barVisible) {
            findController.recompute(0);
        }
    }

    function closeDocument(docId) {
        // M-S1: fechar a aba é uma decisão explícita → o rascunho não é mais
        // necessário (só sobrevive a CRASH). Limpa antes de fechar. O caminho
        // vem do DOCUMENTO (V5), e não de uma posição que já pode ter mudado.
        const path = documents.pathOfDocument(docId);
        if (path !== "") {
            draftClearRequested(path);
        }
        documents.closeDocument(docId);
    }

    // --- Formatação e format-on-save -----------------------------------
    //
    // A maquina de estados (fila do salvar-tudo, salvar depois do format e a
    // perna do FRACASSO) mora no `EditorFormatController`. Aqui so o repasse.
    property alias pendingSaveAfterFormat: format.pendingSaveAfterFormat
    property alias formatterExtensions: format.formatterExtensions

    function applyFormatCapabilities(formatters) {
        format.applyFormatCapabilities(formatters);
    }

    function formatOnSaveEnabled() {
        return format.formatOnSaveEnabled();
    }

    function formattablePath(path) {
        return format.formattablePath(path);
    }

    function saveCurrentFile() {
        format.saveCurrentFile();
    }

    function saveAllFiles() {
        format.saveAllFiles();
    }

    function hasModifiedFiles() {
        return format.hasModifiedFiles();
    }

    function continueSaveAll() {
        format.continueSaveAll();
    }

    function formatCurrentFile() {
        format.formatCurrentFile();
    }

    function handleFormatResolved(path, text, changed) {
        format.handleFormatResolved(path, text, changed);
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

    // --- Camada de linguagem -------------------------------------------
    //
    // Repasse DELIBERADO e curto: os chamadores externos (paleta de comandos,
    // atalhos globais, EditorPane) falam com o editor, e quebrar isso agora
    // trocaria um arquivo grande por dezenas de call sites reescritos sem
    // ninguem ficar mais claro. O que saiu daqui foi a IMPLEMENTACAO — 26
    // funcoes, 12 sinais, dois models e dois timers —, nao o nome do gesto.
    // Quem quiser o dono direto usa `editorController.language`.
    function requestDefinition() {
        language.requestDefinition();
    }

    function requestHover() {
        language.requestHover();
    }

    function handleHoverResolved(content) {
        language.handleHoverResolved(content);
    }

    function requestUsages() {
        language.requestUsages();
    }

    function handleReferencesResolved(references) {
        language.handleReferencesResolved(references);
    }

    function requestSwitchSourceHeader() {
        language.requestSwitchSourceHeader();
    }

    function handleSwitchSourceHeader(path) {
        language.handleSwitchSourceHeader(path);
    }

    function requestCodeActions() {
        language.requestCodeActions();
    }

    function handleCodeActionsResolved(actions) {
        language.handleCodeActionsResolved(actions);
    }

    function moveActions(delta) {
        language.moveActions(delta);
    }

    function applyCodeAction(index) {
        language.applyCodeAction(index);
    }

    function applySelectedAction() {
        language.applySelectedAction();
    }

    function dismissActions() {
        language.dismissActions();
    }

    function openRenameDialog() {
        language.openRenameDialog();
    }

    function confirmRename(name) {
        language.confirmRename(name);
    }

    function handleRenameApplied(files) {
        language.handleRenameApplied(files);
    }

    function handleWorkspaceEditPreview(transactionId, title, files, edits) {
        language.handleWorkspaceEditPreview(transactionId, title, files, edits);
    }

    function applyWorkspaceEdit() {
        language.applyWorkspaceEdit();
    }

    function cancelWorkspaceEdit() {
        language.cancelWorkspaceEdit();
    }

    function handleWorkspaceEditApplied(files) {
        language.handleWorkspaceEditApplied(files);
    }

    function handleWorkspaceEditCancelled() {
        language.handleWorkspaceEditCancelled();
    }

    function resetWorkspaceEditPreview() {
        language.resetWorkspaceEditPreview();
    }

    function refreshSemanticTokens() {
        highlight.refreshSemanticTokens();
    }

    function handleSemanticTokensResolved(path, version, tokens) {
        highlight.handleSemanticTokensResolved(path, version, tokens);
    }

    function refreshSyntaxTree() {
        highlight.refreshSyntaxTree();
    }

    function handleSyntaxTreeResolved(path, version, syntaxLanguageId, hasErrors,
                                      highlights, foldingRanges, outline, locals) {
        highlight.handleSyntaxTreeResolved(path, version, syntaxLanguageId, hasErrors,
                                           highlights, foldingRanges, outline, locals);
    }

    function requestCompletion() {
        language.hoverVisible = false;
        completionController.requestCompletion();
    }

    function acceptCompletion() {
        completionController.accept();
    }

    function moveCompletion(delta) {
        completionController.move(delta);
    }

    function handleCompletionResolved(items, isIncomplete) {
        completionController.handleResolved(items, isIncomplete);
    }

    // --- Sessao e rascunho ---------------------------------------------
    //
    // As duas redes (abas abertas e buffer nao salvo) tem dono proprio no
    // `EditorPersistenceController`, que explica no topo por que elas sao
    // diferentes: sessao sobrevive a fechamento normal, rascunho so a CRASH.
    function restoreSession(files, activeFile) {
        persistence.restoreSession(files, activeFile);
    }

    function scheduleSessionSave() {
        persistence.scheduleSessionSave();
    }

    onCurrentTabChanged: persistence.scheduleSessionSave()

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
            // Cada dono reage a edicao com o que e dele. Este arquivo so
            // ORQUESTRA: quem invalida realce e o `highlight`, quem esconde o
            // hover e o `language`, quem agenda rascunho e o `persistence`.
            highlight.invalidateForEdit();
            language.hoverVisible = false;
            changeDebounce.restart();
            persistence.scheduleDraftSave(); // M-S1: rascunho do buffer sujo.
            completionController.handleTextEdited();
            // D1b: o texto mudou → os offsets dos matches envelheceram.
            // Debounce para não revarrer o arquivo a cada tecla.
            if (findController.barVisible) {
                findRecomputeDebounce.restart();
            }
        }
    }

    // M-S1 (DocsPublic/seguranca/23): rascunhos aguardando o load do disco para sobrepor.
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

    function openOutlineItem(line, column) {
        if (!editableFileOpen()) {
            return;
        }
        textController.goToLine(Number(line), Number(column));
        focusEditor();
    }

    // A recusa do core chega a UM ponto e e distribuida por dono: a camada de
    // linguagem trata os `lsp.*`, o completion trata o dele, e o que sobra —
    // format-on-save — e desta casa, porque envolve o SALVAR.
    function handleRequestFailed(method, message) {
        language.handleRequestFailed(method, message);
        if (method === "lsp.completion") {
            completionController.handleFailed();
        }
        if (method === "format.text") {
            format.handleFormatFailed();
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
}
