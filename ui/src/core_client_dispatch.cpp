#include "core_client.h"

#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonValue>

namespace kinein {

void CoreClient::handleResponseLine(const QByteArray& line)
{
    QJsonParseError parseError{};
    const QJsonDocument document = QJsonDocument::fromJson(line, &parseError);
    if (parseError.error != QJsonParseError::NoError || !document.isObject()) {
        appendErrorLog(
            QStringLiteral("resposta invalida do core: %1").arg(parseError.errorString()));
        return;
    }

    const QJsonObject response = document.object();
    if (!response.contains(QStringLiteral("id"))) {
        handleNotification(response.value(QStringLiteral("method")).toString(),
                           response.value(QStringLiteral("params")).toObject());
        return;
    }

    const qint64 id = response.value(QStringLiteral("id")).toInteger(-1);
    const QString method = m_pendingMethods.take(id);
    const QString requestPath = m_pendingPaths.take(id);

    if (method != QStringLiteral("lsp.didChange") &&
        method != QStringLiteral("lsp.semanticTokens") &&
        method != QStringLiteral("syntaxTree.update"))
    {
        appendLog(QStringLiteral("<- %1").arg(QString::fromUtf8(line.left(200))));
    }

    if (response.contains(QStringLiteral("error"))) {
        const QJsonObject error = response.value(QStringLiteral("error")).toObject();
        const QString message = error.value(QStringLiteral("message")).toString();
        appendLog(QStringLiteral("erro do core em %1: %2").arg(method, message));
        if (method == QStringLiteral("build.run")) {
            setBuilding(false);
            m_buildJobId.clear();
        }
        if (method == QStringLiteral("test.run")) {
            setTesting(false);
            m_testJobId.clear();
        }
        if (method == QStringLiteral("quality.run")) {
            setAnalyzing(false);
            m_qualityJobId.clear();
        }
        if (method == QStringLiteral("environment.scan")) {
            setScanningEnvironment(false);
            m_environmentJobId.clear();
        }
        if (method == QStringLiteral("fs.write")) {
            emit fileSaveFailed(requestPath, message);
        }
        emit requestFailed(method, message);
        return;
    }

    dispatchResult(method, response.value(QStringLiteral("result")).toObject());
}

void CoreClient::handleNotification(const QString& method, const QJsonObject& params)
{
    if (handleFileSystemNotification(method, params)) {
        return;
    }
    if (method == QStringLiteral("event.git.remoteFinished")) {
        emit gitRemoteOperationFinished(params.value(QStringLiteral("operation")).toString(),
                                        params.value(QStringLiteral("success")).toBool(false),
                                        params.value(QStringLiteral("message")).toString());
        return;
    }
    if (method == QStringLiteral("event.build.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("build iniciado: %1").arg(command));
        emit buildStarted(command);
        return;
    }
    if (method == QStringLiteral("event.build.output")) {
        emit buildOutput(params.value(QStringLiteral("line")).toString());
        return;
    }
    if (method == QStringLiteral("event.build.diagnostic")) {
        emit buildDiagnostic(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.build.finished")) {
        const bool success = params.value(QStringLiteral("success")).toBool();
        appendLog(QStringLiteral("build finalizado: %1")
                      .arg(success ? QStringLiteral("sucesso") : QStringLiteral("falha")));
        setBuilding(false);
        m_buildJobId.clear();
        emit buildFinished(success, params.value(QStringLiteral("exitCode")).toInt(-1),
                           params.value(QStringLiteral("diagnostics")).toInt(0));
        return;
    }
    if (method == QStringLiteral("event.test.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("testes iniciados: %1").arg(command));
        emit testStarted(command);
        return;
    }
    if (method == QStringLiteral("event.test.output")) {
        emit testOutput(params.value(QStringLiteral("line")).toString(),
                        params.value(QStringLiteral("stream")).toString());
        return;
    }
    if (method == QStringLiteral("event.test.case")) {
        emit testCase(params.value(QStringLiteral("name")).toString(),
                      params.value(QStringLiteral("status")).toString());
        return;
    }
    if (method == QStringLiteral("event.test.finished")) {
        setTesting(false);
        m_testJobId.clear();
        emit testFinished(params.value(QStringLiteral("success")).toBool(),
                          params.value(QStringLiteral("passed")).toInt(0),
                          params.value(QStringLiteral("failed")).toInt(0),
                          params.value(QStringLiteral("ignored")).toInt(0));
        return;
    }
    if (method == QStringLiteral("event.quality.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("analise iniciada: %1").arg(command));
        emit qualityStarted(command);
        return;
    }
    if (method == QStringLiteral("event.quality.diagnostic")) {
        emit qualityDiagnostic(params.toVariantMap());
        return;
    }
    if (method == QStringLiteral("event.quality.output")) {
        return;
    }
    if (method == QStringLiteral("event.quality.finished")) {
        setAnalyzing(false);
        m_qualityJobId.clear();
        emit qualityFinished(params.value(QStringLiteral("success")).toBool(),
                             params.value(QStringLiteral("exitCode")).toInt(-1),
                             params.value(QStringLiteral("diagnostics")).toInt(0));
        return;
    }
    if (handleCmakeNotification(method, params)) {
        return;
    }
    if (handleDebugNotification(method, params)) {
        return;
    }
    if (method == QStringLiteral("event.run.started")) {
        const QString command = params.value(QStringLiteral("command")).toString();
        appendLog(QStringLiteral("execucao iniciada: %1").arg(command));
        setRunning(true);
        emit runStarted(command);
        return;
    }
    if (method == QStringLiteral("event.run.output")) {
        emit runOutput(params.value(QStringLiteral("line")).toString(),
                       params.value(QStringLiteral("stream")).toString());
        return;
    }
    if (method == QStringLiteral("event.run.finished")) {
        setRunning(false);
        emit runFinished(params.value(QStringLiteral("success")).toBool(),
                         params.value(QStringLiteral("exitCode")).toInt(-1));
        return;
    }
    if (handleTerminalNotification(method, params)) {
        return;
    }
    if (handleLspNotification(method, params)) {
        return;
    }
    if (handleEnvironmentNotification(method, params)) {
        return;
    }
    if (handleJobNotification(method, params)) {
        return;
    }
}

bool CoreClient::handleFileSystemNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.fs.changed")) {
        emit filesChanged(params.value(QStringLiteral("changes")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("event.fs.watchError")) {
        const QString message = params.value(QStringLiteral("message")).toString();
        appendErrorLog(QStringLiteral("watcher do workspace: %1").arg(message));
        emit fileWatchFailed(message);
        return true;
    }
    return false;
}

bool CoreClient::handleTerminalNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.terminal.render")) {
        // D2 (docs/24): grid do emulador (cores/cursor/spans) — a UI só desenha.
        emit terminalRender(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.terminal.closed")) {
        // D2.3: eventos atrasados de workspaces anteriores não podem alterar o
        // estado das sessões atuais; por isso removemos o ID exato.
        const QString id = params.value(QStringLiteral("id")).toString();
        m_terminalIds.remove(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit terminalClosed(id, params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    return false;
}

bool CoreClient::handleLspNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.lsp.diagnostics")) {
        emit lspDiagnostics(params.value(QStringLiteral("path")).toString(),
                            params.value(QStringLiteral("diagnostics")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("event.lsp.status")) {
        const QString language = params.value(QStringLiteral("language")).toString();
        const QString status = params.value(QStringLiteral("status")).toString();
        const QString message = params.value(QStringLiteral("message")).toString();
        if (status == QStringLiteral("failed")) {
            appendErrorLog(QStringLiteral("lsp %1: %2 (%3)").arg(language, status, message));
        }
        else {
            appendLog(QStringLiteral("lsp %1: %2").arg(language, status));
        }
        return true;
    }
    if (method == QStringLiteral("event.lsp.restarted")) {
        // M4.3b: servidor reiniciou (auto por timeouts ou lsp.restart) — a UI
        // re-sincroniza o arquivo ativo (mesmo caminho do recovered()).
        emit lspRestarted(params.value(QStringLiteral("language")).toString());
        return true;
    }
    return false;
}

bool CoreClient::handleDebugNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.debug.started")) {
        const QString program = params.value(QStringLiteral("program")).toString();
        appendLog(QStringLiteral("debug iniciado: %1").arg(program));
        setDebugging(true);
        emit debugStarted(program);
        return true;
    }
    if (method == QStringLiteral("event.debug.output")) {
        emit debugOutput(params.value(QStringLiteral("category")).toString(),
                         params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.debug.stopped")) {
        emit debugStopped(params.value(QStringLiteral("reason")).toString(),
                          params.value(QStringLiteral("file")).toString(),
                          params.value(QStringLiteral("line")).toInt(0),
                          params.value(QStringLiteral("threadId")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.debug.continued")) {
        emit debugContinued();
        return true;
    }
    if (method == QStringLiteral("event.debug.finished")) {
        setDebugging(false);
        emit debugFinished(params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    return false;
}

bool CoreClient::handleEnvironmentNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.environment.started")) {
        setScanningEnvironment(true);
        appendLog(QStringLiteral("scan de ambiente iniciado"));
        emit environmentScanStarted(params.value(QStringLiteral("tools")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.environment.tool")) {
        emit environmentTool(params.value(QStringLiteral("tool")).toObject().toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.environment.finished")) {
        setScanningEnvironment(false);
        m_environmentJobId.clear();
        const QVariantList tools = params.value(QStringLiteral("tools")).toArray().toVariantList();
        appendLog(QStringLiteral("scan de ambiente finalizado"));
        emit environmentScanFinished(params.value(QStringLiteral("success")).toBool(),
                                     params.value(QStringLiteral("total")).toInt(0),
                                     params.value(QStringLiteral("detected")).toInt(0),
                                     params.value(QStringLiteral("missing")).toInt(0),
                                     params.value(QStringLiteral("failed")).toInt(0), tools);
        emit toolsListed(tools);
        return true;
    }
    return false;
}

bool CoreClient::handleJobNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.job.created")) {
        emit jobCreated(params.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("event.job.progress")) {
        emit jobProgress(params.value(QStringLiteral("jobId")).toString(),
                         params.value(QStringLiteral("status")).toString(),
                         params.value(QStringLiteral("progress")).toDouble(0.0),
                         params.value(QStringLiteral("message")).toString());
        return true;
    }
    if (method == QStringLiteral("event.job.output")) {
        emit jobOutput(params.value(QStringLiteral("jobId")).toString(),
                       params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.job.finished")) {
        emit jobFinished(params.value(QStringLiteral("jobId")).toString(),
                         params.value(QStringLiteral("status")).toString());
        return true;
    }
    return false;
}

bool CoreClient::dispatchCmakeResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("runConfig.list") || method == QStringLiteral("runConfig.save") ||
        method == QStringLiteral("runConfig.delete") ||
        method == QStringLiteral("runConfig.setActive"))
    {
        emit runConfigsResolved(result.value(QStringLiteral("configs")).toArray().toVariantList(),
                                result.value(QStringLiteral("activeId")).toString());
        return true;
    }
    if (method == QStringLiteral("cargo.metadata")) {
        emit cargoMetadataResolved(
            static_cast<int>(result.value(QStringLiteral("packages")).toArray().size()));
        return true;
    }
    if (method == QStringLiteral("cargo.check")) {
        appendLog(QStringLiteral("job aceito (cargo.check): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
        return true;
    }
    if (method == QStringLiteral("cmake.status")) {
        emit cmakeStatusResolved(result.value(QStringLiteral("configured")).toBool(),
                                 result.value(QStringLiteral("hasCompileCommands")).toBool());
        return true;
    }
    if (method == QStringLiteral("cmake.configure")) {
        appendLog(QStringLiteral("job aceito (cmake.configure): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
        return true;
    }
    return false;
}

bool CoreClient::dispatchFileResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("fs.list")) {
        emit dirListed(result.value(QStringLiteral("path")).toString(),
                       result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("fs.read")) {
        emit fileLoaded(result.value(QStringLiteral("path")).toString(),
                        result.value(QStringLiteral("content")).toString());
        return true;
    }
    if (method == QStringLiteral("fs.createFile")) {
        emit fileCreated(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("fs.createDirectory")) {
        emit directoryCreated(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("fs.write")) {
        emit fileSaved(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("format.text")) {
        emit fileFormatted(result.value(QStringLiteral("path")).toString(),
                           result.value(QStringLiteral("text")).toString(),
                           result.value(QStringLiteral("changed")).toBool());
        return true;
    }
    if (method == QStringLiteral("fs.rename")) {
        emit pathRenamed(result.value(QStringLiteral("from")).toString(),
                         result.value(QStringLiteral("to")).toString());
        return true;
    }
    if (method == QStringLiteral("fs.delete")) {
        emit pathDeleted(result.value(QStringLiteral("path")).toString());
        return true;
    }
    return false;
}

bool CoreClient::handleCmakeNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.cmake.started")) {
        appendLog(QStringLiteral("cmake configure iniciado: %1")
                      .arg(params.value(QStringLiteral("command")).toString()));
        return true;
    }
    if (method == QStringLiteral("event.cmake.finished")) {
        const bool success = params.value(QStringLiteral("success")).toBool();
        appendLog(QStringLiteral("cmake configure finalizado (sucesso: %1)")
                      .arg(success ? QStringLiteral("sim") : QStringLiteral("nao")));
        emit cmakeConfigureFinished(success);
        if (m_workspaceBuildSystems.contains(QStringLiteral("cmake"))) {
            cmakeStatus();
        }
        return true;
    }
    return false;
}

bool CoreClient::dispatchDebugResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("debug.start")) {
        appendLog(QStringLiteral("debug aceito: %1")
                      .arg(result.value(QStringLiteral("program")).toString()));
        return true;
    }
    if (method == QStringLiteral("git.status") || method == QStringLiteral("git.stage") ||
        method == QStringLiteral("git.unstage") || method == QStringLiteral("git.discard") ||
        method == QStringLiteral("git.commit") || method == QStringLiteral("git.checkout") ||
        method == QStringLiteral("git.branchCreate") || method == QStringLiteral("git.stash"))
    {
        emit gitStatusResolved(result.value(QStringLiteral("repo")).toBool(),
                               result.value(QStringLiteral("branch")).toString(),
                               result.value(QStringLiteral("detached")).toBool(),
                               result.value(QStringLiteral("shortSha")).toString(),
                               result.value(QStringLiteral("ahead")).toInt(0),
                               result.value(QStringLiteral("behind")).toInt(0),
                               result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.branches")) {
        emit gitBranchesResolved(
            result.value(QStringLiteral("repo")).toBool(),
            result.value(QStringLiteral("branches")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.fileDiff")) {
        emit gitFileDiffResolved(result.value(QStringLiteral("path")).toString(),
                                 result.value(QStringLiteral("repo")).toBool(),
                                 result.value(QStringLiteral("tracked")).toBool(),
                                 result.value(QStringLiteral("hunks")).toArray().toVariantList(),
                                 result.value(QStringLiteral("text")).toString());
        return true;
    }
    if (method == QStringLiteral("git.blame")) {
        emit gitBlameResolved(result.value(QStringLiteral("path")).toString(),
                              result.value(QStringLiteral("repo")).toBool(),
                              result.value(QStringLiteral("tracked")).toBool(),
                              result.value(QStringLiteral("groups")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.log")) {
        emit gitLogResolved(result.value(QStringLiteral("repo")).toBool(),
                            result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.commitDiff")) {
        emit gitCommitDiffResolved(result.value(QStringLiteral("sha")).toString(),
                                   result.value(QStringLiteral("text")).toString());
        return true;
    }
    if (method == QStringLiteral("settings.get") || method == QStringLiteral("settings.set")) {
        emit settingsResolved(result.value(QStringLiteral("settings")).toObject().toVariantMap(),
                              result.value(QStringLiteral("global")).toObject().toVariantMap(),
                              result.value(QStringLiteral("workspace")).toObject().toVariantMap());
        return true;
    }
    if (method == QStringLiteral("debug.stackTrace")) {
        emit debugStackTraceResolved(
            result.value(QStringLiteral("frames")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("debug.variables")) {
        emit debugVariablesResolved(
            result.value(QStringLiteral("frameId")).toDouble(-1),
            result.value(QStringLiteral("ref")).toDouble(-1),
            result.value(QStringLiteral("variables")).toArray().toVariantList());
        return true;
    }
    // O estado da UI vem dos eventos event.debug.*; as respostas de
    // controle/breakpoints nao carregam nada que a UI ja nao saiba.
    return method == QStringLiteral("debug.setBreakpoints") ||
           method == QStringLiteral("debug.continue") || method == QStringLiteral("debug.next") ||
           method == QStringLiteral("debug.stepIn") || method == QStringLiteral("debug.stepOut") ||
           method == QStringLiteral("debug.pause") || method == QStringLiteral("debug.stop");
}

bool CoreClient::dispatchWorkspaceResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("workspace.open") ||
        method == QStringLiteral("workspace.createProject"))
    {
        handleWorkspaceOpened(result);
        return true;
    }
    if (method.startsWith(QStringLiteral("workspace.recent."))) {
        emit recentWorkspacesResolved(
            result.value(QStringLiteral("workspaces")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("workspace.createFolder")) {
        emit workspaceFolderCreated(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("workspace.browse")) {
        emit workspaceBrowseListed(
            result.value(QStringLiteral("path")).toString(),
            result.value(QStringLiteral("parent")).toString(),
            result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("workspace.close")) {
        m_workspaceRoot.clear();
        m_workspaceName.clear();
        m_workspaceKind.clear();
        m_workspaceBuildSystems.clear();
        m_terminalIds.clear();
        setTerminalActive(false);
        emit workspaceChanged();
        return true;
    }
    return false;
}

void CoreClient::handleWorkspaceOpened(const QJsonObject& result)
{
    m_workspaceRoot = result.value(QStringLiteral("root")).toString();
    m_workspaceName = result.value(QStringLiteral("name")).toString();
    m_workspaceKind = result.value(QStringLiteral("kind")).toString();
    m_workspaceBuildSystems.clear();
    const QJsonArray buildSystems = result.value(QStringLiteral("capabilities"))
                                        .toObject()
                                        .value(QStringLiteral("buildSystems"))
                                        .toArray();
    for (const QJsonValue buildSystem : buildSystems) {
        const QString value = buildSystem.toString();
        if (!value.isEmpty() && !m_workspaceBuildSystems.contains(value)) {
            m_workspaceBuildSystems.append(value);
        }
    }
    // Tolerate an older core response during crash recovery; protocol version
    // negotiation still prevents unsupported requests in normal operation.
    if (m_workspaceBuildSystems.isEmpty()) {
        if (m_workspaceKind == QStringLiteral("rustCargo")) {
            m_workspaceBuildSystems.append(QStringLiteral("cargo"));
        }
        else if (m_workspaceKind == QStringLiteral("cmake")) {
            m_workspaceBuildSystems.append(QStringLiteral("cmake"));
        }
    }
    // M4.3: lembra o root para recuperar de um crash futuro.
    m_lastWorkspaceRoot = m_workspaceRoot;
    emit workspaceChanged();
    listRecentWorkspaces();
    listDir(m_workspaceRoot);
    if (m_workspaceBuildSystems.contains(QStringLiteral("cmake"))) {
        cmakeStatus();
    }
    if (m_workspaceBuildSystems.contains(QStringLiteral("cargo"))) {
        cargoMetadata();
    }
    runConfigList();
    gitStatus();
    // Na RECUPERACAO, NAO restaurar a sessao (as abas/edicoes ja estao na
    // UI; reler do disco sobrescreveria edicoes nao salvas). Reabrir o mesmo
    // root nao limpa a UI (WorkspaceController).
    if (m_recovering) {
        setRecovering(false);
        m_recoveryAttempts = 0;
        appendLog(QStringLiteral("core recuperado; workspace reconectado"));
        emit recovered();
        return;
    }
    if (result.contains(QStringLiteral("session"))) {
        const QJsonObject session = result.value(QStringLiteral("session")).toObject();
        QStringList files;
        const QJsonArray sessionFiles = session.value(QStringLiteral("openFiles")).toArray();
        files.reserve(sessionFiles.size());
        for (const QJsonValue file : sessionFiles) {
            files.append(file.toString());
        }
        if (!files.isEmpty()) {
            emit sessionRestored(files, session.value(QStringLiteral("activeFile")).toString());
        }
    }
    // M-S1: rascunhos não salvos recuperados de um crash (docs/23). Só em
    // abertura normal — a recuperação de crash do core (acima) sai antes.
    if (result.contains(QStringLiteral("drafts"))) {
        const QVariantList drafts =
            result.value(QStringLiteral("drafts")).toArray().toVariantList();
        if (!drafts.isEmpty()) {
            emit draftsRecovered(drafts);
        }
    }
}

void CoreClient::dispatchResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("core.ping")) {
        m_protocolVersion = result.value(QStringLiteral("protocolVersion")).toString();
        setStatus(QStringLiteral("conectado"), true);
        return;
    }
    if (dispatchWorkspaceResult(method, result)) {
        return;
    }
    if (dispatchCmakeResult(method, result) || dispatchDebugResult(method, result)) {
        return;
    }
    if (method == QStringLiteral("tools.detect") || method == QStringLiteral("tools.status")) {
        emit toolsListed(result.value(QStringLiteral("tools")).toArray().toVariantList());
        return;
    }
    if (method == QStringLiteral("build.run") || method == QStringLiteral("test.run") ||
        method == QStringLiteral("quality.run") || method == QStringLiteral("environment.scan") ||
        method == QStringLiteral("git.pull") || method == QStringLiteral("git.push"))
    {
        const QString jobId = result.value(QStringLiteral("jobId")).toString();
        appendLog(QStringLiteral("job aceito (%1): %2").arg(method, jobId));
        if (!method.startsWith(QStringLiteral("git."))) {
            storeJobId(method, jobId);
        }
        return;
    }
    if (method == QStringLiteral("command.list")) {
        emit commandsListed(result.value(QStringLiteral("commands")).toArray().toVariantList());
        return;
    }
    if (dispatchFileResult(method, result) || dispatchSyntaxResult(method, result) ||
        dispatchLspResult(method, result))
    {
        return;
    }
    if (method == QStringLiteral("terminal.open")) {
        const QString id = result.value(QStringLiteral("id")).toString();
        const QString shell = result.value(QStringLiteral("shell")).toString();
        m_terminalIds.insert(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit terminalOpened(id, shell);
        appendLog(QStringLiteral("terminal aberto (%1): %2").arg(id, shell));
        return;
    }
    if (method == QStringLiteral("aiBridge.profiles")) {
        emit aiProfilesResolved(result.value(QStringLiteral("profiles")).toArray().toVariantList(),
                                result.value(QStringLiteral("defaultProfile")).toString());
        return;
    }
    if (method == QStringLiteral("aiBridge.terminal.open")) {
        const QString id = result.value(QStringLiteral("id")).toString();
        const QString profileId = result.value(QStringLiteral("profileId")).toString();
        const QString name = result.value(QStringLiteral("name")).toString();
        const QString command = result.value(QStringLiteral("command")).toString();
        m_terminalIds.insert(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit aiTerminalOpened(id, profileId, name, command);
        appendLog(QStringLiteral("AI Terminal aberto (%1): %2").arg(id, command));
        return;
    }
    if (method == QStringLiteral("fs.findFiles")) {
        emit fileSearchResults(result.value(QStringLiteral("matches")).toArray().toVariantList(),
                               result.value(QStringLiteral("truncated")).toBool(false));
        return;
    }
    if (method == QStringLiteral("fs.search")) {
        emit searchResults(result.value(QStringLiteral("matches")).toArray().toVariantList(),
                           result.value(QStringLiteral("truncated")).toBool(false));
        return;
    }
    if (method == QStringLiteral("fs.replace")) {
        QStringList files;
        const QJsonArray fileArray = result.value(QStringLiteral("files")).toArray();
        files.reserve(fileArray.size());
        for (const QJsonValue file : fileArray) {
            files.append(file.toString());
        }
        emit filesReplaced(files, result.value(QStringLiteral("replacements")).toInt(0));
    }
}

bool CoreClient::dispatchSyntaxResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("syntaxTree.update")) {
        return false;
    }
    emit syntaxTreeResolved(result.value(QStringLiteral("path")).toString(),
                            result.value(QStringLiteral("version")).toInt(),
                            result.value(QStringLiteral("language")).toString(),
                            result.value(QStringLiteral("hasErrors")).toBool(),
                            result.value(QStringLiteral("highlights")).toArray().toVariantList(),
                            result.value(QStringLiteral("foldingRanges")).toArray().toVariantList(),
                            result.value(QStringLiteral("outline")).toArray().toVariantList(),
                            result.value(QStringLiteral("locals")).toArray().toVariantList());
    return true;
}

bool CoreClient::dispatchLspResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("lsp.definition")) {
        const QString path = result.value(QStringLiteral("path")).toString();
        if (!path.isEmpty()) {
            emit lspDefinitionResolved(path, result.value(QStringLiteral("line")).toInt(1),
                                       result.value(QStringLiteral("column")).toInt(1));
        }
        return true;
    }
    if (method == QStringLiteral("lsp.hover")) {
        emit lspHoverResolved(result.value(QStringLiteral("content")).toString());
        return true;
    }
    if (method == QStringLiteral("lsp.completion")) {
        emit lspCompletionResolved(result.value(QStringLiteral("items")).toArray().toVariantList(),
                                   result.value(QStringLiteral("isIncomplete")).toBool());
        return true;
    }
    if (method == QStringLiteral("lsp.references")) {
        emit lspReferencesResolved(
            result.value(QStringLiteral("references")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.semanticTokens")) {
        emit lspSemanticTokensResolved(
            result.value(QStringLiteral("path")).toString(),
            result.value(QStringLiteral("version")).toInt(),
            result.value(QStringLiteral("tokens")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.documentSymbols") ||
        method == QStringLiteral("lsp.workspaceSymbols"))
    {
        emit lspSymbolsResolved(result.value(QStringLiteral("symbols")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.switchSourceHeader")) {
        // path ausente = clangd nao achou contraparte (nao e erro).
        emit lspSwitchSourceHeaderResolved(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("lsp.restart")) {
        // A re-sincronizacao vem por event.lsp.restarted; aqui so registramos.
        const QStringList restarted =
            result.value(QStringLiteral("restarted")).toVariant().toStringList();
        appendLog(
            restarted.isEmpty()
                ? QStringLiteral("lsp.restart: nenhum servidor ativo")
                : QStringLiteral("lsp.restart: %1").arg(restarted.join(QStringLiteral(", "))));
        return true;
    }
    if (method == QStringLiteral("lsp.codeActions")) {
        emit lspCodeActionsResolved(
            result.value(QStringLiteral("actions")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.applyCodeAction") || method == QStringLiteral("lsp.rename")) {
        emit lspWorkspaceEditPreviewResolved(
            result.value(QStringLiteral("transactionId")).toString(),
            result.value(QStringLiteral("title")).toString(),
            result.value(QStringLiteral("files")).toArray().toVariantList(),
            result.value(QStringLiteral("edits")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("lsp.workspaceEdit.apply")) {
        QStringList files;
        const QJsonArray fileArray = result.value(QStringLiteral("files")).toArray();
        files.reserve(fileArray.size());
        for (const QJsonValue file : fileArray) {
            files.append(file.toString());
        }
        emit lspWorkspaceEditApplied(files, result.value(QStringLiteral("title")).toString(),
                                     result.value(QStringLiteral("edits")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("lsp.workspaceEdit.cancel")) {
        emit lspWorkspaceEditCancelled(result.value(QStringLiteral("transactionId")).toString());
        return true;
    }
    return false;
}

void CoreClient::storeJobId(const QString& method, const QString& jobId)
{
    if (method == QStringLiteral("build.run")) {
        m_buildJobId = jobId;
    }
    else if (method == QStringLiteral("test.run")) {
        m_testJobId = jobId;
    }
    else if (method == QStringLiteral("quality.run")) {
        m_qualityJobId = jobId;
    }
    else {
        m_environmentJobId = jobId;
    }
}

} // namespace kinein
