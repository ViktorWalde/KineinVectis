import QtQuick
import "../../ui/qml/jobs"
import "../../ui/qml/workspace"

// O que a barra de status diz (Etapa 2, F2, 2026-09-18): o job em curso
// derivado do jobsModel REAL do JobsController, e o estado dos servidores
// de linguagem.
//
// O que se prova: sem job, nada; um job criado aparece com titulo e
// progresso indeterminado; o progresso entra; a ultima linha entra; dois
// jobs contam; terminar um deixa o outro; cancelar so' quando pode e pede
// pelo id; o LSP resume ● todos rodando / … subindo / ✗ caiu, e trocar de
// workspace esquece.
Item {
    id: root

    property var cancelados: []

    JobsController {
        id: jobs
    }

    ActiveJobController {
        id: ativo

        jobsModel: jobs.jobsModel
        onCancelRequested: function(jobId) { root.cancelados.push(jobId); }
    }

    LspStatusController {
        id: lsp
    }

    Component.onCompleted: {
        let failures = 0;
        if (ativo.title !== "" || ativo.runningCount !== 0 || ativo.summary() !== "") failures += 1;

        jobs.handleJobCreated({ id: "j1", kind: "build", title: "Build (cargo)", status: "running", canCancel: true });
        if (ativo.title !== "Build (cargo)" || ativo.runningCount !== 1 || ativo.progress >= 0 || !ativo.canCancel) failures += 2;
        jobs.handleJobProgress("j1", "running", 0.4, "Compiling kinein-core");
        if (Math.abs(ativo.progress - 0.4) > 0.001 || ativo.message !== "Compiling kinein-core") failures += 4;
        jobs.handleJobOutput("j1", "   Compiling kinein-protocol");
        if (ativo.message !== "   Compiling kinein-protocol") failures += 8;

        jobs.handleJobCreated({ id: "j2", kind: "index", title: "Indexar", status: "running", canCancel: false });
        if (ativo.runningCount !== 2 || ativo.title !== "Indexar" || ativo.canCancel) failures += 16;
        if (ativo.summary().indexOf("2 jobs") < 0) failures += 32;
        ativo.cancel();
        if (root.cancelados.length !== 0) failures += 64;

        jobs.handleJobFinished("j2", "success");
        if (ativo.runningCount !== 1 || ativo.title !== "Build (cargo)" || !ativo.canCancel) failures += 128;
        ativo.cancel();
        if (root.cancelados.join(",") !== "j1") failures += 256;
        jobs.handleJobProgress("j1", "cancelRequested", -1, "");
        if (ativo.runningCount !== 1) failures += 512;
        jobs.handleJobFinished("j1", "cancelled");
        if (ativo.runningCount !== 0 || ativo.title !== "" || ativo.summary() !== "") failures += 1024;

        // LSP
        if (lsp.summary() !== "") failures += 2048;
        lsp.handleStatus("cpp", "starting", "");
        if (lsp.summary().indexOf("…") < 0 || lsp.summary().indexOf("cpp") < 0) failures += 4096;
        lsp.handleStatus("cpp", "running", "");
        lsp.handleStatus("rust", "running", "");
        if (lsp.summary() !== "LSP ● 2" || lsp.hasFailure()) failures += 8192;
        lsp.handleStatus("python", "failed", "basedpyright nao encontrado");
        if (lsp.summary() !== "LSP ✗ python" || !lsp.hasFailure() || lsp.detail().indexOf("basedpyright") < 0) failures += 16384;
        lsp.handleStatus("python", "stopped", "");
        if (lsp.summary() !== "LSP ● 2") failures += 32768;
        lsp.workspaceRoot = "/outro";
        if (lsp.summary() !== "") failures += 65536;

        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
