#include "core_client.h"

#include <QJsonArray>
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

void CoreClient::listRecentWorkspaces()
{
    sendRequest(QStringLiteral("workspace.recent.list"), QJsonObject{});
}

void CoreClient::pinRecentWorkspace(const QString& root, bool pinned)
{
    sendRequest(QStringLiteral("workspace.recent.pin"),
                QJsonObject{{QStringLiteral("root"), root}, {QStringLiteral("pinned"), pinned}});
}

void CoreClient::removeRecentWorkspace(const QString& root)
{
    sendRequest(QStringLiteral("workspace.recent.remove"),
                QJsonObject{{QStringLiteral("root"), root}});
}

void CoreClient::clearRecentWorkspaces()
{
    sendRequest(QStringLiteral("workspace.recent.clear"), QJsonObject{});
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
    // M4.3: fechar de proposito nao deve recuperar "vazio" num crash futuro.
    m_lastWorkspaceRoot.clear();
    sendRequest(QStringLiteral("workspace.close"), QJsonObject{});
}

void CoreClient::saveSession(const QStringList& openFiles, const QString& activeFile)
{
    QJsonArray files;
    for (const QString& file : openFiles) {
        files.append(file);
    }
    QJsonObject params{{QStringLiteral("openFiles"), files}};
    if (!activeFile.isEmpty()) {
        params.insert(QStringLiteral("activeFile"), activeFile);
    }
    sendRequest(QStringLiteral("workspace.saveSession"), params);
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

void CoreClient::writeFile(const QString& path, const QString& content,
                           const QString& expectedContent)
{
    sendRequest(QStringLiteral("fs.write"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("expectedContent"), expectedContent}});
}

void CoreClient::formatFile(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("format.text"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("text"), content}});
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

void CoreClient::requestCodeActions(const QString& path, const QString& content, int line,
                                    int column)
{
    sendRequest(QStringLiteral("lsp.codeActions"), QJsonObject{{QStringLiteral("path"), path},
                                                               {QStringLiteral("content"), content},
                                                               {QStringLiteral("line"), line},
                                                               {QStringLiteral("column"), column}});
}

void CoreClient::applyCodeAction(const QString& path, const QString& content, int actionIndex)
{
    sendRequest(QStringLiteral("lsp.applyCodeAction"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("actionIndex"), actionIndex}});
}

void CoreClient::applyWorkspaceEdit(const QString& transactionId)
{
    sendRequest(QStringLiteral("lsp.workspaceEdit.apply"),
                QJsonObject{{QStringLiteral("transactionId"), transactionId}});
}

void CoreClient::cancelWorkspaceEdit(const QString& transactionId)
{
    sendRequest(QStringLiteral("lsp.workspaceEdit.cancel"),
                QJsonObject{{QStringLiteral("transactionId"), transactionId}});
}

void CoreClient::requestDocumentSymbols(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.documentSymbols"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::requestWorkspaceSymbols(const QString& path, const QString& content,
                                         const QString& query)
{
    sendRequest(QStringLiteral("lsp.workspaceSymbols"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("query"), query}});
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

void CoreClient::requestSemanticTokens(const QString& path, const QString& content, int version)
{
    sendRequest(QStringLiteral("lsp.semanticTokens"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("version"), version}});
}

void CoreClient::requestSyntaxTree(const QString& path, const QString& content, int version)
{
    sendRequest(QStringLiteral("syntaxTree.update"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("version"), version}});
}

