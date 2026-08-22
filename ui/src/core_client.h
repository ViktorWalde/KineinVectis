// IPC client that owns the kinein-core child process.
//
// The UI never runs build tools or touches the workspace file system itself;
// it only sends JSON-RPC requests to the core and renders the responses.

#pragma once

#include <QByteArray>
#include <QElapsedTimer>
#include <QHash>
#include <QJsonObject>
#include <QObject>
#include <QProcess>
#include <QSet>
#include <QString>
#include <QStringList>
#include <QVariantList>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

namespace kinein {

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
    Q_PROPERTY(QStringList workspaceBuildSystems READ workspaceBuildSystems NOTIFY workspaceChanged)
    Q_PROPERTY(QString homeDir READ homeDir CONSTANT)
    Q_PROPERTY(QString errorLogFile READ errorLogFile CONSTANT)
    Q_PROPERTY(bool building READ isBuilding NOTIFY buildingChanged)
    Q_PROPERTY(bool testing READ isTesting NOTIFY testingChanged)
    Q_PROPERTY(bool analyzing READ isAnalyzing NOTIFY analyzingChanged)
    Q_PROPERTY(bool running READ isRunning NOTIFY runningChanged)
    Q_PROPERTY(bool debugging READ isDebugging NOTIFY debuggingChanged)
    Q_PROPERTY(bool terminalActive READ isTerminalActive NOTIFY terminalActiveChanged)
    Q_PROPERTY(
        bool scanningEnvironment READ isScanningEnvironment NOTIFY scanningEnvironmentChanged)
    Q_PROPERTY(bool recovering READ isRecovering NOTIFY recoveringChanged)

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
    [[nodiscard]] QStringList workspaceBuildSystems() const;
    [[nodiscard]] static QString homeDir();
    [[nodiscard]] static QString errorLogFile();
    [[nodiscard]] bool isBuilding() const;
    [[nodiscard]] bool isTesting() const;
    [[nodiscard]] bool isAnalyzing() const;
    [[nodiscard]] bool isRunning() const;
    [[nodiscard]] bool isDebugging() const;
    [[nodiscard]] bool isTerminalActive() const;
    [[nodiscard]] bool isScanningEnvironment() const;
    [[nodiscard]] bool isRecovering() const;

