pragma ComponentBehavior: Bound
import QtQuick

// INTELIGÊNCIA DE LINGUAGEM do editor: o que o servidor (LSP) e o Tree-sitter
// respondem, e o estado de tela que isso produz.
//
// # Por que este arquivo nasceu em 2026-09-02
//
// Ele saiu do `EditorController.qml`, que tinha 1.070 linhas contra um limite
// de 400 e era o maior débito do repositório (`LEITURA_TECNICA` §4: *"o maior
// débito bloqueia por área"*). O corte NÃO foi por tamanho — foi por
// responsabilidade, que é o critério da `ARCHITECTURE.md` §4 regra 9. O
// `EditorController` já tinha quatro subcontrollers (documentos, texto,
// completion, find) e **implementava a camada de linguagem inteira inline**:
// 12 dos seus 23 sinais, 26 funções, dois models e dois timers eram disto.
//
// # A fronteira, dita em uma frase
//
// Aqui mora o que a IDE **pergunta ao servidor sobre o símbolo sob o cursor** e
// o que ela **mostra com a resposta**. O que se faz com o TEXTO é do
// `EditorTextController`; que arquivo está aberto e o que está sujo é do
// `EditorDocumentController`; a lista de completion é do
// `EditorCompletionController`; o realce é do `EditorHighlightController`. Este
// controller não escreve arquivo e não move cursor: ele pede, guarda a resposta
// e acende a tela.
//
// # O que este arquivo NÃO faz, e onde isso mora
//
// O REALCE do documento — Tree-sitter e semantic tokens, com os dois relógios
// de versão — saiu para o `EditorHighlightController` no mesmo dia, quando a
// catraca cobrou este arquivo ainda em 452 linhas. São ritmos diferentes sobre
// o mesmo buffer: aqui se pergunta sobre o SÍMBOLO sob o cursor (gesto do
// usuário, resposta pontual, popup ou diálogo); lá se persegue cada tecla
// digitada para pintar o texto inteiro.
//
// # Nada aqui fala com o CoreClient
//
// Como todo controller deste projeto, ele PEDE por sinal. Quem leva ao core é o
// `EditorRequestRouter`, e quem traz a resposta é o `EditorEventRouter`.
Item {
    id: root

    // Colaboradores injetados pelo `EditorController`, que continua sendo o
    // composition root do editor. Nenhum deles é criado aqui: quem cria é quem
    // compõe, e assim "onde X é ligado" tem uma resposta só.
    property var surfaceBridge: null
    property var documentController: null
    property var textController: null
    property var completionController: null
    property var editorSurface: null

    // --- Hover -------------------------------------------------------------
    property bool hoverVisible: false
    property string hoverText: ""

    // --- Usos (references) -------------------------------------------------
    property alias usagesModel: usagesItems
    property bool usagesVisible: false

    // --- Rename ------------------------------------------------------------
    property bool renameDialogVisible: false
    property string renameError: ""

    // --- Code actions ------------------------------------------------------
    property alias actionsModel: actionsItems
    property bool actionsVisible: false
    property int actionsIndex: 0

    // --- WorkspaceEdit (preview transacional de rename multi-arquivo) ------
    property bool workspaceEditPreviewVisible: false
    property string workspaceEditTransactionId: ""
    property string workspaceEditTitle: ""
    property var workspaceEditFiles: []
    property int workspaceEditCount: 0
    property string workspaceEditError: ""

    signal switchSourceHeaderRequested(string path, string content)
    signal definitionRequested(string path, string content, int line, int column)
    signal hoverRequested(string path, string content, int line, int column)
    signal referencesRequested(string path, string content, int line, int column)
    signal renameRequested(string path, string content, int line, int column,
                           string newName)
    signal renameDialogOpenRequested(string currentName)
    signal codeActionsRequested(string path, string content, int line, int column)
    signal codeActionApplyRequested(string path, string content, int actionIndex)
    signal workspaceEditApplyRequested(string transactionId)
    signal workspaceEditCancelRequested(string transactionId)
    // O editor precisa voltar a receber o foco depois de um diálogo fechar; o
    // dono do foco é o composition root, não este controller.
    signal focusEditorRequested()

    visible: false

    ListModel {
        id: usagesItems
    }

    ListModel {
        id: actionsItems
    }

    function ready() {
        return surfaceBridge !== null && surfaceBridge.ready()
                && documentController !== null;
    }

    function currentPath() {
        return documentController === null ? "" : documentController.currentFilePath();
    }

    // Um pedido posicional só faz sentido com arquivo aberto e editor vivo;
    // esta guarda evita 8 repetições do mesmo `if` espalhadas pelo arquivo.
    function positionOrNull() {
        const path = currentPath();
        if (path === "" || !ready()) {
            return null;
        }
        return { path: path, content: surfaceBridge.text(),
                 position: textController.cursorLineColumn() };
    }

    function requestDefinition() {
        const context = positionOrNull();
        if (context === null) {
            return;
        }
        hoverVisible = false;
        definitionRequested(context.path, context.content,
                            context.position.line, context.position.column);
    }

    function requestHover() {
        const context = positionOrNull();
        if (context === null) {
            return;
        }
        hoverRequested(context.path, context.content,
                       context.position.line, context.position.column);
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

    function requestUsages() {
        const context = positionOrNull();
        if (context === null) {
            return;
        }
        hoverVisible = false;
        referencesRequested(context.path, context.content,
                            context.position.line, context.position.column);
    }

    function handleReferencesResolved(references) {
        usagesItems.clear();
        for (let index = 0; index < references.length; index++) {
            const usage = references[index];
            const line = usage.line !== undefined ? Number(usage.line) : 1;
            usagesItems.append({
                path: usage.path,
                line: line,
                column: usage.column !== undefined ? Number(usage.column) : 1,
                display: documentController.relativeToRoot(usage.path) + ":" + line
            });
        }
        usagesVisible = usagesItems.count > 0;
    }

    // T1: alterna header/source (clangd). O core valida a linguagem; pedir num
    // arquivo não-C/C++ apenas devolve INVALID_PARAMS.
    function requestSwitchSourceHeader() {
        const path = currentPath();
        if (path === "" || !ready()) {
            return;
        }
        switchSourceHeaderRequested(path, surfaceBridge.text());
    }

    function handleSwitchSourceHeader(path) {
        // path vazio = clangd não achou contraparte; sem primitiva de aviso
        // discreto ainda (radar DocsPrivate/diario/18), o v1 apenas não navega.
        if (path !== "") {
            documentController.openDiagnostic(path, 1, 1);
        }
    }

    function requestCodeActions() {
        const context = positionOrNull();
        if (context === null) {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        codeActionsRequested(context.path, context.content,
                             context.position.line, context.position.column);
    }

    function handleCodeActionsResolved(actions) {
        actionsItems.clear();
        for (let index = 0; index < actions.length; index++) {
            actionsItems.append({
                title: actions[index].title,
                kind: actions[index].kind !== undefined ? actions[index].kind : ""
            });
        }
        actionsIndex = 0;
        actionsVisible = true;
    }

    function moveActions(delta) {
        if (actionsItems.count === 0) {
            return;
        }
        actionsIndex = Math.max(0, Math.min(actionsItems.count - 1, actionsIndex + delta));
    }

    function applyCodeAction(index) {
        const path = currentPath();
        if (!actionsVisible || path === "" || !ready()) {
            return;
        }
        if (index < 0 || index >= actionsItems.count) {
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
        actionsItems.clear();
    }

    function openRenameDialog() {
        if (currentPath() === "") {
            return;
        }
        completionController.dismiss();
        hoverVisible = false;
        renameError = "";
        renameDialogVisible = true;
        renameDialogOpenRequested(textController.currentWord());
    }

    function confirmRename(name) {
        const path = currentPath();
        if (path === "" || !ready()) {
            return;
        }
        if (name === "") {
            renameError = qsTr("Informe um novo nome.");
            return;
        }
        // Rename atravessa arquivos: uma aba suja em outro arquivo tornaria o
        // WorkspaceEdit obsoleto no meio da transação. Recusar com motivo é
        // melhor que aplicar por cima do que o usuário não salvou.
        const files = documentController.filesModel;
        for (let index = 0; index < files.count; index++) {
            if (index !== documentController.currentTab && files.get(index).modified) {
                renameError = qsTr("Salve as outras abas modificadas antes de renomear.");
                return;
            }
        }
        const position = textController.cursorLineColumn();
        renameDialogVisible = false;
        focusEditorRequested();
        renameRequested(path, surfaceBridge.text(), position.line, position.column, name);
    }

    function handleRenameApplied(files) {
        documentController.handleRenameApplied(files);
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
            focusEditorRequested();
            return;
        }
        workspaceEditCancelRequested(workspaceEditTransactionId);
    }

    function handleWorkspaceEditApplied(files) {
        resetWorkspaceEditPreview();
        handleRenameApplied(files);
        focusEditorRequested();
    }

    function handleWorkspaceEditCancelled() {
        resetWorkspaceEditPreview();
        focusEditorRequested();
    }

    function resetWorkspaceEditPreview() {
        workspaceEditPreviewVisible = false;
        workspaceEditTransactionId = "";
        workspaceEditTitle = "";
        workspaceEditFiles = [];
        workspaceEditCount = 0;
        workspaceEditError = "";
    }

    // Fecha tudo o que este controller acende. Chamado quando o workspace
    // troca: estado de linguagem do projeto anterior na tela é mentira.
    function clear() {
        hoverText = "";
        hoverVisible = false;
        usagesVisible = false;
        usagesItems.clear();
        renameDialogVisible = false;
        renameError = "";
        resetWorkspaceEditPreview();
        dismissActions();
    }

    // A RECUSA do servidor é informação de produto, não ruído de log: "salve as
    // outras abas" e "o arquivo mudou" são respostas à pergunta do usuário.
    function handleRequestFailed(method, message) {
        if (method === "lsp.hover") {
            hoverVisible = false;
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
    }

    Timer {
        id: hoverHideTimer

        interval: 9000
        repeat: false
        onTriggered: root.hoverVisible = false
    }
}
