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

void CoreClient::runConfigFlashProposal(const QString& device, const QString& engine,
                                        double flashSizeBytes, const QString& firmware)
{
    QJsonObject params;
    // Campo ausente != vazio: sem porta o core diz "escolha a porta"; sem
    // motor ele usa o que o modelo sugere; sem tamanho, nao compara.
    if (!device.isEmpty()) {
        params.insert(QStringLiteral("device"), device);
    }
    if (!engine.isEmpty()) {
        params.insert(QStringLiteral("engine"), engine);
    }
    if (flashSizeBytes > 0) {
        params.insert(QStringLiteral("flashSizeBytes"), flashSizeBytes);
    }
    if (!firmware.isEmpty()) {
        params.insert(QStringLiteral("firmware"), firmware);
    }
    sendRequest(QStringLiteral("runConfig.flashProposal"), params);
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

void CoreClient::runStart(const QString& command, const QString& device)
{
    QJsonObject params;
    // A porta e' do LANCADOR PADRAO (`mpremote connect <porta> run main.py`):
    // so' vai sem comando explicito, porque o core recusa os dois juntos —
    // um comando digitado roda como foi escrito. Campo ausente nao e' vazio.
    if (!command.trimmed().isEmpty()) {
        params.insert(QStringLiteral("command"), command);
    }
    else if (!device.isEmpty()) {
        params.insert(QStringLiteral("device"), device);
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

void CoreClient::runStop()
{
    sendRequest(QStringLiteral("run.stop"), QJsonObject{});
}

bool CoreClient::dispatchRunResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("run.start") && method != QStringLiteral("run.script")) {
        return false;
    }
    // A execucao e' uma sessao de terminal (0.125.0): entra na mesma
    // contabilidade do `terminal.open`, senao input/resize/close nao a
    // reconhecem; `running` acompanha a sessao ate' o `event.terminal.closed`.
    const QString command = result.value(QStringLiteral("command")).toString();
    const QString terminalId = result.value(QStringLiteral("terminalId")).toString();
    if (!terminalId.isEmpty()) {
        m_terminalIds.insert(terminalId);
        setTerminalActive(true);
        m_runTerminalId = terminalId;
        setRunning(true);
    }
    appendLog(QStringLiteral("execucao iniciada (%1): %2").arg(terminalId, command));
    emit runStarted(command, terminalId);
    return true;
}

} // namespace kinein
