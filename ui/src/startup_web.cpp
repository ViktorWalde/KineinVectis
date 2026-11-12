#include "startup_web.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>

namespace kinein {

namespace {

// O limite de leitura do core (`settings/file.rs`): o arquivo maior nao e' lido.
constexpr qint64 kReadLimit = 1024LL * 1024;

// O objeto do arquivo quando o core tambem o leria, senao vazio (INF-14). O
// tipo vem antes do `open`, porque abrir um FIFO prende a partida (medido em
// 2026-11-12: sem primeiro quadro, parada no `open`). O JSON quebrado, o que
// nao e' UTF-8 e o que nao e' objeto chegam vazios, sem `schemaVersion`.
QJsonObject readSettings(const QString& file)
{
    if (!QFileInfo(file).isFile()) {
        return {};
    }
    QFile settings(file);
    if (!settings.open(QIODevice::ReadOnly)) {
        return {};
    }
    const QByteArray body = settings.read(kReadLimit + 1);
    if (body.size() > kReadLimit) {
        return {};
    }
    QJsonObject object = QJsonDocument::fromJson(body).object();
    if (object.value(QStringLiteral("schemaVersion")).toInteger(0) != 1) {
        return {};
    }
    return object;
}

} // namespace

QString globalSettingsFile()
{
#ifdef Q_OS_WIN
    QString base = qEnvironmentVariable("APPDATA");
    if (base.isEmpty()) {
        base = QDir::homePath() + QStringLiteral("/AppData/Roaming");
    }
#else
    QString base = qEnvironmentVariable("XDG_CONFIG_HOME");
    if (base.isEmpty() || !QDir::isAbsolutePath(base)) {
        base = QDir::homePath() + QStringLiteral("/.config");
    }
#endif
    return QDir(base).filePath(QStringLiteral("kinein-vectis/settings.json"));
}

bool wantsWebView(const QString& globalFile)
{
    // As opcoes que pedem a visualizacao web: o Grafana (59 §6.1) e o PIO
    // Home (52 §14.45). Qualquer uma ligada liga o contexto compartilhado.
    const QJsonObject settings = readSettings(globalFile);
    return settings.value(QStringLiteral("grafanaWebView")).toBool() ||
           settings.value(QStringLiteral("platformioWebView")).toBool();
}

} // namespace kinein
