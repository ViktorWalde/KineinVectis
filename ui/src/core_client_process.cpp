#include "core_client.h"

#include "cli_args.h"
#include "log_redaction.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
#include <QJsonArray>
#include <QJsonDocument>
#include <QStandardPaths>

namespace kinein {

void CoreClient::start()
{
    if (m_process.state() != QProcess::NotRunning) {
        return;
    }

    const QString binary = resolveCoreBinary();
    if (binary.isEmpty()) {
        setStatus(QStringLiteral("kinein-core nao encontrado"), false);
        appendErrorLog(QStringLiteral("erro: kinein-core nao foi encontrado. Compile com "
                                      "'cargo build -p kinein-core' ou defina KINEIN_CORE_BIN."));
        return;
    }

    // `kinein-vectis <pasta>` abre o projeto direto (Etapa 2, 2026-09-18): e'
    // o que um lancador, um `xdg-open` de pasta e o gate headless precisam.
    //
    // Ate' 2026-09-24 isto varria o argv aqui mesmo, pegando o primeiro
    // argumento que fosse pasta existente e IGNORANDO EM SILENCIO o resto: um
    // caminho com erro de digitacao abria a IDE sem projeto e sem dizer por
    // que. Agora quem decide e' o `cli_args`, que tem teste; o `main` ja'
    // recusou o que nao presta antes de a UI subir, entao aqui so' chega
    // caminho bom.
    const kinein::cli::Arguments request =
        kinein::cli::parse(QCoreApplication::arguments().mid(1), QDir::currentPath());
    if (request.action == kinein::cli::Action::Open) {
        m_startupWorkspace = request.folder;
    }
    setStatus(QStringLiteral("iniciando..."), false);
    appendLog(QStringLiteral("iniciando core: %1").arg(binary));
    m_process.setProgram(binary);
    m_process.setArguments({});
    m_process.start();
}

void CoreClient::handleStarted()
{
    appendLog(QStringLiteral("processo do core iniciado (pid %1)").arg(m_process.processId()));
    ping();
    // M4.3: recuperacao de crash — reabre o mesmo workspace no core novo.
    // O session restore e suprimido em handleWorkspaceOpened (m_recovering),
    // entao as abas/edicoes da UI sao preservadas.
    if (m_recovering && !m_lastWorkspaceRoot.isEmpty()) {
        appendLog(QStringLiteral("recuperando workspace: %1").arg(m_lastWorkspaceRoot));
        openWorkspace(m_lastWorkspaceRoot);
    }
    else if (!m_startupWorkspace.isEmpty()) {
        appendLog(QStringLiteral("abrindo projeto do argumento: %1").arg(m_startupWorkspace));
        openWorkspace(m_startupWorkspace);
        m_startupWorkspace.clear();
    }
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
    // O destrutor desconecta os sinais antes de fechar limpo, entao qualquer
    // handleFinished aqui e uma saida INESPERADA com a IDE viva.
    m_pendingMethods.clear();
    m_pendingPaths.clear();
    m_pendingRemoteDirectoryNames.clear();
    m_pendingDataSourceQueries.clear();
    setBuilding(false);
    setTesting(false);
    setAnalyzing(false);
    setRunning(false);
    m_runTerminalId.clear();
    m_terminalIds.clear();
    setTerminalActive(false);
    setScanningEnvironment(false);
    m_buildJobId.clear();
    m_testJobId.clear();
    m_qualityJobId.clear();
    m_environmentJobId.clear();
    m_stdoutBuffer.clear();
    setStatus(QStringLiteral("desconectado"), false);

    // M4.3: sem workspace aberto, nada a recuperar (o usuario escolhe a
    // pasta). Com workspace, tenta relancar — com guarda anti-loop de fork.
    if (m_lastWorkspaceRoot.isEmpty()) {
        return;
    }
    constexpr int kMaxAttempts = 3;
    constexpr qint64 kWindowMs = 4000;
    if (m_recoveryWindow.isValid() && m_recoveryWindow.elapsed() < kWindowMs) {
        m_recoveryAttempts++;
    }
    else {
        m_recoveryAttempts = 1;
    }
    m_recoveryWindow.restart();
    if (m_recoveryAttempts > kMaxAttempts) {
        setRecovering(false);
        appendErrorLog(QStringLiteral(
            "core caiu repetidamente; recuperacao pausada. Veja o log e reabra o projeto."));
        setStatus(QStringLiteral("core caiu repetidamente"), false);
        return;
    }
    setRecovering(true);
    setStatus(QStringLiteral("recuperando..."), false);
    start();
}

void CoreClient::handleErrorOccurred(QProcess::ProcessError error)
{
    appendErrorLog(QStringLiteral("erro de processo: %1").arg(m_process.errorString()));
    if (error == QProcess::FailedToStart) {
        setStatus(QStringLiteral("falha ao iniciar o core"), false);
    }
}

void CoreClient::sendRequest(const QString& method, const QJsonObject& params)
{
    if (m_process.state() != QProcess::Running) {
        appendLog(QStringLiteral("ignorando %1: core nao esta rodando").arg(method));
        return;
    }

    const qint64 id = m_nextRequestId++;
    m_pendingMethods.insert(id, method);
    if (method == QStringLiteral("datasource.query") ||
        method == QStringLiteral("datasource.impact") ||
        method == QStringLiteral("datasource.test") ||
        method == QStringLiteral("datasource.introspect") ||
        method == QStringLiteral("datasource.destroy"))
    {
        m_pendingDataSourceQueries.insert(
            id, {{QStringLiteral("name"), params.value(QStringLiteral("name")).toString()},
                 {QStringLiteral("sql"), params.value(QStringLiteral("sql")).toString()},
                 {QStringLiteral("confirmWrite"),
                  params.value(QStringLiteral("confirmWrite")).toBool()},
                 {QStringLiteral("maxRows"), params.value(QStringLiteral("maxRows")).toInt()},
                 {QStringLiteral("clientContext"),
                  params.value(QStringLiteral("clientContext")).toString()}});
    }
    const QJsonValue path = method == QStringLiteral("fs.copy")
                                ? params.value(QStringLiteral("to"))
                                : params.value(QStringLiteral("path"));
    if (path.isString()) {
        m_pendingPaths.insert(id, path.toString());
    }
    if (method == QStringLiteral("remote.directories")) {
        m_pendingRemoteDirectoryNames.insert(id, params.value(QStringLiteral("name")).toString());
    }

    const QJsonObject request{
        {QStringLiteral("jsonrpc"), QStringLiteral("2.0")},
        {QStringLiteral("id"), id},
        {QStringLiteral("method"), method},
        {QStringLiteral("params"), params},
    };
    const QByteArray payload = QJsonDocument(request).toJson(QJsonDocument::Compact) + '\n';
    // Teclas/paste podem conter senhas sem um campo chamado "password".
    if (method != QStringLiteral("lsp.didChange") &&
        method != QStringLiteral("syntaxTree.update") && method != QStringLiteral("terminal.input"))
    {
        const QByteArray registered =
            QJsonDocument(kinein::redactSecrets(request)).toJson(QJsonDocument::Compact);
        appendLog(QStringLiteral("-> %1").arg(QString::fromUtf8(registered.left(200).trimmed())));
    }
    m_process.write(payload);
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
