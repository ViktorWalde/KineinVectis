#include "startup_web.h"

#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <optional>

namespace kinein {

namespace {

// `true`/`false` quando o arquivo declara a opcao; nada quando nao declara.
std::optional<bool> readOption(const QString& file)
{
    if (file.isEmpty()) {
        return std::nullopt;
    }
    QFile settings(file);
    if (!settings.open(QIODevice::ReadOnly)) {
        return std::nullopt;
    }
    const QJsonDocument document = QJsonDocument::fromJson(settings.readAll());
    const QJsonValue value = document.object().value(QStringLiteral("grafanaWebView"));
    if (!value.isBool()) {
        return std::nullopt;
    }
    return value.toBool();
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

bool wantsWebView(const QString& globalFile, const QString& workspaceFile)
{
    if (const std::optional<bool> workspace = readOption(workspaceFile)) {
        return *workspace;
    }
    return readOption(globalFile).value_or(false);
}

} // namespace kinein