    Q_INVOKABLE void start();
    Q_INVOKABLE void ping();
    Q_INVOKABLE void openWorkspace(const QString& path);
    Q_INVOKABLE void listRecentWorkspaces();
    Q_INVOKABLE void pinRecentWorkspace(const QString& root, bool pinned);
    Q_INVOKABLE void removeRecentWorkspace(const QString& root);
    Q_INVOKABLE void clearRecentWorkspaces();
    Q_INVOKABLE void browseWorkspaceFolders(const QString& path);
    Q_INVOKABLE void createWorkspaceFolder(const QString& parent, const QString& name);
    Q_INVOKABLE void createWorkspaceProject(const QString& parent, const QString& name,
                                            const QString& templateId);
    Q_INVOKABLE void closeWorkspace();
    Q_INVOKABLE void saveSession(const QStringList& openFiles, const QString& activeFile);
    Q_INVOKABLE void listDir(const QString& path);
    Q_INVOKABLE void readFile(const QString& path);
    Q_INVOKABLE void createFile(const QString& path, const QString& content = QString());
    Q_INVOKABLE void createDirectory(const QString& path);
    Q_INVOKABLE void writeFile(const QString& path, const QString& content,
                               const QString& expectedContent);
    Q_INVOKABLE void formatFile(const QString& path, const QString& content);
    /// Pede o catalogo de formatters (`format.capabilities`, 0.61.0).
    ///
    /// A UI NAO decide o que e formatavel: ate 0.60 ela mantinha duas listas
    /// escritas a mao que divergiam entre si e do core. O catalogo e estatico,
    /// entao basta pedir uma vez ao conectar.
    Q_INVOKABLE void formatCapabilities();
    Q_INVOKABLE void renamePath(const QString& from, const QString& to);
    Q_INVOKABLE void deletePath(const QString& path);
    Q_INVOKABLE void listCommands();
    Q_INVOKABLE void detectTools();
    Q_INVOKABLE void scanEnvironment();
    Q_INVOKABLE void runConfigList();
    Q_INVOKABLE void runConfigSave(const QString& id, const QString& name, const QString& command);
    Q_INVOKABLE void runConfigDelete(const QString& id);
    Q_INVOKABLE void runConfigSetActive(const QString& id);
    Q_INVOKABLE void debugStart(const QString& program = QString());
    Q_INVOKABLE void debugSetBreakpoints(const QString& file, const QVariantList& lines);
    Q_INVOKABLE void debugContinue();
    Q_INVOKABLE void debugNext();
    Q_INVOKABLE void debugStepIn();
    Q_INVOKABLE void debugStepOut();
    Q_INVOKABLE void debugPause();
    Q_INVOKABLE void debugStop();
    Q_INVOKABLE void gitStatus();
    Q_INVOKABLE void gitBranches();
    Q_INVOKABLE void gitCheckout(const QString& branch);
    Q_INVOKABLE void gitCreateBranch(const QString& name, bool checkout = true);
    Q_INVOKABLE void gitPull();
    Q_INVOKABLE void gitPush();
    Q_INVOKABLE void gitStash(const QString& action, const QString& message = QString());
    Q_INVOKABLE void gitFileDiff(const QString& path);
    Q_INVOKABLE void gitStage(const QStringList& paths);
    Q_INVOKABLE void gitUnstage(const QStringList& paths);
    Q_INVOKABLE void gitDiscard(const QStringList& paths);
    Q_INVOKABLE void gitCommit(const QString& message);
    Q_INVOKABLE void gitBlame(const QString& path);
    Q_INVOKABLE void gitLog();
    Q_INVOKABLE void gitCommitDiff(const QString& sha);
    Q_INVOKABLE void settingsGet();
    Q_INVOKABLE void settingsSet(const QString& scope, const QVariantMap& values);
    Q_INVOKABLE void debugStackTrace();
    Q_INVOKABLE void debugVariablesForFrame(double frameId);
    Q_INVOKABLE void debugVariablesForRef(double ref);
    Q_INVOKABLE void cargoCheck();
    Q_INVOKABLE void cargoMetadata();
    Q_INVOKABLE void cmakeConfigure();
    Q_INVOKABLE void cmakeStatus();
    Q_INVOKABLE void runBuild(const QString& buildSystem = QString());
    Q_INVOKABLE void runTests(const QString& filter = QString(),
                              const QString& buildSystem = QString());
    Q_INVOKABLE void runQuality(const QString& buildSystem = QString());
    Q_INVOKABLE void runCoverage(const QString& buildSystem = QString());
    Q_INVOKABLE void runAudit();
    Q_INVOKABLE void runMemcheck();
    Q_INVOKABLE void requestFileContext(const QString& path);
    Q_INVOKABLE void cancelBuild();
    Q_INVOKABLE void cancelTests();
    Q_INVOKABLE void cancelQuality();
    Q_INVOKABLE void cancelEnvironmentScan();
    Q_INVOKABLE void notifyFileChanged(const QString& path, const QString& content);
    Q_INVOKABLE void requestDefinition(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestHover(const QString& path, const QString& content, int line,
                                  int column);
    Q_INVOKABLE void requestCompletion(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestReferences(const QString& path, const QString& content, int line,
                                       int column);
    Q_INVOKABLE void requestCodeActions(const QString& path, const QString& content, int line,
                                        int column);
    Q_INVOKABLE void requestDocumentSymbols(const QString& path, const QString& content);
    Q_INVOKABLE void requestWorkspaceSymbols(const QString& path, const QString& content,
                                             const QString& query);
    Q_INVOKABLE void applyCodeAction(const QString& path, const QString& content, int actionIndex);
    Q_INVOKABLE void applyWorkspaceEdit(const QString& transactionId);
    Q_INVOKABLE void cancelWorkspaceEdit(const QString& transactionId);
    Q_INVOKABLE void requestRename(const QString& path, const QString& content, int line,
                                   int column, const QString& newName);
    Q_INVOKABLE void requestSemanticTokens(const QString& path, const QString& content,
                                           int version);
    Q_INVOKABLE void requestSyntaxTree(const QString& path, const QString& content, int version);
    Q_INVOKABLE void requestSwitchSourceHeader(const QString& path, const QString& content);
    // M4.3b: reinicia servidor(es) LSP; language vazio = todos.
    Q_INVOKABLE void lspRestart(const QString& language);
    // M-S1 (docs/seguranca/23): autosave/limpeza de rascunho não salvo (rede de segurança).
    Q_INVOKABLE void draftSave(const QString& path, const QString& content);
    Q_INVOKABLE void draftClear(const QString& path);
    Q_INVOKABLE void findFiles(const QString& query);
    Q_INVOKABLE void searchInFiles(const QString& query, bool caseSensitive);
    Q_INVOKABLE void replaceInFiles(const QString& query, const QString& replacement,
                                    bool caseSensitive);
    Q_INVOKABLE void runStart(const QString& command);
    Q_INVOKABLE void runScript(const QString& path);
    Q_INVOKABLE void runStdin(const QString& data);
    Q_INVOKABLE void runStop();
    // D2.3 (docs/roadmaps/24): multi-terminal — todo comando leva o id da sessão.
    Q_INVOKABLE void terminalOpen();
    Q_INVOKABLE void terminalInput(const QString& id, const QString& data);
    Q_INVOKABLE void terminalResize(const QString& id, int cols, int rows);
    Q_INVOKABLE void terminalScroll(const QString& id, int offset);
    /// Reporta um gesto de roda ao core (`terminal.mouse`, protocolo 0.60.0).
    ///
    /// `col`/`row` sao a celula sob o ponteiro (0-based, como no render);
    /// `lines` e positivo para cima. `modifiers` sao `Qt::KeyboardModifiers`
    /// cruos e sao traduzidos aqui — a UI nao decide o destino do gesto, o core
    /// decide lendo o modo VT.
    Q_INVOKABLE void terminalWheel(const QString& id, int col, int row, int lines, int modifiers);
    Q_INVOKABLE void terminalClose(const QString& id);

signals:
    void statusChanged();
    void logChanged();
    void workspaceChanged();
    void recentWorkspacesResolved(const QVariantList& workspaces);
    void workspaceBrowseListed(const QString& path, const QString& parent,
                               const QVariantList& entries);
    void workspaceFolderCreated(const QString& path);
    void sessionRestored(const QStringList& files, const QString& activeFile);
    // M-S1 (docs/seguranca/23): rascunhos não salvos recuperados de um crash.
    void draftsRecovered(const QVariantList& drafts);
    void dirListed(const QString& path, const QVariantList& entries);
    void fileLoaded(const QString& path, const QString& content);
    void fileCreated(const QString& path);
    void directoryCreated(const QString& path);
    void fileSaved(const QString& path);
    void fileSaveFailed(const QString& path, const QString& message);
    void filesChanged(const QVariantList& changes);
    void fileWatchFailed(const QString& message);
    void fileFormatted(const QString& path, const QString& text, bool changed);
    void pathRenamed(const QString& from, const QString& to);
    void pathDeleted(const QString& path);
    void commandsListed(const QVariantList& commands);
    void toolsListed(const QVariantList& tools);
    void formatCapabilitiesListed(const QVariantList& formatters);
    void cmakeStatusResolved(bool configured, bool hasCompileCommands);
    void cmakeConfigureFinished(bool success);
    void cargoMetadataResolved(int packages);
    void runConfigsResolved(const QVariantList& configs, const QString& activeId);
    void scanningEnvironmentChanged();
    void recoveringChanged();
    // Emitido quando a recuperacao de crash reconecta: a UI re-sincroniza
    // o arquivo ativo com o LSP novo (highlighting/diagnostico voltam).
    void recovered();
    void environmentScanStarted(int tools);
    void environmentTool(const QVariantMap& tool);
    void environmentScanFinished(bool success, int total, int detected, int missing, int failed,
                                 const QVariantList& tools);
    void jobCreated(const QVariantMap& job);
    void jobProgress(const QString& jobId, const QString& status, double progress,
                     const QString& message);
    void jobOutput(const QString& jobId, const QString& line);
    void jobFinished(const QString& jobId, const QString& status);
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
    void coverageFinished(bool success, double percent, double linesCovered, double linesTotal,
                          const QVariantList& files, const QString& error);
    void auditStarted(const QString& command);
    void auditDiagnostic(const QVariantMap& diagnostic);
    void auditFinished(bool success, int vulnerabilities, int policyFindings,
                       const QVariantMap& database, const QString& error);
    void memcheckStarted(const QString& command);
    void memcheckDiagnostic(const QVariantMap& diagnostic);
    void memcheckFinished(bool success, int tests, int findings, const QString& error);
    void fileContextResolved(const QVariantMap& context);
    void lspDiagnostics(const QString& path, const QVariantList& diagnostics);
    void lspDefinitionResolved(const QString& path, int line, int column);
    void lspHoverResolved(const QString& content);
    void lspCompletionResolved(const QVariantList& items, bool isIncomplete);
    void lspReferencesResolved(const QVariantList& references);
    void lspCodeActionsResolved(const QVariantList& actions);
    void lspWorkspaceEditPreviewResolved(const QString& transactionId, const QString& title,
                                         const QVariantList& files, int edits);
    void lspWorkspaceEditApplied(const QStringList& files, const QString& title, int edits);
    void lspWorkspaceEditCancelled(const QString& transactionId);
    void lspSymbolsResolved(const QVariantList& symbols);
    void lspSemanticTokensResolved(const QString& path, int version, const QVariantList& tokens);
    void syntaxTreeResolved(const QString& path, int version, const QString& language,
                            bool hasErrors, const QVariantList& highlights,
                            const QVariantList& foldingRanges, const QVariantList& outline,
                            const QVariantList& locals);
    void lspSwitchSourceHeaderResolved(const QString& path);
    // M4.3b: um servidor LSP reiniciou — a UI re-sincroniza o arquivo ativo.
    void lspRestarted(const QString& language);
    void fileSearchResults(const QVariantList& matches, bool truncated);
    void searchResults(const QVariantList& matches, bool truncated);
    void filesReplaced(const QStringList& files, int replacements);
    void runningChanged();
    void runStarted(const QString& command);
    void runOutput(const QString& line, const QString& stream);
    void runFinished(bool success, int exitCode);
    void debuggingChanged();
    void debugStarted(const QString& program);
    void debugOutput(const QString& category, const QString& line);
    void debugStopped(const QString& reason, const QString& file, int line, int threadId);
    void debugContinued();
    void debugFinished(int exitCode);
    void debugStackTraceResolved(const QVariantList& frames);
    void debugVariablesResolved(double frameId, double ref, const QVariantList& variables);
    void gitStatusResolved(bool repo, const QString& branch, bool detached, const QString& shortSha,
                           int ahead, int behind, const QVariantList& entries);
    void gitBranchesResolved(bool repo, const QVariantList& branches);
    void gitRemoteOperationFinished(const QString& operation, bool success, const QString& message);
    void gitFileDiffResolved(const QString& path, bool repo, bool tracked,
                             const QVariantList& hunks, const QString& text);
    void gitBlameResolved(const QString& path, bool repo, bool tracked, const QVariantList& groups);
    void gitLogResolved(bool repo, const QVariantList& entries);
    void gitCommitDiffResolved(const QString& sha, const QString& text);
    void settingsResolved(const QVariantMap& effective, const QVariantMap& global,
                          const QVariantMap& workspace);
    void terminalActiveChanged();
    // D2.3: a sessão nasceu; o id identifica a aba dela daqui pra frente.
    void terminalOpened(const QString& id, const QString& shell);
    // D2 (docs/roadmaps/24): grid renderizável do terminal (cols/rows/cursor/lines).
    // Desde a D2.3 o mapa carrega `id`: a UI roteia pro terminal certo.
    void terminalRender(const QVariantMap& render);
    void terminalClosed(const QString& id, int exitCode);
    void requestFailed(const QString& method, const QString& message);

private:
    void handleStarted();
    void handleStdout();
    void handleStderr();
    void handleFinished(int exitCode, QProcess::ExitStatus exitStatus);
    void handleErrorOccurred(QProcess::ProcessError error);
    void handleResponseLine(const QByteArray& line);
    void handleNotification(const QString& method, const QJsonObject& params);
    bool handleFileSystemNotification(const QString& method, const QJsonObject& params);
    bool handleTerminalNotification(const QString& method, const QJsonObject& params);
    bool handleLspNotification(const QString& method, const QJsonObject& params);
    bool handleEnvironmentNotification(const QString& method, const QJsonObject& params);
    bool handleJobNotification(const QString& method, const QJsonObject& params);
    bool handleCmakeNotification(const QString& method, const QJsonObject& params);
    bool handleQualityNotification(const QString& method, const QJsonObject& params);
    bool handleDebugNotification(const QString& method, const QJsonObject& params);
    void dispatchResult(const QString& method, const QJsonObject& result);
    bool dispatchFileResult(const QString& method, const QJsonObject& result);
    bool dispatchLspResult(const QString& method, const QJsonObject& result);
    bool dispatchSyntaxResult(const QString& method, const QJsonObject& result);
    bool dispatchCmakeResult(const QString& method, const QJsonObject& result);
    bool dispatchDebugResult(const QString& method, const QJsonObject& result);
    bool dispatchWorkspaceResult(const QString& method, const QJsonObject& result);
    void handleWorkspaceOpened(const QJsonObject& result);
    void storeJobId(const QString& method, const QString& jobId);
    void cancelJob(const QString& jobId);
    void setBuilding(bool building);
    void setTesting(bool testing);
    void setAnalyzing(bool analyzing);
    void setRunning(bool running);
    void setDebugging(bool debugging);
    void setTerminalActive(bool active);
    void setScanningEnvironment(bool scanning);
    void setRecovering(bool recovering);
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
    QStringList m_workspaceBuildSystems;
    // M4.3: recuperacao de crash do core. m_lastWorkspaceRoot sobrevive ao
    // crash (o que a recuperacao reabre); m_recovering suprime o session
    // restore e sinaliza a UI; a janela+contador cortam loop de fork.
    QString m_lastWorkspaceRoot;
    bool m_recovering = false;
    QElapsedTimer m_recoveryWindow;
    int m_recoveryAttempts = 0;
    QHash<qint64, QString> m_pendingMethods;
    QHash<qint64, QString> m_pendingPaths;
    bool m_connected = false;
    bool m_building = false;
    bool m_testing = false;
    bool m_analyzing = false;
    bool m_running = false;
    bool m_debugging = false;
    bool m_terminalActive = false;
    // D2.3: terminalActive vira "existe ALGUMA sessão viva" — com várias abas
    // um `closed` não pode mais zerar o estado global.
    QSet<QString> m_terminalIds;
    bool m_scanningEnvironment = false;
    QString m_buildJobId;
    QString m_testJobId;
    QString m_qualityJobId;
    QString m_environmentJobId;
    qint64 m_nextRequestId = 1;
};

} // namespace kinein
