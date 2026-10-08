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
namespace {
void appendOperationContext(QJsonObject& params, const QVariantMap& context)
{
    if (context.isEmpty()) {
        return;
    }
    params.insert(QStringLiteral("clientContext"),
                  QJsonValue::fromVariant(context.value(QStringLiteral("clientContext"))));
    params.insert(QStringLiteral("expectedContext"),
                  QJsonValue::fromVariant(context.value(QStringLiteral("expectedContext"))));
}
} // namespace

void CoreClient::setupList()
{
    sendRequest(QStringLiteral("setup.list"), QJsonObject{});
}

void CoreClient::dataSourceList()
{
    sendRequest(QStringLiteral("datasource.list"), QJsonObject{});
}

void CoreClient::dataSourceOdbcSources()
{
    sendRequest(QStringLiteral("datasource.odbc.sources"), QJsonObject{});
}

void CoreClient::dataSourceOdbcAuthorize(const QString& name, const QString& identity,
                                         const QString& workspace)
{
    sendRequest(QStringLiteral("datasource.odbc.authorize"),
                QJsonObject{{QStringLiteral("name"), name},
                            {QStringLiteral("identity"), identity},
                            {QStringLiteral("workspace"), workspace}});
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

void CoreClient::dataSourceConsole(const QString& name, const QVariantMap& context)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    appendOperationContext(params, context);
    sendRequest(QStringLiteral("datasource.console"), params);
}

void CoreClient::dataSourceConsoleStatement(const QVariantMap& operation)
{
    sendRequest(QStringLiteral("datasource.console.statement"),
                QJsonObject::fromVariantMap(operation));
}

void CoreClient::dataSourceTest(const QString& name, const QString& password,
                                const QVariantMap& context)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    // Campo ausente e campo vazio sao coisas diferentes para o core: ausente
    // significa "nao tenho senha", e e' o que faz o perfil `automatic` tentar
    // sem mandar nada.
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    appendOperationContext(params, context);
    sendRequest(QStringLiteral("datasource.test"), params);
}

void CoreClient::dataSourceIntrospect(const QString& name, const QString& password,
                                      const QVariantMap& context)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    appendOperationContext(params, context);
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

void CoreClient::dataSourceDestroy(const QString& name, bool data, const QString& password,
                                   const QVariantMap& context, const QVariantMap& confirmation)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (data) {
        params.insert(QStringLiteral("data"), true);
    }
    appendOperationContext(params, context);
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    if (!confirmation.isEmpty()) {
        params.insert(QStringLiteral("confirmation"), QJsonObject::fromVariantMap(confirmation));
    }
    sendRequest(QStringLiteral("datasource.destroy"), params);
}

void CoreClient::dataSourceQuery(const QString& name, const QString& password, const QString& sql,
                                 int maxRows, bool confirmWrite, const QVariantMap& context,
                                 const QVariantMap& confirmation)
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
    if (context.value(QStringLiteral("preview")).toBool()) {
        params.insert(QStringLiteral("preview"), true);
    }
    appendOperationContext(params, context);
    if (!confirmation.isEmpty()) {
        params.insert(QStringLiteral("confirmation"), QJsonObject::fromVariantMap(confirmation));
    }
    sendRequest(QStringLiteral("datasource.query"), params);
}

void CoreClient::dataSourcePreviewDecide(const QVariantMap& operation)
{
    sendRequest(QStringLiteral("datasource.preview.decide"),
                QJsonObject::fromVariantMap(operation));
}

void CoreClient::dataSourceDisconnect(const QString& name, const QVariantMap& context)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    appendOperationContext(params, context);
    sendRequest(QStringLiteral("datasource.disconnect"), params);
}

void CoreClient::dataSourceHistory(const QString& name)
{
    sendRequest(QStringLiteral("datasource.history"), {{QStringLiteral("name"), name}});
}

void CoreClient::dataSourceHistoryClear(const QString& name)
{
    sendRequest(QStringLiteral("datasource.history.clear"), {{QStringLiteral("name"), name}});
}

void CoreClient::exportDirectory(const QString& path)
{
    sendExport(QStringLiteral("fs.createDirectory"), {{QStringLiteral("path"), path}});
}

