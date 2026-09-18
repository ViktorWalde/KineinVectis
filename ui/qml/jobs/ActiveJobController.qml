import QtQuick

// O JOB EM CURSO para a barra de status (Etapa 2, F2 do roadmaps/43):
// derivado do `jobsModel` do JobsController — o mais recente com status
// `running`/`cancelRequested` —, com o titulo, o progresso, a ultima
// linha e se cancela. A barra mostra "o que esta' acontecendo" sem uma
// segunda lista de jobs: um dono (o JobsController) para os jobs, um dono
// (este) para a pergunta "qual e' o de agora?".
Item {
    id: root

    property var jobsModel: null

    property int runningCount: 0
    property string jobId: ""
    property string title: ""
    property real progress: -1
    property string message: ""
    property bool canCancel: false

    signal cancelRequested(string jobId)

    visible: false

    // Os status em que o job ainda esta' VIVO (o JobsPanel pinta por status;
    // aqui a pergunta e' outra: "ainda esta' acontecendo?").
    readonly property var aliveStatuses: ["queued", "running", "cancelRequested"]

    function isRunning(status) {
        return aliveStatuses.indexOf(status) >= 0;
    }

    function refresh() {
        if (jobsModel === null || jobsModel === undefined) {
            clear();
            return;
        }
        let count = 0;
        let ultimo = -1;
        for (let i = 0; i < jobsModel.count; i++) {
            if (isRunning(jobsModel.get(i).status)) {
                count += 1;
                ultimo = i;
            }
        }
        runningCount = count;
        if (ultimo < 0) {
            clear();
            runningCount = 0;
            return;
        }
        const row = jobsModel.get(ultimo);
        jobId = row.jobId;
        title = row.title;
        progress = row.progress !== undefined ? Number(row.progress) : -1;
        message = row.latestLine !== undefined ? row.latestLine : "";
        canCancel = row.canCancel === true;
    }

    function clear() {
        jobId = "";
        title = "";
        progress = -1;
        message = "";
        canCancel = false;
    }

    function cancel() {
        if (jobId !== "" && canCancel) {
            cancelRequested(jobId);
        }
    }

    // "compilando · 3 jobs" — o que a barra escreve.
    function summary() {
        if (runningCount === 0) {
            return "";
        }
        const extra = runningCount > 1 ? qsTr(" · %1 jobs").arg(runningCount) : "";
        return title + extra;
    }

    Connections {
        target: root.jobsModel

        function onCountChanged() { root.refresh(); }
        function onDataChanged() { root.refresh(); }
    }

    onJobsModelChanged: refresh()
}
