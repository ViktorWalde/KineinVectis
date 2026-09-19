import QtQuick

// Estado git da UI (fatia M3.1): branch/contadores para a status bar e o
// mapa path→estado que colore a árvore. O core é stateless — este
// controller decide QUANDO consultar (open/save/fs-ops/manual).
Item {
    id: root

    property string workspaceRoot: ""
    property bool repo: false
    property string branchLabel: ""
    property int aheadCount: 0
    property int behindCount: 0
    property int changeCount: 0
    // path ABSOLUTO -> kind; a revisão força os bindings da árvore a reavaliarem.
    property var gitKinds: ({})
    property int revision: 0
    // Diff do arquivo ativo no editor: linha→kind para as marcas da gutter.
    property string activeDiffPath: ""
    property var diffLineKinds: ({})
    property int diffRevision: 0
    property bool diffDialogVisible: false
    property string diffDialogPath: ""
    property string diffDialogText: ""
    property bool diffDialogTracked: true
    property bool diffDialogLoading: false
    property alias changesModel: gitChangesModel
    property int stagedCount: 0
    property string lastMutationError: ""
    property bool discardDialogVisible: false
    property string discardDialogPath: ""
    property string discardDialogAbsPath: ""
    // Blame e log moram no GitHistoryController (o PASSADO); estes alias
    // existem para os pontos de leitura ja' escritos fora daqui.
    property alias blameVisible: historyController.blameVisible
    property alias blamePath: historyController.blamePath
    property alias blameLineAnnotations: historyController.blameLineAnnotations
    property alias blameRevision: historyController.blameRevision
    // O historico (vista da aba Git): o modelo, a vista, o carregando, as raias.
    property alias historyModel: historyController.historyModel
    property alias historyVisible: historyController.historyVisible
    property alias historyLoading: historyController.historyLoading
    property alias historyLaneCount: historyController.laneCount
    property alias historyFilterText: historyController.filterText
    property alias historyLogRef: historyController.logRef
    property alias branchesModel: gitBranchesModel
    property bool branchMenuVisible: false
    property bool remoteOperationRunning: false
    // O painel da direita da HUD (2026-09-18): dono proprio, filho deste.
    readonly property alias inspector: inspectorController
    // Reescrever o ultimo commit (git commit --amend) no proximo Commit.
    property bool amend: false

    signal statusRequested()
    signal fileDiffRequested(string path)
    signal stageRequested(var paths)
    signal unstageRequested(var paths)
    signal discardRequested(var paths)
    signal commitRequested(string message, bool amend)
    signal blameRequested(string path)
    signal logRequested(string ref)
    signal commitDiffRequested(string sha)
    signal branchesRequested()
    signal checkoutRequested(string branch)
    signal branchCreateRequested(string name)
    signal pullRequested()
    signal pushRequested()
    signal stashRequested(string action, string message)

    visible: false

    ListModel {
        id: gitChangesModel
    }

    ListModel {
        id: gitBranchesModel
    }

    GitRules { id: gitRules }

    GitInspectorController {
        id: inspectorController

        onFileDiffWanted: function(path) { root.fileDiffRequested(path); }
        onCommitDiffWanted: function(sha) { root.commitDiffRequested(sha); }
    }

    GitHistoryController {
        id: historyController

        // Reemite para NAO mudar o contrato: o GitRequestRouter continua com um
        // `Connections { target: gitController }` so', sem conhecer internals.
        onBlameRequested: function(path) { root.blameRequested(path); }
        onLogRequested: function(ref) { root.logRequested(ref); }
    }

    // Delegacoes para o GitHistoryController (deliberadas: 11 pontos de
    // chamada em 5 arquivos). A regra e' NAO crescer esta lista — dono novo
    // no historico entra por `historyController`, nao por mais um repasse.
    function toggleBlame(path) { historyController.toggleBlame(path); }
    function hideBlame() { historyController.hideBlame(); }
    function requestBlameFor(path) { historyController.requestBlameFor(path); }
    function handleBlame(path, isRepo, tracked, groups) {
        historyController.handleBlame(path, isRepo, tracked, groups);
    }
    function openHistory() { historyController.openHistory(); }
    function showChanges() { historyController.showChanges(); }
    function refreshHistory() { historyController.refreshHistory(); }
    function handleLog(isRepo, entries) { historyController.handleLog(isRepo, entries); }
    function historyEntry(sha) { return historyController.entry(sha); }
    function setHistoryFilter(text) { historyController.setFilterText(text); }
    function setHistoryRef(ref) { historyController.setLogRef(ref); }

    function refresh() {
        if (workspaceRoot === "") {
            return;
        }
        statusRequested();
        if (activeDiffPath !== "") {
            fileDiffRequested(activeDiffPath);
        }
    }

    // Chamado na troca de aba do editor e no save do arquivo ativo.
    function requestDiffFor(path) {
        activeDiffPath = path;
        if (path === "") {
            diffLineKinds = {};
            diffRevision++;
            return;
        }
        fileDiffRequested(path);
    }

    function openDiffDialog(path) {
        if (path === "") {
            return;
        }
        diffDialogPath = path;
        diffDialogText = "";
        diffDialogTracked = true;
        diffDialogLoading = true;
        diffDialogVisible = true;
        fileDiffRequested(path);
    }

    function closeDiffDialog() {
        // Só esconde o diálogo: a lista de mudanças/staged pertence ao
        // workspace e é limpa no clear() (bug corrigido na M3.4).
        diffDialogVisible = false;
    }

    function handleFileDiff(path, isRepo, tracked, hunks, text) {
        inspectorController.handleFileDiff(path, tracked, text);
        if (diffDialogVisible && path === diffDialogPath) {
            diffDialogText = text;
            diffDialogTracked = tracked;
            diffDialogLoading = false;
        }
        if (path !== activeDiffPath) {
            return;
        }
        const kinds = {};
        if (isRepo) {
            for (let i = 0; i < hunks.length; i++) {
                const hunk = hunks[i];
                if (hunk.kind === "removed") {
                    kinds[hunk.startLine] = "removed";
                    continue;
                }
                for (let line = hunk.startLine;
                     line < hunk.startLine + hunk.lineCount; line++) {
                    kinds[line] = hunk.kind;
                }
            }
        }
        diffLineKinds = kinds;
        diffRevision++;
    }

    function clear() {
        inspectorController.clear();
        amend = false;
        repo = false;
        branchLabel = "";
        aheadCount = 0;
        behindCount = 0;
        changeCount = 0;
        gitKinds = {};
        revision++;
        activeDiffPath = "";
        diffLineKinds = {};
        diffRevision++;
        diffDialogVisible = false;
        gitChangesModel.clear();
        stagedCount = 0;
        lastMutationError = "";
        discardDialogVisible = false;
        historyController.clear();
        gitBranchesModel.clear();
        branchMenuVisible = false;
        remoteOperationRunning = false;
    }

    function handleStatus(isRepo, branch, detached, shortSha, ahead, behind,
                          entries) {
        repo = isRepo;
        if (!isRepo) {
            clear();
            return;
        }
        if (branch !== "") {
            branchLabel = branch;
        } else if (detached && shortSha !== "") {
            branchLabel = qsTr("%1 (solto)").arg(shortSha);
        } else {
            branchLabel = "HEAD";
        }
        aheadCount = ahead;
        behindCount = behind;
        changeCount = entries.length;
        lastMutationError = "";
        const kinds = {};
        gitChangesModel.clear();
        let staged = 0;
        for (let i = 0; i < entries.length; i++) {
            const absPath = workspaceRoot + "/" + entries[i].path;
            kinds[absPath] = entries[i].kind;
            gitChangesModel.append({
                path: entries[i].path,
                absPath: absPath,
                folder: gitRules.folderOf(entries[i].path),
                kind: entries[i].kind,
                staged: entries[i].staged
            });
            if (entries[i].staged) {
                staged++;
            }
        }
        stagedCount = staged;
        gitKinds = kinds;
        revision++;
        inspectorController.dropChangeIfGone(Object.keys(kinds));
        if (pushAfterCommit) {
            pushAfterCommit = false;
            startRemote("push");
        }
        // Mutações (stage/discard/commit) também mexem no diff do arquivo
        // ativo: re-aponta a gutter junto.
        if (activeDiffPath !== "") {
            fileDiffRequested(activeDiffPath);
        }
    }

    function toggleStaged(index) {
        if (index < 0 || index >= gitChangesModel.count) {
            return;
        }
        const entry = gitChangesModel.get(index);
        if (entry.staged) {
            unstageRequested([entry.absPath]);
        } else {
            stageRequested([entry.absPath]);
        }
    }

    // Toda uma pasta de uma vez (o checkbox da secao, HUD fatia 2).
    function stageFolder(absPaths, stageAll) {
        if (absPaths.length > 0) { stageAll ? stageRequested(absPaths) : unstageRequested(absPaths); }
    }

    function openDiscardDialog(index) {
        if (index < 0 || index >= gitChangesModel.count) {
            return;
        }
        const entry = gitChangesModel.get(index);
        discardDialogPath = entry.path;
        discardDialogAbsPath = entry.absPath;
        discardDialogVisible = true;
    }

    function confirmDiscard() {
        discardDialogVisible = false;
        if (discardDialogAbsPath !== "") {
            discardRequested([discardDialogAbsPath]);
        }
    }

    function cancelDiscard() {
        discardDialogVisible = false;
    }

    // Um commit novo pede algo staged; um amend so' de mensagem nao.
    function commit(message) {
        if (message.trim() === "" || (stagedCount === 0 && !amend)) {
            return;
        }
        commitRequested(message, amend);
        amend = false;
    }

    // Commit e Push: o push sai quando o status do commit voltar.
    property bool pushAfterCommit: false

    function commitAndPush(message) {
        if (message.trim() === "" || (stagedCount === 0 && !amend)) {
            return;
        }
        pushAfterCommit = true;
        commit(message);
    }

    function handleRequestFailed(method, message) {
        if (method === "git.commit") {
            pushAfterCommit = false;
        }
        if (method.startsWith("git.")) {
            lastMutationError = message;
        }
    }

    function openBranchMenu() {
        if (branchMenuVisible) {
            branchMenuVisible = false;
            return;
        }
        branchMenuVisible = true;
        branchesRequested();
    }

    function closeBranchMenu() {
        branchMenuVisible = false;
    }

    function handleBranches(isRepo, branches) {
        gitBranchesModel.clear();
        if (!isRepo) {
            branchMenuVisible = false;
            return;
        }
        for (let i = 0; i < branches.length; i++) {
            gitBranchesModel.append({
                name: branches[i].name,
                current: branches[i].current === true
            });
        }
    }

    function checkoutBranch(branch) {
        branchMenuVisible = false;
        checkoutRequested(branch);
    }

    function createBranch(name) {
        if (name.trim() === "") {
            lastMutationError = qsTr("Informe o nome da nova branch.");
            return;
        }
        branchMenuVisible = false;
        branchCreateRequested(name.trim());
    }

    function startRemote(operation) {
        if (remoteOperationRunning) {
            return;
        }
        remoteOperationRunning = true;
        lastMutationError = "";
        if (operation === "pull") {
            pullRequested();
        } else {
            pushRequested();
        }
    }

    function handleRemoteFinished(operation, success, message) {
        remoteOperationRunning = false;
        if (!success) {
            lastMutationError = message;
        }
        refresh();
        if (root.historyVisible) {
            refreshHistory();
        }
    }

    function rejectDirtyOperation() {
        remoteOperationRunning = false;
        lastMutationError = qsTr("Salve todas as abas modificadas antes desta operacao Git.");
    }

    // O diff de um commit vai para o painel da direita da HUD (2026-09-18);
    // o dialogo ficou so' para o diff de ARQUIVO pedido do editor.
    function handleCommitDiff(sha, text) {
        inspectorController.handleCommitDiff(sha, text);
    }
}
