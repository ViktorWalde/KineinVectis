#include "core_client.h"

#include <QJsonObject>

namespace kinein {

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

void CoreClient::scanEnvironment()
{
    if (m_scanningEnvironment || m_process.state() != QProcess::Running) {
        return;
    }
    setScanningEnvironment(true);
    sendRequest(QStringLiteral("environment.scan"), QJsonObject{});
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

void CoreClient::cancelBuild()
{
    cancelJob(m_buildJobId);
}

void CoreClient::cancelTests()
{
    cancelJob(m_testJobId);
}

void CoreClient::cancelQuality()
{
    cancelJob(m_qualityJobId);
}

void CoreClient::cancelEnvironmentScan()
{
    cancelJob(m_environmentJobId);
}

void CoreClient::cancelJob(const QString& jobId)
{
    if (jobId.isEmpty()) {
        return;
    }
    sendRequest(QStringLiteral("job.cancel"), QJsonObject{{QStringLiteral("jobId"), jobId}});
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

} // namespace kinein
