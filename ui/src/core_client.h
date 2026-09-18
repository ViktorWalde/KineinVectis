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
    // KINEIN_STARTUP_COMMANDS (Etapa 2, medicao): ids da paleta, separados por virgula,
    // executados depois de o workspace abrir. Vazio sem a env.
    Q_PROPERTY(QStringList startupCommands READ startupCommands CONSTANT)
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
    [[nodiscard]] static QStringList startupCommands();
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
    /// Pede o catalogo do que "Executar"/"Depurar" aceitam (`run.capabilities`).
    Q_INVOKABLE void runCapabilities();
    Q_INVOKABLE void renamePath(const QString& from, const QString& to);
    Q_INVOKABLE void deletePath(const QString& path);
    Q_INVOKABLE void listCommands();
    Q_INVOKABLE void detectTools();
    Q_INVOKABLE void scanEnvironment();
    Q_INVOKABLE void runConfigList();
    Q_INVOKABLE void runConfigSave(const QString& id, const QString& name, const QString& command);
    Q_INVOKABLE void runConfigDelete(const QString& id);
    Q_INVOKABLE void runConfigSetActive(const QString& id);
    // "Gravar" como configuracao de execucao (runConfig.flashProposal, 0.113.0): a linha do
    // motor, sem rodar; flashSizeBytes <= 0 = sem a checagem; firmware (0.116.0) = id do
    // catalogo baixado, no lugar dos artefatos do build.
    Q_INVOKABLE void runConfigFlashProposal(const QString& device, const QString& engine,
                                            double flashSizeBytes = 0,
                                            const QString& firmware = QString());
    Q_INVOKABLE void debugStart(const QString& program = QString(),
                                const QVariantMap& connect = QVariantMap());
    // `breakpoints` e uma lista de mapas { line, condition?, hitCondition? }.
    // Trocou a lista crua de linhas no protocolo 0.66.0: a condicao viaja POR
    // breakpoint, e array paralelo de condicoes seria a forma de eles saírem
    // de sincronia em silencio.
    Q_INVOKABLE void debugSetBreakpoints(const QString& file, const QVariantList& breakpoints);
    Q_INVOKABLE void debugEvaluate(const QString& expression, double frameId);
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
    // O que o depurador de embarcado MOSTRA (P3, 0.118.0): os escopos de um frame
    // (Registers, os perifericos do SVD), a memoria e o disassembly — o DAP padrao.
    Q_INVOKABLE void debugScopes(double frameId);
    Q_INVOKABLE void debugReadMemory(const QString& memoryReference, double count,
                                     double offset = 0);
    Q_INVOKABLE void debugDisassemble(const QString& memoryReference, double instructionCount,
                                      double instructionOffset = 0);
    Q_INVOKABLE void cargoCheck();
    Q_INVOKABLE void cargoMetadata();
    // O configure leva o preset do KIT ativo (o ultimo `toolchain.get` pedido pela
    // tela); vazio = o core escolhe o padrao do projeto (P0, 0.115.0).
    Q_INVOKABLE void cmakeConfigure();
    Q_INVOKABLE void cmakeTargetsList();
    Q_INVOKABLE void cmakeStatus();
    // `preset` vazio = o kit padrao do workspace (etapa 14: a escolha passou a
    // ser por KIT, nao por workspace).
    Q_INVOKABLE void setupList();
    Q_INVOKABLE void dataSourceList();
    Q_INVOKABLE void dataSourceSave(const QVariantMap& profile);
    Q_INVOKABLE void dataSourceRemove(const QString& name);
    Q_INVOKABLE void dataSourceTest(const QString& name, const QString& password);
    Q_INVOKABLE void dataSourceIntrospect(const QString& name, const QString& password);
    /// O que responde nesta maquina (0.124.0): servidores no loopback,
    /// containers de banco, arquivos SQLite do projeto.
    Q_INVOKABLE void dataSourceDiscover();
    /// Cria um SQLite vazio no projeto (`path` vazio = data/<name>.sqlite).
    Q_INVOKABLE void dataSourceCreateSqlite(const QString& name, const QString& path);
    /// Sobe um PostgreSQL/MongoDB em container no loopback (job; baixa imagem).
    Q_INVOKABLE void dataSourceCreateServer(const QString& engine, const QString& name, int port);
    // Executar o que o autor escreveu (datasource.query, 0.121.0): job; `confirmWrite`
    // e' o reconhecimento de que a instrucao escreve (o core recusa sem ele).
    Q_INVOKABLE void dataSourceQuery(const QString& name, const QString& password,
                                     const QString& sql, int maxRows, bool confirmWrite);

    // Observabilidade: o Grafana que observa este projeto. A licenca dele
    // (AGPL-3.0) decide a FORMA — a IDE CONVERSA, nunca embute.
    Q_INVOKABLE void grafanaGet();
    Q_INVOKABLE void grafanaSave(const QVariantMap& profile);
    Q_INVOKABLE void grafanaForget();
    Q_INVOKABLE void grafanaProbe(const QString& token);

    Q_INVOKABLE void libraryList();
    Q_INVOKABLE void libraryPlan(const QString& id, const QString& target);
    Q_INVOKABLE void toolchainGet(const QString& preset);
    Q_INVOKABLE void toolchainInstallable();
    Q_INVOKABLE void toolchainInspectSysroot(const QString& path);
    Q_INVOKABLE void toolchainImportKit(const QString& path);
    Q_INVOKABLE void toolchainInstall(const QString& id);
    Q_INVOKABLE void toolchainSet(const QString& role, const QString& id, const QString& preset);
    Q_INVOKABLE void toolchainSetKit(const QString& preset, const QString& sysroot,
                                     const QString& targetTriple, const QString& chip,
                                     const QString& toolchainFile = QString(),
                                     const QString& svdFile = QString());
    // Embarcados (roadmaps/35 §5.7): a sonda que esta' no USB agora.
    Q_INVOKABLE void probeList();
    // Tamanho do ELF do kit (build.size): flash/RAM usados.
    Q_INVOKABLE void buildSize(const QString& program);
    // Portas seriais USB (serial.list): o canal que toda placa compartilha.
    Q_INVOKABLE void serialList();
    // Monitor serial (serial.monitor): tio/picocom/minicom/espflash numa aba de terminal.
    Q_INVOKABLE void serialMonitor(const QString& device, int baud = 0);
    // Identidade Espressif pelo canal (serial.identify): esptool flash-id como job (0.112.0).
    Q_INVOKABLE void serialIdentify(const QString& device);
    // Permissao por canal (serial.access, 0.114.0): o que falta e o passo oficial; nao roda nada.
    Q_INVOKABLE void serialAccess(const QString& device = QString());
    // Arquivos na placa MicroPython (serial.files, 0.116.0): mpremote fs como job.
    Q_INVOKABLE void serialFiles(const QString& device, const QString& action,
                                 const QString& path = QString(), const QString& local = QString());
    // O modelo do projeto embarcado (project.model): framework, SDKs, artefatos, alvo.
    Q_INVOKABLE void projectModel();
    // O indice do projeto inteiro (index.*): totais e busca por nome, sem LSP.
    Q_INVOKABLE void indexStatus();
    Q_INVOKABLE void indexSymbols(const QString& query, int limit = 0);
    // Como UM arquivo e' compilado/executado (CDB, cargo, interpretador).
    Q_INVOKABLE void indexContext(const QString& path);
    // O ambiente Python do projeto (python.*): o que vale, e criar o .venv.
    Q_INVOKABLE void pythonStatus();
    Q_INVOKABLE void pythonCreateEnvironment(const QString& tool = QString());
    // Os stubs da placa MicroPython em typings/ (python.stubs, 0.116.0): job; vazio = o
    // modelo do projeto decide o pacote.
    Q_INVOKABLE void pythonStubs(const QString& port = QString(), const QString& board = QString());
    // Cobertura dos testes (coverage.*, 0.119.0): o job que escreve o LCOV e as linhas de
    // um arquivo do ultimo relatorio, para a calha do editor.
    Q_INVOKABLE void coverageRun();
    // Os configure presets do projeto (cmake.presets.list): o seletor de preset do kit.
    Q_INVOKABLE void cmakePresetsList();
    Q_INVOKABLE void coverageLines(const QString& file);
    // O alvo Linux por SSH (remote.*, 0.120.0): catalogo sem segredo, sonda e deploy como
    // jobs, e a linha `ssh …` para configuracao de execucao/kit. Nunca senha.
    Q_INVOKABLE void remoteList();
    Q_INVOKABLE void remoteSave(const QVariantMap& target);
    Q_INVOKABLE void remoteRemove(const QString& name);
    Q_INVOKABLE void remoteProbe(const QString& name);
    Q_INVOKABLE void remoteDeploy(const QString& name, const QString& source = QString(),
                                  const QString& dest = QString());
    Q_INVOKABLE void remoteCommand(const QString& name, const QString& kind,
                                   const QString& program = QString(), int port = 0);
    Q_INVOKABLE void toolchainSetKitRemote(const QString& remoteTarget, const QString& debugServer);
    // O workspace espelhado (remote.open/sync/status, 0.122.0): a pasta do alvo vira
    // espelho local por rsync; a UI abre o espelho com o openWorkspace de sempre.
    Q_INVOKABLE void remoteOpen(const QString& name, const QString& path);
    Q_INVOKABLE void remoteSync(const QString& direction, const QStringList& paths = {});
    Q_INVOKABLE void remoteStatus();
    // Containers (roadmaps/28 §0, dominio NATIVO): Docker ou Podman, o que responder.
    Q_INVOKABLE void containerStatus();
    Q_INVOKABLE void containerList(bool all = true);
    Q_INVOKABLE void containerImages();
    Q_INVOKABLE void containerAction(const QString& id, const QString& action);
    Q_INVOKABLE void containerOpen(const QString& id, const QString& mode);
    Q_INVOKABLE void containerCompose(const QString& action, const QString& file);
    Q_INVOKABLE void configActionList(bool includeHiddenByScope = false);
    Q_INVOKABLE void configActionPreview(const QString& id, const QVariantMap& params);
    Q_INVOKABLE void configActionApply(const QString& id, const QVariantMap& params,
                                       const QVariantList& expected);
    Q_INVOKABLE void runBuild(const QString& buildSystem = QString());
    Q_INVOKABLE void runTests(const QString& filter = QString(),
                              const QString& buildSystem = QString(),
                              const QString& testId = QString());
    Q_INVOKABLE void discoverTests(const QString& buildSystem = QString());
    Q_INVOKABLE void runQuality(const QString& buildSystem = QString());
    Q_INVOKABLE void cancelBuild();
    // Cancela um job pelo id (F2 da Etapa 2: o cancelar da barra de status).
    Q_INVOKABLE void cancelJob(const QString& jobId);
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
    // M-S1 (DocsPublic/seguranca/23): autosave/limpeza de rascunho não salvo (rede de segurança).
    Q_INVOKABLE void draftSave(const QString& path, const QString& content);
    Q_INVOKABLE void draftClear(const QString& path);
    Q_INVOKABLE void findFiles(const QString& query);
    Q_INVOKABLE void searchInFiles(const QString& query, bool caseSensitive);
    Q_INVOKABLE void replaceInFiles(const QString& query, const QString& replacement,
                                    bool caseSensitive);
    Q_INVOKABLE void runStart(const QString& command, const QString& device = QString());
    Q_INVOKABLE void runScript(const QString& path, const QString& device = QString());
    Q_INVOKABLE void runStdin(const QString& data);
    Q_INVOKABLE void runStop();
    // D2.3 (DocsPublic/roadmaps/24): multi-terminal — todo comando leva o id da sessão.
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
    // M-S1 (DocsPublic/seguranca/23): rascunhos não salvos recuperados de um crash.
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
    void runCapabilitiesListed(const QVariantList& runnable, const QVariantList& debuggable);
    void cmakePresetsResolved(const QVariantList& presets);
    void cmakeStatusResolved(bool configured, bool hasCompileCommands, bool cdbStale,
                             const QString& cdbStaleBecause, const QString& preset);
    void cmakeConfigureFinished(bool success);
    /// `origin` diz de onde os nomes vieram: "fileApi" (confirmados por um
    /// configure), "source" (lidos do CMakeLists) ou "none".
    void cmakeTargetsResolved(const QVariantList& targets, const QString& origin);
    void cargoMetadataResolved(int packages);
    void setupListResolved(const QString& distroName, const QString& family,
                           const QVariantList& tools);
    void dataSourceListResolved(const QVariantList& profiles);
    void dataSourceTestAccepted(const QString& jobId);
    /// `candidates` com o perfil pronto de cada um; `hint` quando vazio.
    void dataSourceDiscovered(const QVariantList& candidates, const QString& containerEngine,
                              const QString& hint);
    /// `profile` preenchido = criado na hora (SQLite); `jobId` = servidor
    /// subindo, com o `command` exato para a tela mostrar.
    void dataSourceCreateResolved(const QVariantMap& profile, const QString& jobId,
                                  const QString& command);
    /// Desfecho do servidor em container: perfil salvo ou o motivo.
    void dataSourceCreated(bool success, const QVariantMap& profile, const QString& message);
    void dataSourceQueried(const QVariantMap& outcome);
    /// Veredito do teste de conexao. `secretRequired` diz para PEDIR A SENHA;
    /// a UI nunca decide isso lendo `message`, que vem localizada do servidor.
    void dataSourceTested(const QString& name, bool ok, const QString& serverVersion,
                          const QString& message, bool secretRequired);
    /// Estrutura lida do banco. DUAS FORMAS, e exatamente uma vem preenchida:
    /// `schemas` e' `esquema -> tabela -> coluna` dos motores relacionais;
    /// `collections` e' `colecao -> campo` do MongoDB, onde campo tem presenca,
    /// pode ter mais de um tipo e aninha. Forcar o segundo no primeiro faria a
    /// tela afirmar tres coisas falsas.
    void dataSourceIntrospected(const QString& name, bool ok, const QVariantList& schemas,
                                const QVariantList& collections, const QString& message,
                                bool secretRequired);
    /// A instancia salva neste workspace. `exists` distingue "nao ha' nenhuma"
    /// de "ha' uma com campos vazios" — a tela desenha coisas diferentes.
    void grafanaProfileResolved(const QVariantMap& profile, bool exists);
    void grafanaProbeAccepted(const QString& jobId);
    /// O que a sonda achou. Um mapa so' porque o resultado e' composto:
    /// alcance, autenticacao, fontes de dados, dashboards e o CRUZAMENTO com
    /// os perfis de banco deste workspace.
    void grafanaProbed(const QVariantMap& result);

    void libraryListResolved(const QVariantList& libraries);
    void libraryPlanResolved(const QVariantMap& plan);
    void toolchainResolved(const QVariantList& selections, const QVariantList& candidates,
                           const QString& preset, const QString& sysroot,
                           const QString& targetTriple, const QString& chip,
                           const QString& presetToolchainFile);
    void toolchainAdvice(const QString& sysrootHint, const QVariantList& rustTargets,
                         bool rustTargetsKnown);
    /// O arquivo de toolchain do KIT (0.104.0), ao lado do toolchainResolved —
    /// sinal proprio para o de cima nao crescer em argumento posicional.
    void toolchainKitFileResolved(const QString& toolchainFile);
    /// O SVD do KIT (0.118.0, P3): os registradores de periferico no depurador.
    void toolchainKitSvdResolved(const QString& svdFile);
    /// `toolchain.inspectSysroot`: o que a pasta contem (SysrootReport).
    void sysrootInspected(const QVariantMap& report);
    /// `toolchain.importKit`: a proposta de kit lida do SDK (KitImport).
    void kitImported(const QVariantMap& proposal);
    /// O catalogo de toolchains instalaveis (integracoes/39 §5): URL, tamanho,
    /// sha256, licenca e fonte de cada uma, o estado nesta maquina e a
    /// familia que o projeto aberto recomenda.
    void toolchainInstallableResolved(const QVariantList& toolchains, const QString& installRoot,
                                      const QString& projectFamily);
    /// `event.toolchain.installed`: o job acabou (jobId, id, version, path,
    /// success, error).
    void toolchainInstalled(const QVariantMap& outcome);
    void probesResolved(const QVariantList& probes, bool toolAvailable, const QString& rawOutput,
                        const QString& hint);
    void buildSizeResolved(const QVariantList& sections, const QVariantList& regions,
                           bool toolAvailable, const QString& tool, const QString& rawOutput);
    void serialPortsResolved(const QVariantList& ports, const QString& hint);
    void serialMonitorOpened(const QString& id, const QString& command, const QString& tool);
    void serialIdentifyStarted(const QString& jobId, const QString& command);
    void serialIdentified(const QVariantMap& outcome);
    void serialAccessResolved(const QVariantList& channels);
    void serialFilesStarted(const QString& jobId, const QString& command);
    void serialFilesResolved(const QVariantMap& outcome);
    void projectModelResolved(const QVariantMap& model);
    void projectChanged(const QVariantMap& model);
    void indexStatusResolved(const QVariantMap& stats);
    void indexSymbolsResolved(const QVariantList& symbols, int total, const QString& state);
    void indexContextResolved(const QVariantMap& context);
    void pythonStatusResolved(const QVariantMap& status);
    void pythonEnvironmentAccepted(const QString& jobId);
    void pythonEnvironmentFinished(const QVariantMap& outcome);
    void pythonStubsAccepted(const QString& jobId, const QString& package);
    void pythonStubsFinished(const QVariantMap& outcome);
    void coverageFinished(const QVariantMap& outcome);
    void coverageLinesResolved(const QString& file, bool known, const QVariantList& covered,
                               const QVariantList& missed);
    void remoteTargetsResolved(const QVariantList& targets);
    void remoteJobAccepted(const QString& method, const QString& jobId, const QString& command);
    void remoteCommandResolved(const QVariantMap& result);
    void remoteProbed(const QVariantMap& outcome);
    void remoteDeployed(const QVariantMap& outcome);
    void remoteOpenAccepted(const QString& jobId, const QString& command, const QString& mirror);
    void remoteSynced(const QVariantMap& outcome);
    // O espelho que o workspace aberto e' (vazio = nao e' espelho); sai do
    // workspace.open e do remote.status.
    void remoteMirrorChanged(const QVariantMap& mirror);
    void indexProgressed(int files, int symbols);
    void indexFinished(const QVariantMap& stats);
    void containerStatusResolved(const QVariantMap& status);
    void containersResolved(const QVariantList& containers, const QString& engine,
                            const QString& rawOutput, const QString& hint);
    void containerImagesResolved(const QVariantList& images, const QString& hint);
    void containerActionAccepted(const QString& jobId);
    void containerTerminalOpened(const QString& id, const QString& command);
    void containerFinished(const QVariantMap& event);
    void configActionsListed(const QVariantList& actions, const QStringList& activeBuildSystems);
    void configActionPreviewed(const QVariantMap& preview);
    void configActionApplied(const QString& id, const QString& message, const QStringList& files,
                             const QString& jobId);
    void runConfigsResolved(const QVariantList& configs, const QString& activeId);
    void flashProposalResolved(const QVariantMap& proposal);
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
    void testFinished(bool success, int passed, int failed, int ignored, const QString& error);
    /// `event.test.discovered`: a arvore de testes antes do run (jobId,
    /// runner, command, tests[], success, error).
    void testsDiscovered(const QVariantMap& outcome);
    void analyzingChanged();
    void qualityStarted(const QString& command);
    void qualityOutput(const QString& line, const QString& stream);
    void qualityDiagnostic(const QVariantMap& diagnostic);
    void qualityFinished(bool success, int exitCode, int diagnostics);
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
    // running | starting | failed | exited | stopped | restarting, com o motivo quando ha'.
    void lspStatusChanged(const QString& language, const QString& status, const QString& message);
    void lspDocumentsClosed(const QString& language, int count);
    void fileSearchResults(const QVariantList& matches, bool truncated);
    void searchResults(const QVariantList& matches, bool truncated);
    void filesReplaced(const QStringList& files, int replacements);
    void runningChanged();
    void runStarted(const QString& command);
    void runOutput(const QString& line, const QString& stream);
    void runFinished(bool success, int exitCode);
    void debuggingChanged();
    void debugStarted(const QString& program, bool attached);
    void debugOutput(const QString& category, const QString& line);
    void debugStopped(const QString& reason, const QString& file, int line, int threadId);
    void debugContinued();
    void debugFinished(int exitCode);
    void debugStackTraceResolved(const QVariantList& frames);
    void debugVariablesResolved(double frameId, double ref, const QVariantList& variables);
    void debugScopesResolved(double frameId, const QVariantList& scopes);
    void debugMemoryResolved(const QVariantMap& memory);
    void debugDisassemblyResolved(const QVariantList& instructions);
    void debugEvaluateResolved(const QString& expression, const QString& value,
                               const QString& typeName, double ref);
    void debugEvaluateFailed(const QString& expression, const QString& message);
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
    // D2 (DocsPublic/roadmaps/24): grid renderizável do terminal (cols/rows/cursor/lines).
    // Desde a D2.3 o mapa carrega `id`: a UI roteia pro terminal certo.
    void terminalRender(const QVariantMap& render);
    void terminalClosed(const QString& id, int exitCode);
    /// Recusa do core. `code` e' o `JsonRpcErrorCode` estavel — `SECRET_REQUIRED`,
    /// `INVALID_PARAMS`, ... — e existe para a UI decidir por ELE, nunca pelo
    /// texto de `message`, que vem localizado e muda.
    ///
    /// Ate' 2026-09-04 o codigo era descartado aqui, e o comentario do
    /// `JsonRpcErrorCode::SecretRequired` no protocolo dizia com todas as
    /// letras que ele existia para evitar casamento por texto — evitar algo
    /// que a UI nao tinha como fazer de outro jeito. Handlers QML que declaram
    /// menos parametros continuam validos.
    void requestFailed(const QString& method, const QString& message, const QString& code);

