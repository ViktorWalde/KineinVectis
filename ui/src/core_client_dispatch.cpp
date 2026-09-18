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
        if (method == QStringLiteral("debug.evaluate")) {
            // O core devolve a expressao nos `details`; sem ela a falha de um
            // watch marcaria todos os outros.
            emit debugEvaluateFailed(error.value(QStringLiteral("details"))
                                         .toObject()
                                         .value(QStringLiteral("expression"))
                                         .toString(),
                                     message);
        }
        emit requestFailed(method, message, error.value(QStringLiteral("code")).toString());
        return;
    }

    dispatchResult(method, response.value(QStringLiteral("result")).toObject());
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
    if (method == QStringLiteral("run.capabilities")) {
        emit runCapabilitiesListed(
            result.value(QStringLiteral("runnable")).toArray().toVariantList(),
            result.value(QStringLiteral("debuggable")).toArray().toVariantList());
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
        else if (m_workspaceKind == QStringLiteral("python")) {
            m_workspaceBuildSystems.append(QStringLiteral("python"));
        }
    }
    // M4.3: lembra o root para recuperar de um crash futuro.
    m_lastWorkspaceRoot = m_workspaceRoot;
    emit workspaceChanged();
    // Um espelho remoto (0.122.0) diz de quem e'; vazio = workspace comum.
    emit remoteMirrorChanged(result.value(QStringLiteral("remote")).toObject().toVariantMap());
    listRecentWorkspaces();
    listDir(m_workspaceRoot);
    if (m_workspaceBuildSystems.contains(QStringLiteral("cmake"))) {
        cmakeStatus();
        // Os presets do projeto, para o seletor do kit (pente-fino 2026-09-18:
        // o metodo existia desde 0.25.0 e nenhuma tela o pedia).
        cmakePresetsList();
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
    // M-S1: rascunhos não salvos recuperados de um crash (DocsPublic/seguranca/23). Só em
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
    if (method == QStringLiteral("test.discover")) {
        // Job de listagem: o resultado vem por event.test.discovered; nao e' o
        // job de testes (m_testJobId), entao nao entra em `testing`.
        appendLog(QStringLiteral("job aceito (test.discover): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
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
