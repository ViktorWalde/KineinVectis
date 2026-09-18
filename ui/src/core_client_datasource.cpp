// Dominio de FONTES DE DADOS no lado da UI: pedidos e dispatch.
//
// Arquivo proprio pelo mesmo motivo do `core_client_library.cpp` e do
// `core_client_toolchain.cpp`: a §5 da ARCHITECTURE manda dividir o
// `CoreClient` por dominio. Dominio novo, arquivo novo.
//
// A SENHA PASSA POR AQUI, e e' o unico lugar da UI onde isso acontece. Ela
// viaja em `params` de `datasource.test`, nunca e' guardada e nunca chega ao
// log: `sendRequest` redige por NOME de campo antes de registrar
// (`core_client_process.cpp`). A decisao esta' em `DocsPublic/seguranca/40`.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::setupList()
{
    sendRequest(QStringLiteral("setup.list"), QJsonObject{});
}

void CoreClient::dataSourceList()
{
    sendRequest(QStringLiteral("datasource.list"), QJsonObject{});
}

void CoreClient::dataSourceSave(const QVariantMap& profile)
{
    sendRequest(QStringLiteral("datasource.save"),
                QJsonObject{{QStringLiteral("profile"), QJsonObject::fromVariantMap(profile)}});
}

void CoreClient::dataSourceRemove(const QString& name)
{
    sendRequest(QStringLiteral("datasource.remove"), QJsonObject{{QStringLiteral("name"), name}});
}

void CoreClient::dataSourceTest(const QString& name, const QString& password)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    // Campo ausente e campo vazio sao coisas diferentes para o core: ausente
    // significa "nao tenho senha", e e' o que faz o perfil `automatic` tentar
    // sem mandar nada.
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    sendRequest(QStringLiteral("datasource.test"), params);
}

void CoreClient::dataSourceIntrospect(const QString& name, const QString& password)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    sendRequest(QStringLiteral("datasource.introspect"), params);
}

void CoreClient::dataSourceQuery(const QString& name, const QString& password, const QString& sql,
                                 int maxRows, bool confirmWrite)
{
    QJsonObject params{{QStringLiteral("name"), name}, {QStringLiteral("sql"), sql}};
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    if (maxRows > 0) {
        params.insert(QStringLiteral("maxRows"), maxRows);
    }
    if (confirmWrite) {
        params.insert(QStringLiteral("confirmWrite"), true);
    }
    sendRequest(QStringLiteral("datasource.query"), params);
}

bool CoreClient::dispatchDataSourceResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("setup.list")) {
        emit setupListResolved(result.value(QStringLiteral("distroName")).toString(),
                               result.value(QStringLiteral("family")).toString(),
                               result.value(QStringLiteral("tools")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("datasource.list") ||
        method == QStringLiteral("datasource.save") ||
        method == QStringLiteral("datasource.remove"))
    {
        emit dataSourceListResolved(
            result.value(QStringLiteral("profiles")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("datasource.test") ||
        method == QStringLiteral("datasource.introspect") ||
        method == QStringLiteral("datasource.query"))
    {
        // O teste responde com o JOB; o veredito chega depois, por evento.
        emit dataSourceTestAccepted(result.value(QStringLiteral("jobId")).toString());
        return true;
    }
    return dispatchGrafanaResult(method, result);
}

} // namespace kinein
