#include "core_client.h"

#include <QDir>
#include <QStandardPaths>

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

QStringList CoreClient::workspaceBuildSystems() const
{
    return m_workspaceBuildSystems;
}

QString CoreClient::homeDir()
{
    return QDir::homePath();
}

QString CoreClient::errorLogFile()
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation) +
           QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt");
}

} // namespace kinein
