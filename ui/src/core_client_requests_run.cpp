// Dominio RUN no lado da UI: os pedidos `run.*` e `test.*`.
//
// Mesmo precedente do `_git.cpp` e do `_debug.cpp`: o requests.cpp esta na
// catraca e a §5 manda dividir por dominio.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::runConfigList()
{
    sendRequest(QStringLiteral("runConfig.list"), QJsonObject{});
}

void CoreClient::runConfigSave(const QString& id, const QString& name, const QString& command)
{
    QJsonObject params{{QStringLiteral("name"), name}, {QStringLiteral("command"), command}};
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("runConfig.save"), params);
}

void CoreClient::runConfigDelete(const QString& id)
{
    sendRequest(QStringLiteral("runConfig.delete"), QJsonObject{{QStringLiteral("id"), id}});
}

void CoreClient::runConfigSetActive(const QString& id)
{
    QJsonObject params;
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("runConfig.setActive"), params);
}

void CoreClient::runBuild(const QString& buildSystem)
{
    if (m_building || m_process.state() != QProcess::Running) {
        return;
    }
    setBuilding(true);
    QJsonObject params;
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    sendRequest(QStringLiteral("build.run"), params);
}

void CoreClient::runTests(const QString& filter, const QString& buildSystem, const QString& testId)
{
    if (m_testing || m_process.state() != QProcess::Running) {
        return;
    }
    QJsonObject params;
    if (!filter.trimmed().isEmpty()) {
        params.insert(QStringLiteral("filter"), filter);
    }
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    // UM teste pelo id que o test.discover deu (2026-09-13); vence o filter.
    if (!testId.trimmed().isEmpty()) {
        params.insert(QStringLiteral("testId"), testId);
    }
    setTesting(true);
    sendRequest(QStringLiteral("test.run"), params);
}

void CoreClient::discoverTests(const QString& buildSystem)
{
    QJsonObject params;
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    sendRequest(QStringLiteral("test.discover"), params);
}

void CoreClient::runQuality(const QString& buildSystem)
{
    if (m_analyzing || m_process.state() != QProcess::Running) {
        return;
    }
    setAnalyzing(true);
    QJsonObject params;
    if (!buildSystem.trimmed().isEmpty()) {
        params.insert(QStringLiteral("buildSystem"), buildSystem);
    }
    sendRequest(QStringLiteral("quality.run"), params);
}

void CoreClient::runStart(const QString& command)
{
    QJsonObject params;
    if (!command.trimmed().isEmpty()) {
        params.insert(QStringLiteral("command"), command);
    }
    sendRequest(QStringLiteral("run.start"), params);
}

void CoreClient::runScript(const QString& path, const QString& device)
{
    QJsonObject params{{QStringLiteral("path"), path}};
    // A porta so' vai quando a tela a escolheu: num projeto MicroPython o
    // core roda o .py NA PLACA (`mpremote connect <porta> run`); sem porta o
    // mpremote usa a primeira que achar. Campo ausente nao e' campo vazio.
    if (!device.isEmpty()) {
        params.insert(QStringLiteral("device"), device);
    }
    sendRequest(QStringLiteral("run.script"), params);
}

void CoreClient::runStdin(const QString& data)
{
    sendRequest(QStringLiteral("run.stdin"), QJsonObject{{QStringLiteral("data"), data}});
}

void CoreClient::runStop()
{
    sendRequest(QStringLiteral("run.stop"), QJsonObject{});
}

} // namespace kinein
