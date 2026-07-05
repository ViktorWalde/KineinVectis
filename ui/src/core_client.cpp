#include "core_client.h"

#include <QCoreApplication>
#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonValue>
#include <QStandardPaths>
#include <QTextStream>
#include <QTime>

namespace kinein {

namespace {

constexpr int kMaxLogLines = 500;

QString timestamp()
{
    return QTime::currentTime().toString(QStringLiteral("HH:mm:ss.zzz"));
}

} // namespace

CoreClient::CoreClient(QObject* parent) : QObject(parent), m_status(QStringLiteral("desconectado"))
{
    connect(&m_process, &QProcess::started, this, &CoreClient::handleStarted);
    connect(&m_process, &QProcess::readyReadStandardOutput, this, &CoreClient::handleStdout);
    connect(&m_process, &QProcess::readyReadStandardError, this, &CoreClient::handleStderr);
    connect(&m_process, &QProcess::finished, this, &CoreClient::handleFinished);
    connect(&m_process, &QProcess::errorOccurred, this, &CoreClient::handleErrorOccurred);
}

CoreClient::~CoreClient()
{
    // Fecha o core de forma limpa ao encerrar a IDE. Sem isto, o QProcess e
    // destruido com o filho ainda vivo ("Destroyed while process is still
    // running"), o que pode corromper o heap no teardown. Os sinais sao
    // desconectados antes para nenhum slot tocar em membros ja destruidos.
    m_process.disconnect(this);
    if (m_process.state() == QProcess::NotRunning) {
        return;
    }
    // Fecha o stdin: o loop do core le linhas de stdin e sai no EOF.
    m_process.closeWriteChannel();
    if (m_process.waitForFinished(2000)) {
        return;
    }
    m_process.terminate();
    if (!m_process.waitForFinished(1000)) {
        m_process.kill();
        m_process.waitForFinished(1000);
    }
}

QString CoreClient::status() const
{
    return m_status;
}

bool CoreClient::isConnected() const
{
    return m_connected;
}

QString CoreClient::protocolVersion() const
{
    return m_protocolVersion;
}

QStringList CoreClient::logLines() const
{
    return m_logLines;
}

QString CoreClient::workspaceRoot() const
{
    return m_workspaceRoot;
}

QString CoreClient::workspaceName() const
{
    return m_workspaceName;
}

QString CoreClient::workspaceKind() const
{
    return m_workspaceKind;
}

QString CoreClient::homeDir()
{
    return QDir::homePath();
}

QString CoreClient::errorLogFile()
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation) +
           QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt");
}

void CoreClient::start()
{
    if (m_process.state() != QProcess::NotRunning) {
        return;
    }

    const QString binary = resolveCoreBinary();
    if (binary.isEmpty()) {
        setStatus(QStringLiteral("kinein-core nao encontrado"), false);
        appendErrorLog(
            QStringLiteral("erro: kinein-core nao foi encontrado. Compile com "
                           "'cargo build -p kinein-core' ou defina KINEIN_CORE_BIN."));
        return;
    }

    setStatus(QStringLiteral("iniciando..."), false);
    appendLog(QStringLiteral("iniciando core: %1").arg(binary));
    m_process.setProgram(binary);
    m_process.setArguments({});
    m_process.start();
}

void CoreClient::ping()
{
    sendRequest(QStringLiteral("core.ping"), QJsonObject{});
}

