import QtQuick

// O PAINEL DA DIREITA da HUD do Git (2026-09-18): o que esta' selecionado
// — uma mudanca (o diff dela contra o HEAD) ou um commit (autor, refs,
// arquivos, patch). Filho do GitController: os pedidos saem pelos sinais
// do pai (fileDiffRequested/commitDiffRequested) e as respostas voltam
// por handleFileDiff/handleCommitDiff, que o pai repassa.
Item {
    id: root

    // "" | "change" | "commit"
    property string kind: ""
    property string path: ""
    property string sha: ""
    property string shortSha: ""
    property string author: ""
    property string age: ""
    property string summary: ""
    property var refs: []
    property bool loading: false
    property bool tracked: true
    property string patch: ""
    readonly property var files: rules.patchFiles(patch)

    signal fileDiffWanted(string path)
    signal commitDiffWanted(string sha)

    visible: false

    GitRules { id: rules }

    function clear() {
        kind = "";
        path = "";
        sha = "";
        shortSha = "";
        author = "";
        age = "";
        summary = "";
        refs = [];
        loading = false;
        tracked = true;
        patch = "";
    }

    // Uma mudanca da lista: o diff do arquivo contra o HEAD.
    function showChange(absPath, relativePath) {
        kind = "change";
        path = absPath;
        summary = relativePath;
        sha = "";
        shortSha = "";
        refs = [];
        patch = "";
        tracked = true;
        loading = true;
        fileDiffWanted(absPath);
    }

    // Um commit do historico: o patch dele.
    function showCommit(entry) {
        kind = "commit";
        path = "";
        sha = entry.sha;
        shortSha = entry.shortSha;
        author = entry.author;
        age = entry.age;
        summary = entry.summary;
        refs = entry.refsText === undefined || entry.refsText === "" ? [] : entry.refsText.split("\u001f");
        patch = "";
        loading = true;
        commitDiffWanted(entry.sha);
    }

    function handleFileDiff(absPath, isTracked, text) {
        if (kind !== "change" || absPath !== path) {
            return;
        }
        patch = text;
        tracked = isTracked;
        loading = false;
    }

    function handleCommitDiff(commitSha, text) {
        if (kind !== "commit" || commitSha !== sha) {
            return;
        }
        patch = text;
        loading = false;
    }

    // A mudanca selecionada sumiu da lista (commitada/descartada): fecha.
    function dropChangeIfGone(paths) {
        if (kind === "change" && paths.indexOf(path) < 0) {
            clear();
        }
    }
}
