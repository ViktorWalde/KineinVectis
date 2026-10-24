#include "startup_web.h"

#include <QDir>
#include <QFile>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonValue>
#include <optional>

namespace kinein {

namespace {

// `true`/`false` quando o arquivo declara `option`; nada quando nao declara.
std::optional<bool> readOption(const QString& file, const QString& option)
{
    if (file.isEmpty()) {
        return std::nullopt;
    }
    QFile settings(file);
    if (!settings.open(QIODevice::ReadOnly)) {
        return std::nullopt;
    }
    const QJsonDocument document = QJsonDocument::fromJson(settings.readAll());
    const QJsonValue value = document.object().value(option);
    if (!value.isBool()) {
        return std::nullopt;
    }
    return value.toBool();
}

// A do projeto vence a global; nenhuma das duas declarando conta desligada.
bool wantsOption(const QString& globalFile, const QString& workspaceFile, const QString& option)
{
    const std::optional<bool> workspace = readOption(workspaceFile, option);
    return workspace.value_or(readOption(globalFile, option).value_or(false));
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
    // As opcoes que pedem a visualizacao web: o Grafana (59 §6.1) e o PIO
    // Home (52 §14.45). Qualquer uma ligada liga o contexto compartilhado.
    return wantsOption(globalFile, workspaceFile, QStringLiteral("grafanaWebView")) ||
           wantsOption(globalFile, workspaceFile, QStringLiteral("platformioWebView"));
}

} // namespace kinein