void CoreClient::openWorkspace(const QString& path)
{
    sendRequest(QStringLiteral("workspace.open"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::browseWorkspaceFolders(const QString& path)
{
    sendRequest(QStringLiteral("workspace.browse"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::createWorkspaceFolder(const QString& parent, const QString& name)
{
    sendRequest(QStringLiteral("workspace.createFolder"),
                QJsonObject{{QStringLiteral("parent"), parent}, {QStringLiteral("name"), name}});
}

void CoreClient::createWorkspaceProject(const QString& parent, const QString& name,
                                        const QString& templateId)
{
    sendRequest(QStringLiteral("workspace.createProject"),
                QJsonObject{{QStringLiteral("parent"), parent},
                            {QStringLiteral("name"), name},
                            {QStringLiteral("template"), templateId}});
}

void CoreClient::closeWorkspace()
{
    sendRequest(QStringLiteral("workspace.close"), QJsonObject{});
}

void CoreClient::listDir(const QString& path)
{
    sendRequest(QStringLiteral("fs.list"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::readFile(const QString& path)
{
    sendRequest(QStringLiteral("fs.read"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::createFile(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("fs.createFile"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::createDirectory(const QString& path)
{
    sendRequest(QStringLiteral("fs.createDirectory"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::writeFile(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("fs.write"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::renamePath(const QString& from, const QString& to)
{
    sendRequest(QStringLiteral("fs.rename"),
                QJsonObject{{QStringLiteral("from"), from}, {QStringLiteral("to"), to}});
}

void CoreClient::deletePath(const QString& path)
{
    sendRequest(QStringLiteral("fs.delete"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::listCommands()
{
    sendRequest(QStringLiteral("command.list"), QJsonObject{});
}

void CoreClient::detectTools()
{
    sendRequest(QStringLiteral("tools.detect"), QJsonObject{});
}

bool CoreClient::isBuilding() const
{
    return m_building;
}

void CoreClient::notifyFileChanged(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.didChange"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::requestDefinition(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.definition"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestHover(const QString& path, const QString& content, int line, int column)
{
    sendRequest(QStringLiteral("lsp.hover"), QJsonObject{{QStringLiteral("path"), path},
                                                         {QStringLiteral("content"), content},
                                                         {QStringLiteral("line"), line},
                                                         {QStringLiteral("column"), column}});
}

void CoreClient::requestCompletion(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.completion"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestReferences(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.references"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestRename(const QString& path, const QString& content, int line, int column,
                               const QString& newName)
{
    sendRequest(QStringLiteral("lsp.rename"), QJsonObject{{QStringLiteral("path"), path},
                                                          {QStringLiteral("content"), content},
                                                          {QStringLiteral("line"), line},
                                                          {QStringLiteral("column"), column},
                                                          {QStringLiteral("newName"), newName}});
}

void CoreClient::requestSemanticTokens(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.semanticTokens"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::findFiles(const QString& query)
{
    sendRequest(QStringLiteral("fs.findFiles"), QJsonObject{{QStringLiteral("query"), query}});
}

void CoreClient::searchInFiles(const QString& query, bool caseSensitive)
{
    sendRequest(QStringLiteral("fs.search"),
                QJsonObject{{QStringLiteral("query"), query},
                            {QStringLiteral("caseSensitive"), caseSensitive}});
}

void CoreClient::runBuild()
{
    if (m_building || m_process.state() != QProcess::Running) {
        return;
    }
    setBuilding(true);
    sendRequest(QStringLiteral("build.run"), QJsonObject{});
}

void CoreClient::setBuilding(bool building)
{
    if (m_building == building) {
        return;
    }
    m_building = building;
    emit buildingChanged();
}

void CoreClient::runTests(const QString& filter)
{
    if (m_testing || m_process.state() != QProcess::Running) {
        return;
    }
    QJsonObject params;
    if (!filter.trimmed().isEmpty()) {
        params.insert(QStringLiteral("filter"), filter);
    }
    setTesting(true);
    sendRequest(QStringLiteral("test.run"), params);
}

void CoreClient::runQuality()
{
    if (m_analyzing || m_process.state() != QProcess::Running) {
        return;
    }
    setAnalyzing(true);
    sendRequest(QStringLiteral("quality.run"), QJsonObject{});
}

bool CoreClient::isAnalyzing() const
{
    return m_analyzing;
}

void CoreClient::setAnalyzing(bool analyzing)
{
    if (m_analyzing == analyzing) {
        return;
    }
    m_analyzing = analyzing;
    emit analyzingChanged();
}

bool CoreClient::isTesting() const
{
    return m_testing;
}

void CoreClient::setTesting(bool testing)
{
    if (m_testing == testing) {
        return;
    }
    m_testing = testing;
    emit testingChanged();
}

bool CoreClient::isRunning() const
{
    return m_running;
}

void CoreClient::setRunning(bool running)
{
    if (m_running == running) {
        return;
    }
    m_running = running;
    emit runningChanged();
}

void CoreClient::runStart(const QString& command)
{
    QJsonObject params;
    if (!command.trimmed().isEmpty()) {
        params.insert(QStringLiteral("command"), command);
    }
    sendRequest(QStringLiteral("run.start"), params);
}

void CoreClient::runStdin(const QString& data)
{
    sendRequest(QStringLiteral("run.stdin"), QJsonObject{{QStringLiteral("data"), data}});
}

void CoreClient::runStop()
{
    sendRequest(QStringLiteral("run.stop"), QJsonObject{});
}

bool CoreClient::isTerminalActive() const
{
    return m_terminalActive;
}

void CoreClient::setTerminalActive(bool active)
{
    if (m_terminalActive == active) {
        return;
    }
    m_terminalActive = active;
    emit terminalActiveChanged();
}

void CoreClient::terminalOpen()
{
    sendRequest(QStringLiteral("terminal.open"), QJsonObject{});
}

void CoreClient::terminalInput(const QString& data)
{
    sendRequest(QStringLiteral("terminal.input"), QJsonObject{{QStringLiteral("data"), data}});
}

void CoreClient::terminalClose()
{
    sendRequest(QStringLiteral("terminal.close"), QJsonObject{});
}

void CoreClient::handleStarted()
{
    appendLog(QStringLiteral("processo do core iniciado (pid %1)").arg(m_process.processId()));
    ping();
}

void CoreClient::handleStdout()
{
    m_stdoutBuffer.append(m_process.readAllStandardOutput());

    qsizetype newline = m_stdoutBuffer.indexOf('\n');
    while (newline >= 0) {
        const QByteArray line = m_stdoutBuffer.left(newline);
        m_stdoutBuffer.remove(0, newline + 1);
        if (!line.trimmed().isEmpty()) {
            handleResponseLine(line);
        }
        newline = m_stdoutBuffer.indexOf('\n');
    }
}

void CoreClient::handleStderr()
{
    const QString text = QString::fromUtf8(m_process.readAllStandardError()).trimmed();
    if (!text.isEmpty()) {
        appendErrorLog(QStringLiteral("core stderr: %1").arg(text));
    }
}

void CoreClient::handleFinished(int exitCode, QProcess::ExitStatus exitStatus)
{
    if (exitStatus == QProcess::NormalExit) {
        appendLog(QStringLiteral("core finalizou: saida normal, codigo %1").arg(exitCode));
    }
    else {
        appendErrorLog(QStringLiteral("core finalizou: crash"));
    }
    m_pendingMethods.clear();
    setBuilding(false);
    setTesting(false);
    setAnalyzing(false);
    setRunning(false);
    setTerminalActive(false);
    setStatus(QStringLiteral("desconectado"), false);
}

void CoreClient::handleErrorOccurred(QProcess::ProcessError error)
{
    appendErrorLog(QStringLiteral("erro de processo: %1").arg(m_process.errorString()));
    if (error == QProcess::FailedToStart) {
        setStatus(QStringLiteral("falha ao iniciar o core"), false);
    }
}

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

    // Notificacoes (eventos) nao tem id e nao poluem o log da IDE linha a
    // linha; o fluxo do build vai para a aba Build.
    if (!response.contains(QStringLiteral("id"))) {
        handleNotification(response.value(QStringLiteral("method")).toString(),
                           response.value(QStringLiteral("params")).toObject());
        return;
    }

    const qint64 id = response.value(QStringLiteral("id")).toInteger(-1);
    const QString method = m_pendingMethods.take(id);

    if (method != QStringLiteral("lsp.didChange") && method != QStringLiteral("lsp.semanticTokens"))
    {
        appendLog(QStringLiteral("<- %1").arg(QString::fromUtf8(line.left(200))));
    }

    if (response.contains(QStringLiteral("error"))) {
        const QJsonObject error = response.value(QStringLiteral("error")).toObject();
        const QString message = error.value(QStringLiteral("message")).toString();
        appendLog(QStringLiteral("erro do core em %1: %2").arg(method, message));
        if (method == QStringLiteral("build.run")) {
            setBuilding(false);
        }
        if (method == QStringLiteral("test.run")) {
            setTesting(false);
        }
        if (method == QStringLiteral("quality.run")) {
            setAnalyzing(false);
        }
        emit requestFailed(method, message);
        return;
    }

    dispatchResult(method, response.value(QStringLiteral("result")).toObject());
}

void CoreClient::handleNotification(const QString& method, const QJsonObject& params)
{
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
        const QString status = params.value(QStringLiteral("success")).toBool()
                                   ? QStringLiteral("sucesso")
                                   : QStringLiteral("falha");
        appendLog(QStringLiteral("build finalizado: %1").arg(status));
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
    if (method == QStringLiteral("event.quality.output") ||
        method == QStringLiteral("event.quality.finished"))
    {
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
    if (method == QStringLiteral("event.terminal.data")) {
        emit terminalData(params.value(QStringLiteral("data")).toString());
        return;
    }
    if (method == QStringLiteral("event.terminal.closed")) {
        setTerminalActive(false);
        emit terminalClosed(params.value(QStringLiteral("exitCode")).toInt(-1));
        return;
    }
    if (method == QStringLiteral("event.lsp.diagnostics")) {
        emit lspDiagnostics(params.value(QStringLiteral("path")).toString(),
                            params.value(QStringLiteral("diagnostics")).toArray().toVariantList());
        return;
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
    }
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

    if (method == QStringLiteral("workspace.open")) {
        m_workspaceRoot = result.value(QStringLiteral("root")).toString();
        m_workspaceName = result.value(QStringLiteral("name")).toString();
        m_workspaceKind = result.value(QStringLiteral("kind")).toString();
        emit workspaceChanged();
        listDir(m_workspaceRoot);
        return;
    }

    if (method == QStringLiteral("workspace.createProject")) {
        m_workspaceRoot = result.value(QStringLiteral("root")).toString();
        m_workspaceName = result.value(QStringLiteral("name")).toString();
        m_workspaceKind = result.value(QStringLiteral("kind")).toString();
        emit workspaceChanged();
        listDir(m_workspaceRoot);
        return;
    }

    if (method == QStringLiteral("workspace.createFolder")) {
        emit workspaceFolderCreated(result.value(QStringLiteral("path")).toString());
        return;
    }

    if (method == QStringLiteral("workspace.browse")) {
        emit workspaceBrowseListed(
            result.value(QStringLiteral("path")).toString(),
            result.value(QStringLiteral("parent")).toString(),
            result.value(QStringLiteral("entries")).toArray().toVariantList());
        return;
    }

    if (method == QStringLiteral("workspace.close")) {
        m_workspaceRoot.clear();
        m_workspaceName.clear();
        m_workspaceKind.clear();
        emit workspaceChanged();
        return;
    }

    if (method == QStringLiteral("tools.detect") || method == QStringLiteral("tools.status")) {
        emit toolsListed(result.value(QStringLiteral("tools")).toArray().toVariantList());
        return;
    }

    if (method == QStringLiteral("build.run")) {
        setBuilding(false);
        emit buildFinished(result.value(QStringLiteral("success")).toBool(),
                           result.value(QStringLiteral("exitCode")).toInt(-1),
                           result.value(QStringLiteral("diagnostics")).toInt(0));
        return;
    }

    if (method == QStringLiteral("test.run")) {
        setTesting(false);
        emit testFinished(result.value(QStringLiteral("success")).toBool(),
                          result.value(QStringLiteral("passed")).toInt(0),
                          result.value(QStringLiteral("failed")).toInt(0),
                          result.value(QStringLiteral("ignored")).toInt(0));
        return;
    }

    if (method == QStringLiteral("quality.run")) {
        setAnalyzing(false);
        emit qualityFinished(result.value(QStringLiteral("success")).toBool(),
                             result.value(QStringLiteral("exitCode")).toInt(-1),
                             result.value(QStringLiteral("diagnostics")).toInt(0));
        return;
    }

    if (method == QStringLiteral("command.list")) {
        emit commandsListed(result.value(QStringLiteral("commands")).toArray().toVariantList());
        return;
    }

    if (dispatchFileResult(method, result)) {
        return;
    }

    if (dispatchLspResult(method, result)) {
        return;
    }

    if (method == QStringLiteral("terminal.open")) {
        setTerminalActive(true);
        appendLog(QStringLiteral("terminal aberto: %1")
                      .arg(result.value(QStringLiteral("shell")).toString()));
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
    }
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
        emit lspCompletionResolved(result.value(QStringLiteral("items")).toArray().toVariantList());
        return true;
    }

    if (method == QStringLiteral("lsp.references")) {
        emit lspReferencesResolved(
            result.value(QStringLiteral("references")).toArray().toVariantList());
        return true;
    }

    if (method == QStringLiteral("lsp.semanticTokens")) {
        emit lspSemanticTokensResolved(
            result.value(QStringLiteral("tokens")).toArray().toVariantList());
        return true;
    }

    if (method == QStringLiteral("lsp.rename")) {
        QStringList files;
        const QJsonArray fileArray = result.value(QStringLiteral("files")).toArray();
        files.reserve(fileArray.size());
        for (const QJsonValue file : fileArray) {
            files.append(file.toString());
        }
        emit lspRenameApplied(files, result.value(QStringLiteral("edits")).toInt(0));
        return true;
    }

    return false;
}

void CoreClient::sendRequest(const QString& method, const QJsonObject& params)
{
    if (m_process.state() != QProcess::Running) {
        appendLog(QStringLiteral("ignorando %1: core nao esta rodando").arg(method));
        return;
    }

    const qint64 id = m_nextRequestId++;
    m_pendingMethods.insert(id, method);

    const QJsonObject request{
        {QStringLiteral("jsonrpc"), QStringLiteral("2.0")},
        {QStringLiteral("id"), id},
        {QStringLiteral("method"), method},
        {QStringLiteral("params"), params},
    };
    const QByteArray payload = QJsonDocument(request).toJson(QJsonDocument::Compact) + '\n';
    if (method != QStringLiteral("lsp.didChange")) {
        appendLog(QStringLiteral("-> %1").arg(QString::fromUtf8(payload.left(200).trimmed())));
    }
    m_process.write(payload);
}

void CoreClient::appendLog(const QString& line)
{
    m_logLines.append(QStringLiteral("[%1] %2").arg(timestamp(), line));
    while (m_logLines.size() > kMaxLogLines) {
        m_logLines.removeFirst();
    }
    emit logChanged();
}

void CoreClient::appendErrorLog(const QString& line)
{
    appendLog(line);

    const QString path = errorLogFile();
    if (!QDir().mkpath(QFileInfo(path).absolutePath())) {
        return;
    }
    QFile file(path);
    if (file.open(QIODevice::Append | QIODevice::Text)) {
        QTextStream stream(&file);
        stream << QDateTime::currentDateTime().toString(Qt::ISODate) << ' ' << line << '\n';
    }
}

void CoreClient::setStatus(const QString& status, bool connected)
{
    if (m_status == status && m_connected == connected) {
        return;
    }
    m_status = status;
    m_connected = connected;
    emit statusChanged();
}

QString CoreClient::resolveCoreBinary()
{
    QString fromEnv = qEnvironmentVariable("KINEIN_CORE_BIN");
    if (!fromEnv.isEmpty() && QFileInfo::exists(fromEnv)) {
        return fromEnv;
    }

    const QStringList candidates{
        QCoreApplication::applicationDirPath() + QStringLiteral("/kinein-core"),
        QDir::currentPath() + QStringLiteral("/target/debug/kinein-core"),
        QDir::currentPath() + QStringLiteral("/../target/debug/kinein-core"),
        QDir::currentPath() + QStringLiteral("/../../target/debug/kinein-core"),
    };
    for (const QString& candidate : candidates) {
        if (QFileInfo::exists(candidate)) {
            return QFileInfo(candidate).absoluteFilePath();
        }
    }

    return QStandardPaths::findExecutable(QStringLiteral("kinein-core"));
}

} // namespace kinein