void CoreClient::exportFile(const QString& path, const QString& content)
{
    sendExport(QStringLiteral("fs.createFile"),
               {{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

// O pedido marcado pelo id: a resposta volta por `exportSucceeded`/
// `exportFailed`, e o core parado responde na hora, para a tela nao esperar.
void CoreClient::sendExport(const QString& method, const QJsonObject& params)
{
    const qint64 id = sendRequest(method, params, true);
    if (id < 0) {
        emit exportFailed(method, params.value(QStringLiteral("path")).toString(),
                          QStringLiteral("o core nao esta rodando"));
        return;
    }
    m_pendingExports.insert(id);
}

void CoreClient::dataSourceImpact(const QString& name, const QString& password, const QString& sql,
                                  const QVariantMap& context)
{
    QJsonObject params{{QStringLiteral("name"), name}, {QStringLiteral("sql"), sql}};
    if (!password.isEmpty()) {
        params.insert(QStringLiteral("password"), password);
    }
    appendOperationContext(params, context);
    sendRequest(QStringLiteral("datasource.impact"), params);
}

// O historico de consultas (0.166.0) num elo proprio da cadeia de despacho.
bool CoreClient::dispatchDataSourceHistoryResult(const QString& method, const QJsonObject& result)
{
    const QString name = result.value(QStringLiteral("name")).toString();
    if (method == QStringLiteral("datasource.history")) {
        emit dataSourceHistoryListed(
            name, result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("datasource.history.clear")) {
        emit dataSourceHistoryCleared(name);
        return true;
    }
    return false;
}

bool CoreClient::dispatchDataSourceResult(const QString& method, const QJsonObject& result)
{
    if (dispatchDataSourceHistoryResult(method, result)) {
        return true;
    }
    if (method == QStringLiteral("datasource.odbc.sources")) {
        emit dataSourceOdbcSourcesResolved(
            result.value(QStringLiteral("sources")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("datasource.odbc.authorize")) {
        emit dataSourceOdbcAuthorized(result.value(QStringLiteral("name")).toString(),
                                      result.value(QStringLiteral("identity")).toString(),
                                      result.value(QStringLiteral("workspace")).toString());
        return true;
    }
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
            result.value(QStringLiteral("profiles")).toArray().toVariantList(),
            result.value(QStringLiteral("consoleBindings")).toArray().toVariantList(),
            result.value(QStringLiteral("workspace")).toString(),
            result.value(QStringLiteral("providers")).toArray().toVariantList(),
            result.value(QStringLiteral("unavailable")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("datasource.console")) {
        emit dataSourceConsoleResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("datasource.console.statement")) {
        emit dataSourceConsoleStatementResolved(result.toVariantMap());
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
            result.value(QStringLiteral("note")).toString(),
            result.value(QStringLiteral("clientContext")).toString());
        return true;
    }
    if (method == QStringLiteral("datasource.create")) {
        emit dataSourceCreateResolved(
            result.value(QStringLiteral("profile")).toObject().toVariantMap(),
            result.value(QStringLiteral("jobId")).toString(),
            result.value(QStringLiteral("command")).toString());
        return true;
    }
    if (method == QStringLiteral("datasource.query")) {
        emit dataSourceQueryAccepted(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("datasource.preview.decide")) {
        return true;
    }
    if (method == QStringLiteral("datasource.disconnect")) {
        emit dataSourceDisconnectAccepted(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("datasource.test") ||
        method == QStringLiteral("datasource.introspect"))
    {
        // O teste responde com o JOB; o veredito chega depois, por evento.
        emit dataSourceTestAccepted(result.value(QStringLiteral("jobId")).toString());
        return true;
    }
    if (method == QStringLiteral("datasource.impact")) {
        // So' o aceite do job: o impacto chega em event.datasource.impact.
        return true;
    }
    return dispatchGrafanaResult(method, result);
}

void CoreClient::handleDataSourceFailure(const QString& method, const QJsonObject& error,
                                         const QVariantMap& requestQuery)
{
    if (!requestQuery.isEmpty()) {
        emit dataSourceOperationFailed(method, error.value(QStringLiteral("message")).toString(),
                                       error.value(QStringLiteral("code")).toString(),
                                       requestQuery);
    }
    if (error.value(QStringLiteral("code")).toString() !=
        QStringLiteral("DRIVER_APPROVAL_REQUIRED"))
    {
        return;
    }
    QVariantMap details = error.value(QStringLiteral("details")).toObject().toVariantMap();
    if (!requestQuery.isEmpty()) {
        details.insert(QStringLiteral("query"), requestQuery);
    }
    emit dataSourceDriverRequired(method, details);
}

} // namespace kinein
