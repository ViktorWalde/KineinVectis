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
    property alias jobsModel: jobList.model
    property string testSummary: ""
    // O placar da ultima rodada de testes, para a aba (HUD 2026-09-18); -1 = nao rodou.
    property int testsPassed: -1
    property int testsFailed: 0
    readonly property string testsBadge: testsPassed < 0 ? "" : testsPassed + "/" + (testsPassed + testsFailed)

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

    ListModel { id: problemItemsModel }
    ListModel { id: testItemsModel }

    // A saida BRUTA do runner (event.test.output): e' onde o pytest explica a
    // falha e onde "No module named pytest" aparece. Ate 2026-09-13 o sinal
    // existia no C++ e ninguem ouvia (41 A6).
    ListModel {
        id: testOutputItemsModel
    }

    ListModel { id: discoveredItemsModel }
    JobListController { id: jobList }

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
        jobList.clear();
        testSummary = "";
        testsPassed = -1;
        testsFailed = 0;
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
        // C1-45: a aba mostra as linhas desta listagem, como as de um run.
        testOutputItemsModel.clear();
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
            testSummary = qsTr("não listou: %1").arg(outcome.error);
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

    // As regras puras dos problemas (proximo passo; repeticao build/LSP).
    ProblemRules { id: problemRules }

    function appendDiagnostic(diagnostic, defaultSeverity, source) {
        if (problemRules.isDuplicate(problemItemsModel, diagnostic)) {
            return;
        }
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

    function handleBuildStarted(command) { appendBuildLine("$ " + command); }
    function handleBuildOutput(line) { appendBuildLine(line); }

    function handleBuildDiagnostic(diagnostic) {
        appendDiagnostic(diagnostic, "error", "build");
    }

    function handleBuildFinished(success, exitCode) {
        appendBuildLine(success
                        ? qsTr("== build concluído com sucesso ==")
                        : qsTr("== build falhou (código %1) ==").arg(exitCode));
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

    function handleTestStarted(command) { appendTestLine("$ " + command); }
    function handleTestOutput(line) { appendTestLine(line); }

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
            testSummary = qsTr("não rodou: %1").arg(error);
            appendTestLine(error);
            return;
        }
        testSummary = (success
                       ? qsTr("passou: %1")
                       : qsTr("FALHOU — passou: %1")).arg(passed)
                + qsTr("  falhou: %1  ignorado: %2").arg(failed).arg(ignored);
        testsPassed = passed;
        testsFailed = failed;
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

    // A lista dos jobs e' do JobListController; os routers chamam estes.
    function handleJobCreated(job) { jobList.handleCreated(job); }
    function handleJobProgress(jobId, status, progress, message) { jobList.handleProgress(jobId, status, progress, message); }
    function handleJobOutput(jobId, line) { jobList.handleOutput(jobId, line); }
    function handleJobFinished(jobId, status) { jobList.handleFinished(jobId, status); }

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

    // Erro entre os problemas? A cor do contador da aba (amarelo so' com avisos).
    function hasErrorProblems() {
        for (let i = 0; i < problemItemsModel.count; i++) {
            if (StatusColors.isError(problemItemsModel.get(i).severity)) return true;
        }
        return false;
    }
}
