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
    if (handleQualityNotification(method, params)) {
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
        // D2 (docs/roadmaps/24): grid do emulador (cores/cursor/spans) — a UI só desenha.
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
    if (method == QStringLiteral("format.capabilities")) {
        emit formatCapabilitiesListed(
            result.value(QStringLiteral("formatters")).toArray().toVariantList());
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
