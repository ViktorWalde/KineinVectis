// IPC client that owns the kernwerk-core child process.
//
// The UI never runs build tools or touches the workspace file system itself;
// it only sends JSON-RPC requests to the core and renders the responses.

#pragma once

#include <QByteArray>
#include <QHash>
#include <QJsonObject>
#include <QObject>
#include <QProcess>
#include <QString>
#include <QStringList>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

namespace kernwerk {

class CoreClient : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QString status READ status NOTIFY statusChanged)
    Q_PROPERTY(bool connected READ isConnected NOTIFY statusChanged)
    Q_PROPERTY(QString protocolVersion READ protocolVersion NOTIFY statusChanged)
    Q_PROPERTY(QStringList logLines READ logLines NOTIFY logChanged)
    Q_PROPERTY(QString workspaceRoot READ workspaceRoot NOTIFY workspaceChanged)
    Q_PROPERTY(QString workspaceName READ workspaceName NOTIFY workspaceChanged)
    Q_PROPERTY(QString workspaceKind READ workspaceKind NOTIFY workspaceChanged)
    Q_PROPERTY(QString homeDir READ homeDir CONSTANT)
    Q_PROPERTY(QString errorLogFile READ errorLogFile CONSTANT)
    Q_PROPERTY(bool building READ isBuilding NOTIFY buildingChanged)
    Q_PROPERTY(bool testing READ isTesting NOTIFY testingChanged)
    Q_PROPERTY(bool analyzing READ isAnalyzing NOTIFY analyzingChanged)
    Q_PROPERTY(bool running READ isRunning NOTIFY runningChanged)
    Q_PROPERTY(bool terminalActive READ isTerminalActive NOTIFY terminalActiveChanged)

