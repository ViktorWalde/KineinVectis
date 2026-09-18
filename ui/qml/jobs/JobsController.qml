import QtQuick

Item {
    id: root

    property string workspaceRoot: ""
    property bool building: false
    property bool testing: false
    property bool analyzing: false
    property alias buildOutputModel: buildOutputItemsModel
    property alias problemsModel: problemItemsModel
    property alias testModel: testItemsModel
    property alias testOutputModel: testOutputItemsModel
    // A ARVORE de testes antes do run (test.discover, 2026-09-13): id exato,
    // nome, arquivo e o status do ultimo run (vazio = nunca rodou).
    property alias discoveredModel: discoveredItemsModel
    property string discoverRunner: ""
    property bool discovering: false
    property alias jobsModel: jobItemsModel
    property string testSummary: ""

    signal runBuildRequested(string buildSystem)
    signal runTestsRequested(string buildSystem)
    signal runOneTestRequested(string testId, string buildSystem)
    signal discoverTestsRequested(string buildSystem)
    signal runQualityRequested()
    signal runCoverageRequested()
    signal showTabRequested(string tab)

    visible: false

    ListModel {
        id: buildOutputItemsModel
    }

    ListModel {
        id: problemItemsModel
    }

    ListModel {
        id: testItemsModel
    }

    // A saida BRUTA do runner (event.test.output): e' onde o pytest explica a
    // falha e onde "No module named pytest" aparece. Ate 2026-09-13 o sinal
    // existia no C++ e ninguem ouvia (41 A6).
    ListModel {
        id: testOutputItemsModel
    }

    ListModel {
        id: discoveredItemsModel
    }

    ListModel {
        id: jobItemsModel
    }

    function clear() {
        buildOutputItemsModel.clear();
        removeProblemsBySource("build");
        removeProblemsBySource("lsp");
        removeProblemsBySource("quality");
        testItemsModel.clear();
        testOutputItemsModel.clear();
        discoveredItemsModel.clear();
        discoverRunner = "";
        discovering = false;
        jobItemsModel.clear();
        testSummary = "";
    }

    function startBuild(buildSystem) {
        if (building || workspaceRoot === "") {
            return;
        }
        buildOutputItemsModel.clear();
        removeProblemsBySource("build");
        showTabRequested("build");
        runBuildRequested(buildSystem || "");
    }

    function startTests(buildSystem) {
        if (testing || workspaceRoot === "") {
            return;
        }
        prepareTestRun();
        runTestsRequested(buildSystem || "");
    }

    // Rodar UM teste da arvore, pelo id exato que o core deu.
    function runOneTest(testId, buildSystem) {
        if (testing || workspaceRoot === "" || testId === undefined || testId === "") {
            return;
        }
        prepareTestRun();
        runOneTestRequested(testId, buildSystem || "");
    }

    // Um run novo: os casos soltos e a saida zeram; a arvore FICA, com os
    // status apagados — e' ela que vai receber os resultados.
    function prepareTestRun() {
        testItemsModel.clear();
        testOutputItemsModel.clear();
        for (let i = 0; i < discoveredItemsModel.count; i++) {
            discoveredItemsModel.setProperty(i, "status", "");
        }
        testSummary = qsTr("rodando testes...");
        showTabRequested("tests");
    }

    function discoverTests(buildSystem) {
        if (discovering || workspaceRoot === "") {
            return;
        }
        discovering = true;
        showTabRequested("tests");
        discoverTestsRequested(buildSystem || "");
    }

    // O job de listagem acabou: a arvore e' a do core, na ordem dele. Falhou
    // (sem pytest, sem runner): o motivo vira o resumo, a arvore some.
    function handleTestsDiscovered(outcome) {
        discovering = false;
        if (outcome === undefined || outcome === null) return;
        discoveredItemsModel.clear();
        if (outcome.success !== true) {
            discoverRunner = "";
            testSummary = qsTr("nao listou: %1").arg(outcome.error);
            appendTestLine(String(outcome.error));
            return;
        }
        discoverRunner = outcome.runner === undefined ? "" : outcome.runner;
        const tests = outcome.tests === undefined || outcome.tests === null ? [] : outcome.tests;
        for (let i = 0; i < tests.length; i++) {
            const t = tests[i];
            discoveredItemsModel.append({ id: t.id, name: t.name === undefined ? t.id : t.name,
                                          file: t.file === undefined ? "" : t.file, status: "" });
        }
        testSummary = qsTr("%1 teste(s) listado(s) (%2)").arg(tests.length).arg(discoverRunner);
    }

    function discoveredIndex(id) {
        for (let i = 0; i < discoveredItemsModel.count; i++) {
            if (discoveredItemsModel.get(i).id === id) return i;
        }
        return -1;
    }

    function startQuality() {
        if (analyzing || workspaceRoot === "") {
            return;
        }
        removeProblemsBySource("quality");
        showTabRequested("problems");
        runQualityRequested();
    }

    // A cobertura dos testes (D8): um job como os outros; o desfecho vai
    // ao CoverageController pelo roteador, e a calha do editor o pinta.
    function startCoverage() {
        if (workspaceRoot === "") {
            return;
        }
        showTabRequested("jobs");
        runCoverageRequested();
    }

    function removeProblemsBySource(source) {
        for (let i = problemItemsModel.count - 1; i >= 0; i--) {
            if (problemItemsModel.get(i).source === source) {
                problemItemsModel.remove(i);
            }
        }
    }

    function relativeToRoot(path) {
        if (workspaceRoot !== "" && path.indexOf(workspaceRoot + "/") === 0) {
            return path.substring(workspaceRoot.length + 1);
        }
        return path;
    }

    function appendBuildLine(line) {
        buildOutputItemsModel.append({ line: line });
        while (buildOutputItemsModel.count > 2000) {
            buildOutputItemsModel.remove(0);
        }
    }

    function appendDiagnostic(diagnostic, defaultSeverity, source) {
        problemItemsModel.append({
            severity: diagnostic.severity !== undefined
                      ? diagnostic.severity : defaultSeverity,
            message: diagnostic.message !== undefined ? diagnostic.message : "",
            code: diagnostic.code !== undefined ? diagnostic.code : "",
            file: diagnostic.file !== undefined ? diagnostic.file : "",
            line: diagnostic.line !== undefined ? Number(diagnostic.line) : 0,
            column: diagnostic.column !== undefined ? Number(diagnostic.column) : 0,
            source: source
        });
    }

    function handleBuildStarted(command) {
        appendBuildLine("$ " + command);
    }

    function handleBuildOutput(line) {
        appendBuildLine(line);
    }

    function handleBuildDiagnostic(diagnostic) {
        appendDiagnostic(diagnostic, "error", "build");
    }

    function handleBuildFinished(success, exitCode) {
        appendBuildLine(success
                        ? qsTr("== build concluido com sucesso ==")
                        : qsTr("== build falhou (codigo %1) ==").arg(exitCode));
        if (!success && problemItemsModel.count > 0) {
            showTabRequested("problems");
        }
    }

    function appendTestLine(line) {
        testOutputItemsModel.append({ line: line });
        while (testOutputItemsModel.count > 2000) {
            testOutputItemsModel.remove(0);
        }
    }

    function handleTestStarted(command) {
        appendTestLine("$ " + command);
    }

    function handleTestOutput(line) {
        appendTestLine(line);
    }

    // O caso que rodou e' o mesmo id que a arvore listou (pytest: o node id;
    // cargo/ctest: o nome): o status vai para a linha da arvore; o que a
    // arvore nao conhece continua na lista de casos soltos.
    function handleTestCase(name, status) {
        const i = discoveredIndex(name);
        if (i >= 0) {
            discoveredItemsModel.setProperty(i, "status", status);
            return;
        }
        testItemsModel.append({ name: name, status: status });
        while (testItemsModel.count > 5000) {
            testItemsModel.remove(0);
        }
    }

    // `error` vem quando o runner nem correu (interpretador ou pytest
    // ausentes, tipo sem runner): e' a mensagem do core, com o passo a dar.
    function handleTestFinished(success, passed, failed, ignored, error) {
        if (error !== undefined && error !== "") {
            testSummary = qsTr("nao rodou: %1").arg(error);
            appendTestLine(error);
            return;
        }
        testSummary = (success
                       ? qsTr("passou: %1")
                       : qsTr("FALHOU — passou: %1")).arg(passed)
                + qsTr("  falhou: %1  ignorado: %2").arg(failed).arg(ignored);
    }

    // A analise escreve no MESMO painel do build: e' o mesmo tipo de saida
    // (o comando e as linhas da ferramenta) — pente-fino 2026-09-18.
    function handleQualityStarted(command) { appendBuildLine("$ " + command); }
    function handleQualityOutput(line) { appendBuildLine(line); }

    function handleQualityDiagnostic(diagnostic) {
        appendDiagnostic(diagnostic, "warning", "quality");
    }

    function handleQualityFinished() {
        showTabRequested("problems");
    }

    function findJobIndex(jobId) {
        for (let i = 0; i < jobItemsModel.count; i++) {
            if (jobItemsModel.get(i).jobId === jobId) {
                return i;
            }
        }
        return -1;
    }

    function trimJobsModel() {
        while (jobItemsModel.count > 100) {
            jobItemsModel.remove(0);
        }
    }

    function appendFallbackJob(jobId, status) {
        jobItemsModel.append({
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
        trimJobsModel();
        return jobItemsModel.count - 1;
    }

    function handleJobCreated(job) {
        const jobId = job.id !== undefined ? job.id : "";
        if (jobId === "") {
            return;
        }
        const existingIndex = findJobIndex(jobId);
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
            jobItemsModel.set(existingIndex, row);
            return;
        }
        jobItemsModel.append(row);
        trimJobsModel();
    }

    function handleJobProgress(jobId, status, progress, message) {
        if (jobId === "") {
            return;
        }
        let index = findJobIndex(jobId);
        if (index < 0) {
            index = appendFallbackJob(jobId, status);
        }
        jobItemsModel.setProperty(index, "status", status);
        if (progress >= 0) {
            jobItemsModel.setProperty(index, "progress", progress);
        }
        if (message !== "") {
            jobItemsModel.setProperty(index, "latestLine", message);
        }
    }

    function handleJobOutput(jobId, line) {
        const index = findJobIndex(jobId);
        if (index < 0) {
            return;
        }
        const row = jobItemsModel.get(index);
        jobItemsModel.setProperty(index, "latestLine", line);
        jobItemsModel.setProperty(index, "outputCount", Number(row.outputCount) + 1);
    }

    function handleJobFinished(jobId, status) {
        const index = findJobIndex(jobId);
        if (index < 0) {
            return;
        }
        jobItemsModel.setProperty(index, "status", status);
        jobItemsModel.setProperty(index, "progress", 1);
    }

    function handleLspDiagnostics(path, diagnostics) {
        const rel = relativeToRoot(path);
        for (let i = problemItemsModel.count - 1; i >= 0; i--) {
            const row = problemItemsModel.get(i);
            if (row.source === "lsp" && row.file === rel) {
                problemItemsModel.remove(i);
            }
        }
        for (let j = 0; j < diagnostics.length; j++) {
            const diagnostic = diagnostics[j];
            problemItemsModel.append({
                severity: diagnostic.severity !== undefined
                          ? diagnostic.severity : "error",
                message: diagnostic.message !== undefined
                         ? diagnostic.message : "",
                code: diagnostic.code !== undefined ? diagnostic.code : "",
                file: rel,
                line: diagnostic.line !== undefined ? Number(diagnostic.line) : 0,
                column: diagnostic.column !== undefined
                        ? Number(diagnostic.column) : 0,
                source: "lsp"
            });
        }
    }
}
