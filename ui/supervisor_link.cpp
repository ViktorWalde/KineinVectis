#include "supervisor_link.h"

#include <QJsonDocument>
#include <QJsonObject>

namespace prova {

SupervisorLink::SupervisorLink(QObject* parent) : QObject(parent)
{
    // O stdout traz os eventos; o stderr do supervisor vai direto para o da UI.
    m_process.setProcessChannelMode(QProcess::ForwardedErrorChannel);
    m_process.setReadChannel(QProcess::StandardOutput);
    connect(&m_process, &QProcess::readyReadStandardOutput, this, &SupervisorLink::onReadyRead);
    connect(&m_process, &QProcess::finished, this, [this](int code, QProcess::ExitStatus) {
        m_error = QStringLiteral("o supervisor saiu (código %1)").arg(code);
        emit errorChanged();
    });
}

void SupervisorLink::start(const QStringList& command)
{
    if (command.isEmpty()) {
        m_error = QStringLiteral("sem comando do supervisor");
        emit errorChanged();
        return;
    }
    m_process.start(command.first(), command.mid(1));
}

void SupervisorLink::stop()
{
    if (m_process.state() == QProcess::NotRunning) {
        return;
    }
    // Fim do stdin: o supervisor grava o rascunho e encerra o resto sozinho.
    m_process.closeWriteChannel();
    if (!m_process.waitForFinished(5000)) {
        m_process.kill();
        m_process.waitForFinished(1000);
    }
}

void SupervisorLink::onReadyRead()
{
    m_buffer.append(m_process.readAllStandardOutput());
    qsizetype newline = 0;
    while ((newline = m_buffer.indexOf('\n')) >= 0) {
        const QByteArray line = m_buffer.left(newline);
        m_buffer.remove(0, newline + 1);
        const QJsonObject event = QJsonDocument::fromJson(line).object();
        m_events.append(event.toVariantMap());
        const QString kind = event.value(QStringLiteral("evento")).toString();
        if (kind == QLatin1String("nvim_pronto")) {
            m_socketPath = event.value(QStringLiteral("socket")).toString();
            emit socketPathChanged();
            if (!m_error.isEmpty()) {
                m_error.clear();
                emit errorChanged();
            }
        } else if (kind == QLatin1String("nvim_saiu")) {
            m_socketPath.clear();
            emit socketPathChanged();
        } else if (kind == QLatin1String("contencao_falhou") || kind == QLatin1String("nvim_falhou")) {
            m_error = QStringLiteral("O editor não foi entregue: a contenção do Neovim falhou. %1")
                          .arg(QString::fromUtf8(line));
            emit errorChanged();
        }
    }
}

} // namespace prova
