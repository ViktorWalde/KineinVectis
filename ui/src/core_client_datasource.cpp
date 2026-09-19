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

void CoreClient::dataSourceDiscover()
{
    sendRequest(QStringLiteral("datasource.discover"), QJsonObject{});
}

void CoreClient::dataSourceCreateSqlite(const QString& name, const QString& path)
{
    QJsonObject params{{QStringLiteral("kind"), QStringLiteral("sqliteFile")},
                       {QStringLiteral("name"), name}};
    if (!path.trimmed().isEmpty()) {
        params.insert(QStringLiteral("path"), path.trimmed());
    }
    sendRequest(QStringLiteral("datasource.create"), params);
}

void CoreClient::dataSourceCreateServer(const QString& engine, const QString& name, int port)
{
    sendRequest(QStringLiteral("datasource.create"),
                QJsonObject{{QStringLiteral("kind"), QStringLiteral("containerServer")},
                            {QStringLiteral("engine"), engine},
                            {QStringLiteral("name"), name},
                            {QStringLiteral("port"), port}});
}

void CoreClient::dataSourceDestroy(const QString& name, bool data)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (data) {
        params.insert(QStringLiteral("data"), true);
    }
    sendRequest(QStringLiteral("datasource.destroy"), params);
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
    if (method == QStringLiteral("datasource.discover")) {
        emit dataSourceDiscovered(
            result.value(QStringLiteral("candidates")).toArray().toVariantList(),
            result.value(QStringLiteral("containerEngine")).toString(),
            result.value(QStringLiteral("hint")).toString());
        return true;
    }
    if (method == QStringLiteral("datasource.destroy")) {
        const bool immediate = result.contains(QStringLiteral("profiles"));
        emit dataSourceDestroyResolved(
            result.value(QStringLiteral("profiles")).toArray().toVariantList(), immediate,
            result.value(QStringLiteral("jobId")).toString(),
            result.value(QStringLiteral("command")).toString(),
            result.value(QStringLiteral("note")).toString());
        return true;
    }
    if (method == QStringLiteral("datasource.create")) {
        emit dataSourceCreateResolved(
            result.value(QStringLiteral("profile")).toObject().toVariantMap(),
            result.value(QStringLiteral("jobId")).toString(),
            result.value(QStringLiteral("command")).toString());
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
