// Dominio de OBSERVABILIDADE no lado da UI: pedidos, respostas e o evento.
//
// Arquivo proprio pela mesma regra do `core_client_datasource.cpp`: a §5 da
// ARCHITECTURE manda dividir o `CoreClient` por dominio, e dominio novo ganha
// arquivo novo em vez de engordar o vizinho.
//
// O TOKEN PASSA POR AQUI, e e' o unico lugar da UI onde isso acontece. Ele
// viaja em `params` de `grafana.probe`, nunca e' guardado e nunca chega ao
// log: `sendRequest` redige por NOME de campo antes de registrar
// (`core_client_process.cpp`), e `token` esta' na lista.
#include "core_client.h"

namespace kinein {

void CoreClient::grafanaGet()
{
    sendRequest(QStringLiteral("grafana.get"), QJsonObject{});
}

void CoreClient::grafanaSave(const QVariantMap& profile)
{
    sendRequest(QStringLiteral("grafana.save"),
                QJsonObject{{QStringLiteral("profile"), QJsonObject::fromVariantMap(profile)}});
}

void CoreClient::grafanaForget()
{
    sendRequest(QStringLiteral("grafana.forget"), QJsonObject{});
}

void CoreClient::grafanaProbe(const QString& token)
{
    QJsonObject params;
    // Campo ausente e campo vazio sao coisas diferentes para o core: ausente
    // significa "nao tenho token", e e' o que faz a politica `none` sondar so'
    // a saude publica em vez de ser recusada.
    if (!token.isEmpty()) {
        params.insert(QStringLiteral("token"), token);
    }
    sendRequest(QStringLiteral("grafana.probe"), params);
}

bool CoreClient::dispatchGrafanaResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("grafana.get") || method == QStringLiteral("grafana.save") ||
        method == QStringLiteral("grafana.forget"))
    {
        // A AUSENCIA E' UM ESTADO, e ela precisa chegar como tal. Um mapa vazio
        // sozinho nao distingue "nao ha' Grafana neste workspace" de "havia e a
        // resposta veio truncada"; o booleano decide, e a tela desenha um
        // convite em vez de um formulario em branco.
        const QJsonValue profile = result.value(QStringLiteral("profile"));
        emit grafanaProfileResolved(profile.toObject().toVariantMap(), profile.isObject());
        return true;
    }
    if (method == QStringLiteral("grafana.probe")) {
        // A sonda responde com o JOB; o que ela achou chega depois, por evento.
        emit grafanaProbeAccepted(result.value(QStringLiteral("jobId")).toString());
        return true;
    }
    return false;
}

} // namespace kinein
