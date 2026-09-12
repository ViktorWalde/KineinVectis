// Dominio de containers no lado da UI: Docker e Podman como dominio NATIVO
// (roadmaps/28 §0, priorizado em 2026-09-12).
//
// Arquivo proprio pela regra "dominio novo, arquivo novo". A invariante do 28
// §4 vale aqui: a UI NUNCA chama `docker`; ela pede ao core e mostra. Logs e
// shell chegam como uma sessao de TERMINAL (o mesmo `open_command` do dominio
// terminal), e por isso este arquivo faz a mesma contabilidade do
// `terminal.open` antes de anunciar a aba.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::containerStatus()
{
    sendRequest(QStringLiteral("container.status"), QJsonObject{});
}

void CoreClient::containerList(bool all)
{
    sendRequest(QStringLiteral("container.list"), QJsonObject{{QStringLiteral("all"), all}});
}

void CoreClient::containerImages()
{
    sendRequest(QStringLiteral("container.images"), QJsonObject{});
}

void CoreClient::containerAction(const QString& id, const QString& action)
{
    sendRequest(QStringLiteral("container.action"),
                QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("action"), action}});
}

void CoreClient::containerOpen(const QString& id, const QString& mode)
{
    sendRequest(QStringLiteral("container.open"),
                QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("mode"), mode}});
}

void CoreClient::containerCompose(const QString& action, const QString& file)
{
    QJsonObject params{{QStringLiteral("action"), action}};
    // Ausente, o compose usa o arquivo padrao do workspace; vazio seria pedir
    // um arquivo chamado "".
    if (!file.isEmpty()) {
        params.insert(QStringLiteral("file"), file);
    }
    sendRequest(QStringLiteral("container.compose"), params);
}

bool CoreClient::dispatchContainerResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("container.status")) {
        emit containerStatusResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("container.list")) {
        emit containersResolved(
            result.value(QStringLiteral("containers")).toArray().toVariantList(),
            result.value(QStringLiteral("engine")).toString(),
            result.value(QStringLiteral("rawOutput")).toString(),
            result.value(QStringLiteral("hint")).toString());
        return true;
    }
    if (method == QStringLiteral("container.images")) {
        emit containerImagesResolved(
            result.value(QStringLiteral("images")).toArray().toVariantList(),
            result.value(QStringLiteral("hint")).toString());
        return true;
    }
    if (method == QStringLiteral("container.action") ||
        method == QStringLiteral("container.compose"))
    {
        emit containerActionAccepted(result.value(QStringLiteral("jobId")).toString());
        return true;
    }
    if (method == QStringLiteral("container.open")) {
        // E' uma sessao de terminal como outra qualquer: entra na mesma
        // contabilidade do `terminal.open`, senao `terminal.input`/`close`
        // nao a reconhecem como viva.
        const QString id = result.value(QStringLiteral("id")).toString();
        const QString command = result.value(QStringLiteral("command")).toString();
        m_terminalIds.insert(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit containerTerminalOpened(id, command);
        appendLog(QStringLiteral("terminal de container aberto (%1): %2").arg(id, command));
        return true;
    }
    return dispatchIndexResult(method, result);
}

} // namespace kinein
