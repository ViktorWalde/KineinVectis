pragma ComponentBehavior: Bound
import QtQuick

// O PASSADO do repositorio: blame (por linha) e log (por commit).
//
// Saiu do GitController.qml em 2026-09-03 (etapa 17 do roadmaps/34), que
// estava em 464/400 misturando seis dominios.
//
// POR QUE BLAME E LOG SAO O MESMO DONO, e nao dois. Os dois respondem
// "quem e quando", os dois leem metadado de commit, e a evidencia concreta e'
// que `ageLabel` — o formatador de "ha' quanto tempo" — era chamado pelos
// DOIS. Um helper compartilhado por dois grupos costuma ser o sinal de que os
// dois grupos sao um so'. O resto do GitController cuida do PRESENTE: status,
// staging, diff, branch.
Item {
    id: root

    // Blame: quem escreveu cada linha do arquivo aberto.
    property bool blameVisible: false
    property string blamePath: ""
    property var blameLineAnnotations: ({})
    property int blameRevision: 0

    // Log: os commits do repositorio.
    property alias historyModel: gitHistoryModel
    property bool historyVisible: false
    property bool historyLoading: false

    signal blameRequested(string path)
    signal logRequested()

    visible: false

    ListModel {
        id: gitHistoryModel
    }

    // Esquece tudo do workspace anterior. Dono unico do "limpa o passado":
    // o GitController nao mexe mais em gitHistoryModel/historyVisible.
    function clear() {
        hideBlame();
        gitHistoryModel.clear();
        historyVisible = false;
        historyLoading = false;
    }

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

    GitRules { id: gitRules }

    // Quantas raias o grafo tem, para a largura da coluna.
    property int laneCount: 1

    // A linha do historico com esse sha (para o painel da direita).
    function entry(sha) {
        for (let i = 0; i < gitHistoryModel.count; i++) {
            if (gitHistoryModel.get(i).sha === sha) {
                return gitHistoryModel.get(i);
            }
        }
        return null;
    }

    function handleLog(isRepo, entries) {
        historyLoading = false;
        gitHistoryModel.clear();
        laneCount = 1;
        if (!isRepo) {
            return;
        }
        // 0.126.0: os pais viram raias (GitRules.lanes) e os refs viram chips.
        const raias = gitRules.lanes(entries);
        let maximo = 1;
        for (let i = 0; i < entries.length; i++) {
            maximo = Math.max(maximo, raias[i].laneCount);
            gitHistoryModel.append({
                sha: entries[i].sha,
                shortSha: entries[i].shortSha,
                author: entries[i].author,
                age: ageLabel(entries[i].authorTime),
                summary: entries[i].summary,
                // Um ListModel nao guarda array de string como tal: os refs
                // viajam juntos por US (0x1f) e a vista separa.
                refsText: entries[i].refs === undefined ? "" : entries[i].refs.join("\u001f"),
                lane: raias[i].lane,
                merge: raias[i].merge
            });
        }
        laneCount = maximo;
    }

}
