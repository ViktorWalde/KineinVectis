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
    // path ABSOLUTO -> kind (modified|added|deleted|renamed|untracked|
    // conflicted); a revisão força os bindings da árvore a reavaliarem.
    property var gitKinds: ({})
    property int revision: 0
    // Diff do arquivo ativo no editor: hunks expandidos em linha→kind
    // (added|modified|removed) para as marcas da gutter.
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
    // M3.4: blame do arquivo ativo (toggle por comando; segue a aba) —
    // linha → rótulo "autor, idade"; a revisão força o rebind da gutter.
    property bool blameVisible: false
    property string blamePath: ""
    property var blameLineAnnotations: ({})
    property int blameRevision: 0
    // M3.4: histórico de commits (vista da aba Git) e diff de commit no
    // GitDiffDialog reusado (título via diffDialogCommitLabel).
    property alias historyModel: gitHistoryModel
    property bool historyVisible: false
    property bool historyLoading: false
    property string diffDialogCommitLabel: ""
    property string diffDialogSha: ""
    property alias branchesModel: gitBranchesModel
    property bool branchMenuVisible: false
    property bool remoteOperationRunning: false

    signal statusRequested()
    signal fileDiffRequested(string path)
    signal stageRequested(var paths)
    signal unstageRequested(var paths)
    signal discardRequested(var paths)
    signal commitRequested(string message)
    signal blameRequested(string path)
    signal logRequested()
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
        id: gitHistoryModel
    }

    ListModel {
        id: gitBranchesModel
    }

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
        diffDialogCommitLabel = "";
        diffDialogSha = "";
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
        hideBlame();
        gitHistoryModel.clear();
        historyVisible = false;
        historyLoading = false;
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

    function commit(message) {
        if (message.trim() === "" || stagedCount === 0) {
            return;
        }
        commitRequested(message);
    }

    function handleRequestFailed(method, message) {
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
        if (historyVisible) {
            refreshHistory();
        }
    }

    function rejectDirtyOperation() {
        remoteOperationRunning = false;
        lastMutationError = qsTr("Salve todas as abas modificadas antes desta operacao Git.");
    }

    // ---- M3.4: blame ----

    function toggleBlame(path) {
        if (blameVisible) {
            hideBlame();
            return;
        }
        if (path === "") {
            return;
        }
        blameVisible = true;
        requestBlameFor(path);
    }

    function hideBlame() {
        blameVisible = false;
        blamePath = "";
        blameLineAnnotations = {};
        blameRevision++;
    }

    // Chamado na troca de aba e no save enquanto o blame está ligado.
    function requestBlameFor(path) {
        if (!blameVisible) {
            return;
        }
        if (path === "") {
            blamePath = "";
            blameLineAnnotations = {};
            blameRevision++;
            return;
        }
        blamePath = path;
        blameRequested(path);
    }

    // Idade relativa compacta ("min", "h", "d", "m" de meses, "a").
    function ageLabel(epochSeconds) {
        if (epochSeconds <= 0) {
            return "";
        }
        const seconds = Math.max(0, Date.now() / 1000 - epochSeconds);
        if (seconds < 3600) {
            return qsTr("%1min").arg(Math.max(1, Math.floor(seconds / 60)));
        }
        if (seconds < 86400) {
            return qsTr("%1h").arg(Math.floor(seconds / 3600));
        }
        if (seconds < 2592000) {
            return qsTr("%1d").arg(Math.floor(seconds / 86400));
        }
        if (seconds < 31536000) {
            return qsTr("%1m").arg(Math.floor(seconds / 2592000));
        }
        return qsTr("%1a").arg(Math.floor(seconds / 31536000));
    }

    function handleBlame(path, isRepo, tracked, groups) {
        if (!blameVisible || path !== blamePath) {
            return;
        }
        const annotations = {};
        if (isRepo && tracked) {
            for (let i = 0; i < groups.length; i++) {
                const group = groups[i];
                const label = group.committed
                        ? group.author + ", " + ageLabel(group.authorTime)
                        : qsTr("não commitado");
                for (let line = group.startLine;
                     line < group.startLine + group.lineCount; line++) {
                    annotations[line] = label;
                }
            }
        }
        blameLineAnnotations = annotations;
        blameRevision++;
    }

    // ---- M3.4: histórico e diff de commit ----

    function openHistory() {
        historyVisible = true;
        refreshHistory();
    }

    function showChanges() {
        historyVisible = false;
    }

    function refreshHistory() {
        historyLoading = true;
        logRequested();
    }

    function handleLog(isRepo, entries) {
        historyLoading = false;
        gitHistoryModel.clear();
        if (!isRepo) {
            return;
        }
        for (let i = 0; i < entries.length; i++) {
            gitHistoryModel.append({
                sha: entries[i].sha,
                shortSha: entries[i].shortSha,
                author: entries[i].author,
                age: ageLabel(entries[i].authorTime),
                summary: entries[i].summary
            });
        }
    }

    function openCommitDiff(sha, shortSha, summary) {
        diffDialogSha = sha;
        diffDialogCommitLabel = qsTr("commit %1 — %2").arg(shortSha).arg(summary);
        diffDialogPath = "";
        diffDialogText = "";
        diffDialogTracked = true;
        diffDialogLoading = true;
        diffDialogVisible = true;
        commitDiffRequested(sha);
    }

    function handleCommitDiff(sha, text) {
        if (!diffDialogVisible || sha !== diffDialogSha) {
            return;
        }
        diffDialogText = text;
        diffDialogLoading = false;
    }
}
