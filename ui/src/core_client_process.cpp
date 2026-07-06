#include "core_client.h"

#include <QCoreApplication>
#include <QDir>
#include <QFileInfo>
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
    setScanningEnvironment(false);
    m_buildJobId.clear();
    m_testJobId.clear();
    m_qualityJobId.clear();
    m_environmentJobId.clear();
    setStatus(QStringLiteral("desconectado"), false);
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
