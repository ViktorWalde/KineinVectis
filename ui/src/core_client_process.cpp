#include "core_client.h"

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
    // So' o primeiro argumento que e' uma pasta existente; o resto e' do Qt.
    const QStringList argumentos = QCoreApplication::arguments();
    for (qsizetype i = 1; i < argumentos.size(); ++i) {
        const QString& candidato = argumentos.at(i);
        if (!candidato.startsWith(QLatin1Char('-')) && QDir(candidato).exists()) {
            m_startupWorkspace = QDir(candidato).absolutePath();
            break;
        }
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
        appendLog(QStringLiteral("abrindo workspace do argumento: %1").arg(m_startupWorkspace));
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

namespace {

/// Nomes de campo cujo VALOR nunca pode aparecer no log.
///
/// POR QUE EXISTE (2026-09-04). O log do cliente registra os 200 primeiros
/// bytes de cada pedido enviado ao core, e `appendErrorLog` grava em ARQUIVO.
/// Quando o dominio `datasource` passou a poder mandar a senha da sessao em
/// `params`, esse log virou o caminho mais curto para a senha sair do processo
/// — exatamente a falha que o tipo `Secret` do core fecha do lado Rust
/// (`DocsPublic/seguranca/40`). Redigir por NOME de campo fecha o caminho aqui.
///
/// A lista e' de nomes, nao de metodos, de proposito: um metodo novo que mande
/// `password` ja' nasce protegido, sem ninguem lembrar de acrescenta-lo.
bool isSecretField(const QString& key)
{
    static const QStringList kSecretKeys{
        QStringLiteral("password"), QStringLiteral("passwd"),     QStringLiteral("secret"),
        QStringLiteral("token"),    QStringLiteral("credential"),
    };
    return kSecretKeys.contains(key, Qt::CaseInsensitive);
}

/// Copia o objeto trocando por `***` o valor de todo campo sensivel, em
/// qualquer profundidade.
QJsonValue redactValue(const QJsonValue& value);

QJsonObject redactObject(const QJsonObject& object)
{
    QJsonObject limpo;
    for (auto it = object.constBegin(); it != object.constEnd(); ++it) {
        limpo.insert(it.key(), isSecretField(it.key()) ? QJsonValue(QStringLiteral("***"))
                                                       : redactValue(it.value()));
    }
    return limpo;
}

QJsonValue redactValue(const QJsonValue& value)
{
    if (value.isObject()) {
        return redactObject(value.toObject());
    }
    if (value.isArray()) {
        QJsonArray limpo;
        const QJsonArray original = value.toArray();
        for (const auto item : original) {
            limpo.append(redactValue(item));
        }
        return limpo;
    }
    return value;
}

QJsonObject redactSecrets(const QJsonObject& request)
{
    return redactObject(request);
}

} // namespace

void CoreClient::sendRequest(const QString& method, const QJsonObject& params)
{
    if (m_process.state() != QProcess::Running) {
        appendLog(QStringLiteral("ignorando %1: core nao esta rodando").arg(method));
        return;
    }

    const qint64 id = m_nextRequestId++;
    m_pendingMethods.insert(id, method);
    const QJsonValue path = params.value(QStringLiteral("path"));
    if (path.isString()) {
        m_pendingPaths.insert(id, path.toString());
    }

    const QJsonObject request{
        {QStringLiteral("jsonrpc"), QStringLiteral("2.0")},
        {QStringLiteral("id"), id},
        {QStringLiteral("method"), method},
        {QStringLiteral("params"), params},
    };
    const QByteArray payload = QJsonDocument(request).toJson(QJsonDocument::Compact) + '\n';
    if (method != QStringLiteral("lsp.didChange") && method != QStringLiteral("syntaxTree.update"))
    {
        const QByteArray registrado =
            QJsonDocument(redactSecrets(request)).toJson(QJsonDocument::Compact);
        appendLog(QStringLiteral("-> %1").arg(QString::fromUtf8(registrado.left(200).trimmed())));
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
