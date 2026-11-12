import QtQuick
import KineinVectis

// As linhas da listagem na aba Testes (C1-45, 2026-11-12): o JobsEventRouter,
// o JobsController e o TestsPanel REAIS, com um cliente falso que emite o que
// o core 0.219.0 manda no job do test.discover: o event.test.started, um
// event.test.output por linha e o event.test.discovered no fim. Antes o
// motivo do binario que nao respondeu no prazo so' saia no log do job, que
// nenhuma vista mostra inteiro.
//
// O que se prova: comecar a listagem zera a saida do run anterior; as linhas
// chegam a area de baixo na ordem, com o motivo; o erro da listagem que falha
// entra uma vez, no fim; a arvore fica do tamanho que tinha no run seguinte;
// a lista dos jobs, que saiu para o JobListController, segue pelo router.
Item {
    id: root
    width: 800
    height: 420
    property int failures: 0
    property int listings: 0

    QtObject {
        id: bridge
        signal testStarted(string command)
        signal testOutput(string line, string stream)
        signal testsDiscovered(var outcome)
        signal jobCreated(var job)
        signal jobOutput(string jobId, string line)
        signal jobFinished(string jobId, string status)
        // Os outros sinais que o roteador ouve (sem eles, o Connections avisa).
        signal buildStarted(string command)
        signal buildOutput(string line)
        signal buildDiagnostic(var diagnostic)
        signal buildFinished(bool success, int exitCode, int diagnostics)
        signal lspDiagnostics(string path, var diagnostics)
        signal testCase(string name, string status)
        signal testFinished(bool success, int passed, int failed, int ignored, string error)
        signal qualityStarted(string command)
        signal qualityOutput(string line, string stream)
        signal qualityDiagnostic(var diagnostic)
        signal qualityFinished(bool success, int exitCode, int diagnostics)
        signal jobProgress(string jobId, string status, real progress, string message)
    }
    JobsController {
        id: jobs
        workspaceRoot: "/tmp/proj"
        onDiscoverTestsRequested: function(buildSystem) { root.listings++; }
    }
    JobsEventRouter {
        coreClient: bridge
        jobsController: jobs
    }
    TestsPanel {
        id: panel
        anchors.fill: parent
        controller: jobs
    }

    readonly property string reason: "o binário `/b/server` não respondeu a `--gtest_list_tests` em 5 s, o "
                                     + "`DISCOVERY_TIMEOUT` padrão do `gtest_discover_tests`; os casos dele "
                                     + "ficam de fora, e a linha do ctest roda o binário inteiro"
    readonly property var listing: ["Test project /b", "  Test #1: server", "  Test #2: fake", "",
                                    "Total Tests: 2", root.reason,
                                    "1 caso(s) de gtest/Catch2 dentro dos binários do ctest"]

    function find(item, name) {
        if (item.objectName === name) return item;
        for (const child of item.children) {
            const result = find(child, name);
            if (result) return result;
        }
        return null;
    }
    function check(ok, message) {
        if (!ok) { root.failures++; console.error("FALHOU: " + message); }
    }
    function lines() {
        const all = [];
        for (let i = 0; i < jobs.testOutputModel.count; i++) all.push(jobs.testOutputModel.get(i).line);
        return all;
    }
    // O job do test.discover como o core o manda: cada linha vai a aba e ao log.
    function sendListing(jobId) {
        bridge.jobCreated({ id: jobId, kind: "test", title: "Listar testes (ctest)", status: "running", canCancel: true });
        bridge.testStarted("ctest --test-dir /b -N");
        bridge.jobOutput(jobId, "$ ctest --test-dir /b -N");
        for (const line of root.listing) {
            bridge.testOutput(line, "stdout");
            bridge.jobOutput(jobId, line);
        }
    }

    Component.onCompleted: {
        const tree = find(panel, "testsTree");
        const output = find(panel, "testsOutput");
        check(tree !== null && output !== null, "a arvore e a saida existem");

        // Um run deixa saida na aba.
        jobs.startTests("");
        bridge.testStarted("ctest --test-dir /b --output-on-failure");
        bridge.testOutput("100% tests passed", "stdout");
        check(jobs.testOutputModel.count === 2, "a saida do run: " + lines());

        // A listagem comeca: a saida do run sai da aba.
        jobs.discoverTests("");
        check(root.listings === 1 && jobs.discovering, "listou uma vez");
        check(jobs.testOutputModel.count === 0, "a listagem zera a saida do run: " + lines());

        // As linhas da listagem chegam a area de baixo, na ordem, com o motivo.
        sendListing("job_7");
        bridge.testsDiscovered({ jobId: "job_7", success: true, runner: "ctest", command: "ctest --test-dir /b -N",
                                 tests: [{ id: "server", name: "server" }, { id: "fake", name: "fake" },
                                         { id: "fake::Math.Adds", name: "fake::Math.Adds" }] });
        bridge.jobOutput("job_7", "3 teste(s) encontrado(s)");
        bridge.jobFinished("job_7", "success");
        const expected = ["$ ctest --test-dir /b -N"].concat(root.listing);
        check(JSON.stringify(lines()) === JSON.stringify(expected), "as linhas da listagem: " + JSON.stringify(lines()));
        check(!jobs.discovering && jobs.discoveredModel.count === 3, "a arvore chegou: " + jobs.discoveredModel.count);
        check(jobs.testSummary === "3 teste(s) listado(s) (ctest)", "o resumo: " + jobs.testSummary);
        output.forceLayout();
        check(output.visible && output.count === expected.length, "a area de baixo mostra as linhas: " + output.count);
        const shown = output.itemAtIndex(expected.indexOf(root.reason));
        check(shown !== null && shown.text === root.reason, "o motivo aparece na aba: " + (shown ? shown.text : null));

        // O log do job tem as mesmas linhas, e a lista dos jobs segue pelo router.
        check(jobs.jobsModel.count === 1, "um job: " + jobs.jobsModel.count);
        const job = jobs.jobsModel.get(0);
        check(job.jobId === "job_7" && job.status === "success", "o job da listagem: " + job.jobId + " " + job.status);
        check(job.latestLine === "3 teste(s) encontrado(s)", "a ultima linha do job: " + job.latestLine);
        check(job.outputCount === expected.length + 1, "as linhas do job: " + job.outputCount);

        // C1-44: a arvore que chegou com as linhas fica do tamanho dela no run.
        const shared = tree.height;
        check(shared < root.height - 60, "a arvore divide o painel com a saida: " + shared);
        jobs.runOneTest("fake::Math.Adds", "");
        check(jobs.testOutputModel.count === 0, "o run zera as linhas da listagem: " + lines());
        check(tree.height === shared, "a arvore fica do mesmo tamanho: " + tree.height + " e " + shared);

        // A listagem que falha: as linhas dela e o erro, uma vez, no fim.
        bridge.testOutput("[  PASSED  ] 1 test.", "stdout");
        jobs.discoverTests("");
        check(root.listings === 2 && jobs.testOutputModel.count === 0, "relistar zera a saida: " + lines());
        sendListing("job_8");
        bridge.testsDiscovered({ jobId: "job_8", success: false, error: "listagem cancelada" });
        const failed = lines();
        check(failed.length === expected.length + 1, "as linhas e o erro: " + JSON.stringify(failed));
        check(failed.filter(line => line === "listagem cancelada").length === 1
              && failed[failed.length - 1] === "listagem cancelada", "o erro uma vez, no fim: " + JSON.stringify(failed));
        check(jobs.testSummary === "não listou: listagem cancelada", "o resumo do erro: " + jobs.testSummary);
        check(jobs.discoveredModel.count === 0, "sem arvore depois do erro");

        // clear() esquece a lista dos jobs, que mora no JobListController.
        check(jobs.jobsModel.count === 2, "dois jobs: " + jobs.jobsModel.count);
        jobs.clear();
        check(jobs.jobsModel.count === 0 && jobs.testOutputModel.count === 0, "o clear esquece os jobs e a saida");

        Qt.exit(root.failures === 0 ? 0 : 1);
    }
}
