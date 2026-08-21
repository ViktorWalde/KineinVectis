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
    property alias jobsModel: jobItemsModel
    property string testSummary: ""
    property bool auditing: false
    property string auditSummary: ""
    property bool memchecking: false
    property string memcheckSummary: ""

    // Cobertura (L2 fatia 3) entra como UM objeto e nao como quatro
    // propriedades soltas: e um resultado coeso (mediu? quanto? por arquivo?
    // por que falhou?) e desce a arvore em uma linha so. O ShellWorkspaceHost
    // esta em debito e a catraca cobra de quem passa por la — quatro
    // pass-throughs para um unico conceito seriam exatamente a cerimonia que
    // o limite existe para impedir.
    readonly property alias coverage: coverageState

    QtObject {
        id: coverageState

        property bool measuring: false
        property string summary: ""
        property string error: ""
        property var filesModel: coverageFilesModel

        // A acao vem junto do resultado de proposito. A alternativa era um
        // sinal subindo BottomPanelHost -> ShellWorkspaceHost -> controller,
        // e o ShellWorkspaceHost esta em debito: tres linhas de cerimonia
        // para chegar a uma chamada que o painel ja tem em maos.
        function start() {
            root.startCoverage("");
        }
    }

    ListModel {
        id: coverageFilesModel
    }

    signal runBuildRequested(string buildSystem)
    signal runTestsRequested(string buildSystem)
    signal runQualityRequested()
    signal runCoverageRequested(string buildSystem)
    signal runAuditRequested()
    signal runMemcheckRequested()
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

    ListModel {
        id: jobItemsModel
    }

    function clear() {
        buildOutputItemsModel.clear();
        removeProblemsBySource("build");
        removeProblemsBySource("lsp");
        removeProblemsBySource("quality");
        removeProblemsBySource("audit");
        removeProblemsBySource("memcheck");
        auditSummary = "";
        memcheckSummary = "";
        testItemsModel.clear();
        jobItemsModel.clear();
        testSummary = "";
        clearCoverage();
    }

    function clearCoverage() {
        coverageFilesModel.clear();
        coverageState.summary = "";
        coverageState.error = "";
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
        testItemsModel.clear();
        testSummary = qsTr("rodando testes...");
        showTabRequested("tests");
        runTestsRequested(buildSystem || "");
    }

    function startQuality() {
        if (analyzing || workspaceRoot === "") {
            return;
        }
        removeProblemsBySource("quality");
        showTabRequested("problems");
        runQualityRequested();
    }

    function startMemcheck() {
        if (memchecking || workspaceRoot === "") {
            return;
        }
        removeProblemsBySource("memcheck");
        memchecking = true;
        memcheckSummary = qsTr("rodando os testes sob o Valgrind...");
        showTabRequested("problems");
        runMemcheckRequested();
    }

    function handleMemcheckDiagnostic(diagnostic) {
        appendDiagnostic(diagnostic, "warning", "memcheck");
    }

    function handleMemcheckFinished(success, tests, findings, error) {
        memchecking = false;
        if (!success) {
            // O erro do core diz o GESTO que falta (configurar o build, por
            // exemplo). Resumir para "falhou" apagaria a unica parte util.
            memcheckSummary = error !== "" ? error : qsTr("analise dinamica falhou");
            return;
        }
        memcheckSummary = findings > 0
            ? qsTr("%1 achado(s) de memoria em %2 teste(s)").arg(findings).arg(tests)
            : qsTr("nenhum problema de memoria em %1 teste(s)").arg(tests);
    }

    function startAudit() {
        if (auditing || workspaceRoot === "") {
            return;
        }
        removeProblemsBySource("audit");
        auditing = true;
        showTabRequested("problems");
        runAuditRequested();
    }

    function handleAuditDiagnostic(diagnostic) {
        appendDiagnostic(diagnostic, "warning", "audit");
    }

    function handleAuditFinished(success, vulnerabilities, policyFindings, database, error) {
        auditing = false;
        // A politica (cargo-deny) roda OFFLINE e pode ter achado coisa mesmo
        // quando o braco de advisories falhou por falta de base ou de rede.
        // Descartar isso ao reportar a falha esconderia trabalho ja feito.
        const politica = policyFindings > 0
            ? qsTr(" · %1 de politica").arg(policyFindings) : "";
        if (!success) {
            // O erro do core ja diz COMO habilitar a rede; a UI repassa em
            // vez de resumir para "falhou".
            auditSummary = (error !== "" ? error : qsTr("auditoria falhou")) + politica;
            return;
        }
        const base = database && database.lastUpdated !== undefined
                   ? qsTr(" · base de %1").arg(String(database.lastUpdated).slice(0, 10))
                   : "";
        const offline = database && database.offline
                      ? qsTr(" (sem atualizar pela rede)") : "";
        auditSummary = (vulnerabilities > 0
                        ? qsTr("%1 vulnerabilidade(s)").arg(vulnerabilities)
                        : qsTr("nenhuma vulnerabilidade")) + politica + base + offline;
    }

    function startCoverage(buildSystem) {
        if (coverageState.measuring || workspaceRoot === "") {
            return;
        }
        clearCoverage();
        coverageState.measuring = true;
        coverageState.summary = qsTr("medindo cobertura...");
        showTabRequested("tests");
        runCoverageRequested(buildSystem || "");
    }

    function handleCoverageFinished(success, percent, linesCovered, linesTotal,
                                    files, error) {
        coverageState.measuring = false;
        if (!success) {
            coverageState.summary = "";
            // O erro do core ja e acionavel por contrato ("compilou com
            // --coverage?"); a UI repassa e nao inventa um 0% que pareceria
            // medicao.
            coverageState.error = error !== "" ? error : qsTr("cobertura falhou");
            return;
        }
        coverageState.error = "";
        coverageState.summary = qsTr("%1% — %2 de %3 linhas")
                .arg(percent.toFixed(1)).arg(linesCovered).arg(linesTotal);
        for (const file of files) {
            coverageFilesModel.append({
                path: relativeToRoot(file.path),
                percent: file.percent,
                linesCovered: file.linesCovered,
                linesTotal: file.linesTotal
            });
        }
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

    function handleTestCase(name, status) {
        testItemsModel.append({ name: name, status: status });
        while (testItemsModel.count > 5000) {
            testItemsModel.remove(0);
        }
    }

    function handleTestFinished(success, passed, failed, ignored) {
        testSummary = (success
                       ? qsTr("passou: %1")
                       : qsTr("FALHOU — passou: %1")).arg(passed)
                + qsTr("  falhou: %1  ignorado: %2").arg(failed).arg(ignored);
    }

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