private:
    void handleStarted();
    void handleStdout();
    void handleStderr();
    void handleFinished(int exitCode, QProcess::ExitStatus exitStatus);
    void handleErrorOccurred(QProcess::ProcessError error);
    void handleResponseLine(const QByteArray& line);
    void handleNotification(const QString& method, const QJsonObject& params);
    bool handleRunnerNotification(const QString& method, const QJsonObject& params);
    bool handleFileSystemNotification(const QString& method, const QJsonObject& params);
    bool handleTerminalNotification(const QString& method, const QJsonObject& params);
    bool handleLspNotification(const QString& method, const QJsonObject& params);
    bool handleEnvironmentNotification(const QString& method, const QJsonObject& params);
    bool handleJobNotification(const QString& method, const QJsonObject& params);
    bool handleCmakeNotification(const QString& method, const QJsonObject& params);
    bool handleDebugNotification(const QString& method, const QJsonObject& params);
    void dispatchResult(const QString& method, const QJsonObject& result);
    bool dispatchFileResult(const QString& method, const QJsonObject& result);
    bool dispatchLspResult(const QString& method, const QJsonObject& result);
    bool dispatchSyntaxResult(const QString& method, const QJsonObject& result);
    bool dispatchCmakeResult(const QString& method, const QJsonObject& result);
    bool dispatchConfigActionResult(const QString& method, const QJsonObject& result);
    bool dispatchToolchainResult(const QString& method, const QJsonObject& result);
    bool dispatchDataSourceResult(const QString& method, const QJsonObject& result);
    bool dispatchGrafanaResult(const QString& method, const QJsonObject& result);
    bool dispatchProbeResult(const QString& method, const QJsonObject& result);
    bool dispatchBuildSizeResult(const QString& method, const QJsonObject& result);
    bool dispatchSerialResult(const QString& method, const QJsonObject& result);
    bool dispatchContainerResult(const QString& method, const QJsonObject& result);
    bool dispatchIndexResult(const QString& method, const QJsonObject& result);
    bool dispatchPythonResult(const QString& method, const QJsonObject& result);
    bool dispatchCoverageResult(const QString& method, const QJsonObject& result);
    bool dispatchRemoteResult(const QString& method, const QJsonObject& result);
    bool dispatchLibraryResult(const QString& method, const QJsonObject& result);
    bool dispatchDebugResult(const QString& method, const QJsonObject& result);
    bool dispatchWorkspaceResult(const QString& method, const QJsonObject& result);
    void handleWorkspaceOpened(const QJsonObject& result);
    void storeJobId(const QString& method, const QString& jobId);
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
    // O preset do kit ativo, como a tela o pediu por toolchain.get ("" = padrao).
    QString m_kitPreset;
    // M4.3: recuperacao de crash do core. m_lastWorkspaceRoot sobrevive ao
    // crash (o que a recuperacao reabre); m_recovering suprime o session
    // restore e sinaliza a UI; a janela+contador cortam loop de fork.
    QString m_lastWorkspaceRoot;
    // A pasta passada na linha de comando, aberta assim que o core responder.
    QString m_startupWorkspace;
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
