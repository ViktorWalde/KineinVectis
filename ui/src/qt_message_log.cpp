#include "qt_message_log.h"

#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QStandardPaths>
#include <QTextStream>

namespace kinein {

namespace {

// Acima disto o arquivo vira `.1` e recomeca: um defeito que repete nao pode
// encher o disco de quem so' esta' usando a IDE.
constexpr qint64 kMaxLogBytes = qint64{1024} * 1024;

// O handler que estava instalado antes (o padrao do Qt, que escreve no
// stderr). Acessor com estatica local: global mutavel e' o que o clang-tidy
// recusa, e com razao.
QtMessageHandler& previousHandler()
{
    static QtMessageHandler handler = nullptr;
    return handler;
}

const char* levelName(QtMsgType type)
{
    switch (type) {
    case QtDebugMsg:
        return "debug";
    case QtInfoMsg:
        return "info";
    case QtWarningMsg:
        return "aviso";
    case QtCriticalMsg:
        return "critico";
    case QtFatalMsg:
        return "fatal";
    }
    return "?";
}

void writeMessage(QtMsgType type, const QMessageLogContext& context, const QString& message)
{
    // Debug/info do Qt nao e' defeito: nao vai para o arquivo.
    if (type != QtDebugMsg && type != QtInfoMsg) {
        const QString path = diagnosticLogPath();
        if (QDir().mkpath(QFileInfo(path).absolutePath())) {
            if (QFileInfo(path).size() > kMaxLogBytes) {
                QFile::remove(path + QStringLiteral(".1"));
                QFile::rename(path, path + QStringLiteral(".1"));
            }
            QFile file(path);
            if (file.open(QIODevice::Append | QIODevice::Text)) {
                QTextStream stream(&file);
                stream << QDateTime::currentDateTime().toString(Qt::ISODate) << " qt-"
                       << levelName(type) << ' ' << message << '\n';
            }
        }
    }
    if (previousHandler() != nullptr) {
        previousHandler()(type, context, message);
    }
}

} // namespace

QString diagnosticLogPath()
{
    return QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation) +
           QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt");
}

void installQtMessageLog()
{
    previousHandler() = qInstallMessageHandler(writeMessage);
}

} // namespace kinein
