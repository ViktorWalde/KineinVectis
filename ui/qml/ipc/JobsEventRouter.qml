import QtQuick

Item {
    id: root

    property var coreClient: null
    property var jobsController: null
    property var diagnosticsController: null

    visible: false

    Connections {
        target: root.coreClient

        function onBuildStarted(command) {
            root.jobsController.handleBuildStarted(command);
        }

        function onBuildOutput(line) {
            root.jobsController.handleBuildOutput(line);
        }

        function onBuildDiagnostic(diagnostic) {
            root.jobsController.handleBuildDiagnostic(diagnostic);
        }

        function onBuildFinished(success, exitCode, diagnostics) {
            root.jobsController.handleBuildFinished(success, exitCode);
        }

        function onLspDiagnostics(path, diagnostics) {
            root.jobsController.handleLspDiagnostics(path, diagnostics);
            root.diagnosticsController.handleLspDiagnostics(path, diagnostics);
        }

        function onTestCase(name, status) {
            root.jobsController.handleTestCase(name, status);
        }

        function onTestFinished(success, passed, failed, ignored) {
            root.jobsController.handleTestFinished(success, passed, failed, ignored);
        }

        function onQualityDiagnostic(diagnostic) {
            root.jobsController.handleQualityDiagnostic(diagnostic);
        }

        function onQualityFinished(success, exitCode, diagnostics) {
            root.jobsController.handleQualityFinished();
        }

        function onAuditDiagnostic(diagnostic) {
            root.jobsController.handleAuditDiagnostic(diagnostic);
        }

        function onAuditFinished(success, vulnerabilities, policyFindings, database, error) {
            root.jobsController.handleAuditFinished(
                success, vulnerabilities, policyFindings, database, error);
        }

        function onMemcheckDiagnostic(diagnostic) {
            root.jobsController.handleMemcheckDiagnostic(diagnostic);
        }

        function onMemcheckFinished(success, tests, findings, error) {
            root.jobsController.handleMemcheckFinished(success, tests, findings, error);
        }

        function onCoverageFinished(success, percent, linesCovered, linesTotal,
                                    files, error) {
            root.jobsController.handleCoverageFinished(
                success, percent, linesCovered, linesTotal, files, error);
        }

        function onJobCreated(job) {
            root.jobsController.handleJobCreated(job);
        }

        function onJobProgress(jobId, status, progress, message) {
            root.jobsController.handleJobProgress(jobId, status, progress, message);
        }

        function onJobOutput(jobId, line) {
            root.jobsController.handleJobOutput(jobId, line);
        }

        function onJobFinished(jobId, status) {
            root.jobsController.handleJobFinished(jobId, status);
        }
    }
}