public:
    explicit CoreClient(QObject* parent = nullptr);
    ~CoreClient() override;

    [[nodiscard]] QString status() const;
    [[nodiscard]] bool isConnected() const;
    [[nodiscard]] QString protocolVersion() const;
    [[nodiscard]] QStringList logLines() const;
    [[nodiscard]] QString workspaceRoot() const;
    [[nodiscard]] QString workspaceName() const;
    [[nodiscard]] QString workspaceKind() const;
    [[nodiscard]] static QString homeDir();
    [[nodiscard]] static QString errorLogFile();
    [[nodiscard]] bool isBuilding() const;
    [[nodiscard]] bool isTesting() const;
    [[nodiscard]] bool isAnalyzing() const;
    [[nodiscard]] bool isRunning() const;
    [[nodiscard]] bool isTerminalActive() const;

    Q_INVOKABLE void start();
    Q_INVOKABLE void ping();
    Q_INVOKABLE void openWorkspace(const QString& path);
    Q_INVOKABLE void browseWorkspaceFolders(const QString& path);
    Q_INVOKABLE void createWorkspaceFolder(const QString& parent, const QString& name);
    Q_INVOKABLE void createWorkspaceProject(const QString& parent, const QString& name,
                                            const QString& templateId);
    Q_INVOKABLE void closeWorkspace();
    Q_INVOKABLE void listDir(const QString& path);
    Q_INVOKABLE void readFile(const QString& path);
    Q_INVOKABLE void createFile(const QString& path, const QString& content = QString());
    Q_INVOKABLE void createDirectory(const QString& path);
    Q_INVOKABLE void writeFile(const QString& path, const QString& content);
    Q_INVOKABLE void renamePath(const QString& from, const QString& to);
    Q_INVOKABLE void deletePath(const QString& path);
    Q_INVOKABLE void listCommands();
    Q_INVOKABLE void detectTools();
    Q_INVOKABLE void runBuild();
    Q_INVOKABLE void runTests(const QString& filter = QString());
    Q_INVOKABLE void runQuality();
    Q_INVOKABLE void notifyFileChanged(const QString& path, const QString& content);
    Q_INVOKABLE void requestDefinition(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestHover(const QString& path, const QString& content, int line,
                                  int column);
    Q_INVOKABLE void requestCompletion(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestReferences(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestRename(const QString& path, const QString& content, int line,
                                   int column, const QString& newName);
    Q_INVOKABLE void requestSemanticTokens(const QString& path, const QString& content);
    Q_INVOKABLE void findFiles(const QString& query);
    Q_INVOKABLE void searchInFiles(const QString& query, bool caseSensitive);
    Q_INVOKABLE void runStart(const QString& command);
    Q_INVOKABLE void runStdin(const QString& data);
    Q_INVOKABLE void runStop();
    Q_INVOKABLE void terminalOpen();
    Q_INVOKABLE void terminalInput(const QString& data);
    Q_INVOKABLE void terminalClose();

signals:
    void statusChanged();
    void logChanged();
    void workspaceChanged();
    void workspaceBrowseListed(const QString& path, const QString& parent,
                               const QVariantList& entries);
    void workspaceFolderCreated(const QString& path);
    void dirListed(const QString& path, const QVariantList& entries);
    void fileLoaded(const QString& path, const QString& content);
    void fileCreated(const QString& path);
    void directoryCreated(const QString& path);
    void fileSaved(const QString& path);
    void pathRenamed(const QString& from, const QString& to);
    void pathDeleted(const QString& path);
    void commandsListed(const QVariantList& commands);
    void toolsListed(const QVariantList& tools);
    void buildingChanged();
    void buildStarted(const QString& command);
    void buildOutput(const QString& line);
    void buildDiagnostic(const QVariantMap& diagnostic);
    void buildFinished(bool success, int exitCode, int diagnostics);
    void testingChanged();
    void testStarted(const QString& command);
    void testOutput(const QString& line, const QString& stream);
    void testCase(const QString& name, const QString& status);
    void testFinished(bool success, int passed, int failed, int ignored);
    void analyzingChanged();
    void qualityStarted(const QString& command);
    void qualityDiagnostic(const QVariantMap& diagnostic);
    void qualityFinished(bool success, int exitCode, int diagnostics);
    void lspDiagnostics(const QString& path, const QVariantList& diagnostics);
    void lspDefinitionResolved(const QString& path, int line, int column);
    void lspHoverResolved(const QString& content);
    void lspCompletionResolved(const QVariantList& items);
    void lspReferencesResolved(const QVariantList& references);
    void lspRenameApplied(const QStringList& files, int edits);
    void lspSemanticTokensResolved(const QVariantList& tokens);
    void fileSearchResults(const QVariantList& matches, bool truncated);
    void searchResults(const QVariantList& matches, bool truncated);
    void runningChanged();
    void runStarted(const QString& command);
    void runOutput(const QString& line, const QString& stream);
    void runFinished(bool success, int exitCode);
    void terminalActiveChanged();
    void terminalData(const QString& data);
    void terminalClosed(int exitCode);
    void requestFailed(const QString& method, const QString& message);

private:
    void handleStarted();
    void handleStdout();
    void handleStderr();
    void handleFinished(int exitCode, QProcess::ExitStatus exitStatus);
    void handleErrorOccurred(QProcess::ProcessError error);
    void handleResponseLine(const QByteArray& line);
    void handleNotification(const QString& method, const QJsonObject& params);
    void dispatchResult(const QString& method, const QJsonObject& result);
    bool dispatchFileResult(const QString& method, const QJsonObject& result);
    bool dispatchLspResult(const QString& method, const QJsonObject& result);
    void setBuilding(bool building);
    void setTesting(bool testing);
    void setAnalyzing(bool analyzing);
    void setRunning(bool running);
    void setTerminalActive(bool active);
    void sendRequest(const QString& method, const QJsonObject& params);
    void appendLog(const QString& line);
    void appendErrorLog(const QString& line);
    void setStatus(const QString& status, bool connected);
    [[nodiscard]] static QString resolveCoreBinary();

    QProcess m_process;
    QByteArray m_stdoutBuffer;
    QStringList m_logLines;
    QString m_status;
    QString m_protocolVersion;
    QString m_workspaceRoot;
    QString m_workspaceName;
    QString m_workspaceKind;
    QHash<qint64, QString> m_pendingMethods;
    bool m_connected = false;
    bool m_building = false;
    bool m_testing = false;
    bool m_analyzing = false;
    bool m_running = false;
    bool m_terminalActive = false;
    qint64 m_nextRequestId = 1;
};

} // namespace kernwerk
