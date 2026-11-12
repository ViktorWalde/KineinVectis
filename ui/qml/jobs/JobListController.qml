import QtQuick

// A LISTA DOS JOBS: uma linha por job, com o status, o progresso e a ultima
// linha da saida, que a aba Jobs e a barra de status leem. Saiu do
// JobsController no C1-45 (2026-11-12), quando ele chegou ao limite de 400
// linhas; o JobsController segue com o `jobsModel` e os `handleJob*`.
QtObject {
    id: root

    readonly property ListModel model: ListModel {}

    function clear() {
        root.model.clear();
    }

    function findIndex(jobId) {
        for (let i = 0; i < root.model.count; i++) {
            if (root.model.get(i).jobId === jobId) {
                return i;
            }
        }
        return -1;
    }

    function trim() {
        while (root.model.count > 100) {
            root.model.remove(0);
        }
    }

    function appendFallback(jobId, status) {
        root.model.append({
            jobId: jobId,
            kind: "job",
            title: jobId,
            status: status,
            progress: -1,
            canCancel: false,
            risk: "low",
            latestLine: "",
            outputCount: 0
        });
        trim();
        return root.model.count - 1;
    }

    function handleCreated(job) {
        const jobId = job.id !== undefined ? job.id : "";
        if (jobId === "") {
            return;
        }
        const existingIndex = findIndex(jobId);
        const row = {
            jobId: jobId,
            kind: job.kind !== undefined ? job.kind : "job",
            title: job.title !== undefined ? job.title : jobId,
            status: job.status !== undefined ? job.status : "running",
            progress: job.progress !== undefined ? Number(job.progress) : -1,
            canCancel: job.canCancel !== undefined ? Boolean(job.canCancel) : false,
            risk: job.risk !== undefined ? job.risk : "low",
            latestLine: "",
            outputCount: 0
        };
        if (existingIndex >= 0) {
            root.model.set(existingIndex, row);
            return;
        }
        root.model.append(row);
        trim();
    }

    function handleProgress(jobId, status, progress, message) {
        if (jobId === "") {
            return;
        }
        let index = findIndex(jobId);
        if (index < 0) {
            index = appendFallback(jobId, status);
        }
        root.model.setProperty(index, "status", status);
        if (progress >= 0) {
            root.model.setProperty(index, "progress", progress);
        }
        if (message !== "") {
            root.model.setProperty(index, "latestLine", message);
        }
    }

    function handleOutput(jobId, line) {
        const index = findIndex(jobId);
        if (index < 0) {
            return;
        }
        const row = root.model.get(index);
        root.model.setProperty(index, "latestLine", line);
        root.model.setProperty(index, "outputCount", Number(row.outputCount) + 1);
    }

    function handleFinished(jobId, status) {
        const index = findIndex(jobId);
        if (index < 0) {
            return;
        }
        root.model.setProperty(index, "status", status);
        root.model.setProperty(index, "progress", 1);
    }
}