void CoreClient::requestSwitchSourceHeader(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.switchSourceHeader"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::lspRestart(const QString& language)
{
    // language vazio = reinicia todos os servidores vivos (M4.3b).
    QJsonObject params;
    if (!language.isEmpty()) {
        params.insert(QStringLiteral("language"), language);
    }
    sendRequest(QStringLiteral("lsp.restart"), params);
}

void CoreClient::draftSave(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("draft.save"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::draftClear(const QString& path)
{
    sendRequest(QStringLiteral("draft.clear"), QJsonObject{{QStringLiteral("path"), path}});
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

void CoreClient::replaceInFiles(const QString& query, const QString& replacement,
                                bool caseSensitive)
{
    sendRequest(QStringLiteral("fs.replace"),
                QJsonObject{{QStringLiteral("query"), query},
                            {QStringLiteral("replacement"), replacement},
                            {QStringLiteral("caseSensitive"), caseSensitive}});
}

void CoreClient::runConfigList()
{
    sendRequest(QStringLiteral("runConfig.list"), QJsonObject{});
}

void CoreClient::runConfigSave(const QString& id, const QString& name, const QString& command)
{
    QJsonObject params{{QStringLiteral("name"), name}, {QStringLiteral("command"), command}};
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("runConfig.save"), params);
}

void CoreClient::runConfigDelete(const QString& id)
{
    sendRequest(QStringLiteral("runConfig.delete"), QJsonObject{{QStringLiteral("id"), id}});
}

void CoreClient::runConfigSetActive(const QString& id)
{
    QJsonObject params;
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("runConfig.setActive"), params);
}

void CoreClient::debugStart(const QString& program)
{
    QJsonObject params;
    if (!program.isEmpty()) {
        params.insert(QStringLiteral("program"), program);
    }
    sendRequest(QStringLiteral("debug.start"), params);
}

void CoreClient::debugSetBreakpoints(const QString& file, const QVariantList& lines)
{
    sendRequest(QStringLiteral("debug.setBreakpoints"),
                QJsonObject{{QStringLiteral("file"), file},
                            {QStringLiteral("lines"), QJsonArray::fromVariantList(lines)}});
}

void CoreClient::debugContinue()
{
    sendRequest(QStringLiteral("debug.continue"), QJsonObject{});
}

void CoreClient::debugNext()
{
    sendRequest(QStringLiteral("debug.next"), QJsonObject{});
}

void CoreClient::debugStepIn()
{
    sendRequest(QStringLiteral("debug.stepIn"), QJsonObject{});
}

void CoreClient::debugStepOut()
{
    sendRequest(QStringLiteral("debug.stepOut"), QJsonObject{});
}

void CoreClient::debugPause()
{
    sendRequest(QStringLiteral("debug.pause"), QJsonObject{});
}

void CoreClient::debugStop()
{
    sendRequest(QStringLiteral("debug.stop"), QJsonObject{});
}

void CoreClient::gitStatus()
{
    sendRequest(QStringLiteral("git.status"), QJsonObject{});
}

void CoreClient::gitBranches()
{
    sendRequest(QStringLiteral("git.branches"), QJsonObject{});
}

void CoreClient::gitCheckout(const QString& branch)
{
    sendRequest(QStringLiteral("git.checkout"), QJsonObject{{QStringLiteral("branch"), branch}});
}

void CoreClient::gitCreateBranch(const QString& name, bool checkout)
{
    sendRequest(
        QStringLiteral("git.branchCreate"),
        QJsonObject{{QStringLiteral("name"), name}, {QStringLiteral("checkout"), checkout}});
}

void CoreClient::gitPull()
{
    sendRequest(QStringLiteral("git.pull"), QJsonObject{});
}

void CoreClient::gitPush()
{
    sendRequest(QStringLiteral("git.push"), QJsonObject{});
}

void CoreClient::gitStash(const QString& action, const QString& message)
{
    QJsonObject params{{QStringLiteral("action"), action}};
    if (!message.trimmed().isEmpty()) {
        params.insert(QStringLiteral("message"), message.trimmed());
    }
    sendRequest(QStringLiteral("git.stash"), params);
}

void CoreClient::gitFileDiff(const QString& path)
{
    sendRequest(QStringLiteral("git.fileDiff"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::gitStage(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.stage"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitUnstage(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.unstage"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitDiscard(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.discard"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitCommit(const QString& message)
{
    sendRequest(QStringLiteral("git.commit"), QJsonObject{{QStringLiteral("message"), message}});
}

void CoreClient::gitBlame(const QString& path)
{
    sendRequest(QStringLiteral("git.blame"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::gitLog()
{
    sendRequest(QStringLiteral("git.log"), QJsonObject{});
}

void CoreClient::gitCommitDiff(const QString& sha)
{
    sendRequest(QStringLiteral("git.commitDiff"), QJsonObject{{QStringLiteral("sha"), sha}});
}

void CoreClient::settingsGet()
{
    sendRequest(QStringLiteral("settings.get"), QJsonObject{});
}

void CoreClient::settingsSet(const QString& scope, const QVariantMap& values)
{
    sendRequest(QStringLiteral("settings.set"),
                QJsonObject{{QStringLiteral("scope"), scope},
                            {QStringLiteral("values"), QJsonObject::fromVariantMap(values)}});
}

void CoreClient::debugStackTrace()
{
    sendRequest(QStringLiteral("debug.stackTrace"), QJsonObject{});
}

void CoreClient::debugVariablesForFrame(double frameId)
{
    sendRequest(QStringLiteral("debug.variables"),
                QJsonObject{{QStringLiteral("frameId"), static_cast<qint64>(frameId)}});
}

void CoreClient::debugVariablesForRef(double ref)
{
    sendRequest(QStringLiteral("debug.variables"),
                QJsonObject{{QStringLiteral("ref"), static_cast<qint64>(ref)}});
}

void CoreClient::cargoCheck()
{
    sendRequest(QStringLiteral("cargo.check"), QJsonObject{});
}

void CoreClient::cargoMetadata()
{
    sendRequest(QStringLiteral("cargo.metadata"), QJsonObject{});
}

void CoreClient::cmakeConfigure()
{
    sendRequest(QStringLiteral("cmake.configure"), QJsonObject{});
}

void CoreClient::cmakeStatus()
{
    sendRequest(QStringLiteral("cmake.status"), QJsonObject{});
}

void CoreClient::runBuild(const QString& buildSystem)
{
    if (m_building || m_process.state() != QProcess::Running) {
        return;
    }
    setBuilding(true);
    QJsonObject params;
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    sendRequest(QStringLiteral("build.run"), params);
}

void CoreClient::runTests(const QString& filter, const QString& buildSystem)
{
    if (m_testing || m_process.state() != QProcess::Running) {
        return;
    }
    QJsonObject params;
    if (!filter.trimmed().isEmpty()) {
        params.insert(QStringLiteral("filter"), filter);
    }
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    setTesting(true);
    sendRequest(QStringLiteral("test.run"), params);
}

void CoreClient::runQuality(const QString& buildSystem)
{
    if (m_analyzing || m_process.state() != QProcess::Running) {
        return;
    }
    setAnalyzing(true);
    QJsonObject params;
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    sendRequest(QStringLiteral("quality.run"), params);
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

void CoreClient::runScript(const QString& path)
{
    sendRequest(QStringLiteral("run.script"), QJsonObject{{QStringLiteral("path"), path}});
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

void CoreClient::terminalInput(const QString& id, const QString& data)
{
    sendRequest(QStringLiteral("terminal.input"),
                QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("data"), data}});
}

void CoreClient::terminalResize(const QString& id, int cols, int rows)
{
    sendRequest(QStringLiteral("terminal.resize"), QJsonObject{{QStringLiteral("id"), id},
                                                               {QStringLiteral("cols"), cols},
                                                               {QStringLiteral("rows"), rows}});
}

void CoreClient::terminalScroll(const QString& id, int offset)
{
    sendRequest(QStringLiteral("terminal.scroll"),
                QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("offset"), offset}});
}

void CoreClient::terminalWheel(const QString& id, int col, int row, int lines, int modifiers)
{
    // Fronteira UI→contrato: os modificadores do Qt param aqui. O core recebe o
    // gesto tipado e decide o destino (relatorio a aplicacao, cursor keys ou
    // historico local) lendo o modo VT — ver docs/arquitetura/03-ipc-protocol.md.
    const auto keys = static_cast<Qt::KeyboardModifiers>(modifiers);
    sendRequest(
        QStringLiteral("terminal.mouse"),
        QJsonObject{
            {QStringLiteral("id"), id},
            {QStringLiteral("col"), col},
            {QStringLiteral("row"), row},
            {QStringLiteral("event"), QJsonObject{{QStringLiteral("kind"), QStringLiteral("wheel")},
                                                  {QStringLiteral("lines"), lines}}},
            {QStringLiteral("modifiers"),
             QJsonObject{{QStringLiteral("shift"), keys.testFlag(Qt::ShiftModifier)},
                         {QStringLiteral("alt"), keys.testFlag(Qt::AltModifier)},
                         {QStringLiteral("ctrl"), keys.testFlag(Qt::ControlModifier)}}}});
}

void CoreClient::terminalClose(const QString& id)
{
    sendRequest(QStringLiteral("terminal.close"), QJsonObject{{QStringLiteral("id"), id}});
}

} // namespace kinein
