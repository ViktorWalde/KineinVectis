#include "core_client.h"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QTextStream>
#include <QTime>

namespace kinein {

namespace {

constexpr int kMaxLogLines = 500;

QString timestamp()
{
    return QTime::currentTime().toString(QStringLiteral("HH:mm:ss.zzz"));
}

} // namespace

void CoreClient::appendLog(const QString& line)
{
    m_logLines.append(QStringLiteral("[%1] %2").arg(timestamp(), line));
    while (m_logLines.size() > kMaxLogLines) {
        m_logLines.removeFirst();
    }
    emit logChanged();
}

void CoreClient::appendErrorLog(const QString& line)
{
    appendLog(line);

    const QString path = errorLogFile();
    if (!QDir().mkpath(QFileInfo(path).absolutePath())) {
        return;
    }
    QFile file(path);
    if (file.open(QIODevice::Append | QIODevice::Text)) {
        QTextStream stream(&file);
        stream << QDateTime::currentDateTime().toString(Qt::ISODate) << ' ' << line << '\n';
    }
}

void CoreClient::setStatus(const QString& status, bool connected)
{
    if (m_status == status && m_connected == connected) {
        return;
    }
    m_status = status;
    m_connected = connected;
    emit statusChanged();
}

} // namespace kinein
