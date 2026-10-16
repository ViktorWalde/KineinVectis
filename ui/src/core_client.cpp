#include "core_client.h"

#include "qt_message_log.h"

#include <QDir>
#include <QJsonObject>

namespace kinein {

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

bool CoreClient::workspaceTrusted() const
{
    return m_workspaceTrusted;
}

void CoreClient::setWorkspaceTrusted(bool trusted)
{
    if (m_workspaceTrusted == trusted) {
        return;
    }
    m_workspaceTrusted = trusted;
    emit workspaceTrustedChanged();
}

// 16.4b (roadmaps/59 §7.1.4): o gesto que libera o CMake automatico e o
// rust-analyzer completo. A confianca fica fora do projeto, no core.
void CoreClient::trustWorkspace()
{
    sendRequest(QStringLiteral("workspace.trust"), QJsonObject{});
}

QStringList CoreClient::workspaceBuildSystems() const
{
    return m_workspaceBuildSystems;
}

QString CoreClient::homeDir()
{
    return QDir::homePath();
}

QStringList CoreClient::startupCommands()
{
    const QString raw = QString::fromUtf8(qgetenv("KINEIN_STARTUP_COMMANDS"));
    QStringList commands;
    for (const QString& part : raw.split(QLatin1Char(','), Qt::SkipEmptyParts)) {
        commands.append(part.trimmed());
    }
    return commands;
}

QString CoreClient::errorLogFile()
{
    // Um dono so' para o caminho: o handler das mensagens do Qt grava no mesmo.
    return diagnosticLogPath();
}

} // namespace kinein
