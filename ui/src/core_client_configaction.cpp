// Dominio das Configuration Actions no lado da UI: pedidos e dispatch.
//
// POR QUE ESTE ARQUIVO EXISTE. A `ARCHITECTURE.md` §5 manda dividir o
// `CoreClient` por dominio, e os dois arquivos onde este codigo caberia
// naturalmente — `core_client_requests.cpp` (660 linhas) e
// `core_client_dispatch.cpp` (751) — estao na catraca. Fazer um deles crescer
// seria pagar a divida de outro com a fatia de hoje; espremer linha ou subir o
// baseline sao as duas trapacas que a §4 regra 9 nomeia. A terceira saida e a
// que o documento ja mandava, e e a mesma de 2026-08-30: dominio novo, arquivo
// proprio.
//
// Pedido e dispatch moram juntos AQUI (e nao em dois arquivos) porque o ciclo
// list -> preview -> apply e um so contrato: separar o que se manda do que se
// recebe espalharia a mesma conversa por dois lugares que envelheceriam
// separados. O dominio e pequeno; quando crescer, vira pasta.
#include "core_client.h"

#include <QJsonArray>
#include <QJsonValue>

namespace kinein {
namespace {

/// Converte o mapa de parametros da UI em objeto JSON de strings.
///
/// O protocolo declara `params` como mapa de STRING para STRING: os valores
/// vem de campos de texto e o core os valida por acao. Converter aqui evita
/// que um `int` do QML vire numero JSON e o `deny_unknown_fields` do core
/// recuse a requisicao inteira por causa do tipo.
QJsonObject stringParams(const QVariantMap& params)
{
    QJsonObject object;
    for (auto it = params.constBegin(); it != params.constEnd(); ++it) {
        object.insert(it.key(), it.value().toString());
    }
    return object;
}

/// Reconstroi a lista `expected` (o snapshot que o preview mostrou).
QJsonArray expectedFiles(const QVariantList& expected)
{
    QJsonArray array;
    for (const QVariant& entry : expected) {
        const QVariantMap file = entry.toMap();
        QJsonObject object{{QStringLiteral("path"), file.value(QStringLiteral("path")).toString()}};
        // `content` ausente significa "o arquivo nao existia no preview"; e
        // uma afirmacao diferente de "existia vazio", e o core trata as duas
        // de forma diferente.
        if (file.contains(QStringLiteral("content"))) {
            object.insert(QStringLiteral("content"),
                          file.value(QStringLiteral("content")).toString());
        }
        array.append(object);
    }
    return array;
}

QStringList stringsOf(const QJsonArray& array)
{
    QStringList values;
    values.reserve(array.size());
    for (const QJsonValue value : array) {
        values.append(value.toString());
    }
    return values;
}

} // namespace

void CoreClient::configActionList(bool includeHiddenByScope)
{
    sendRequest(QStringLiteral("configAction.list"),
                QJsonObject{{QStringLiteral("includeHiddenByScope"), includeHiddenByScope}});
}

void CoreClient::configActionPreview(const QString& id, const QVariantMap& params)
{
    sendRequest(
        QStringLiteral("configAction.preview"),
        QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("params"), stringParams(params)}});
}

void CoreClient::configActionApply(const QString& id, const QVariantMap& params,
                                   const QVariantList& expected)
{
    sendRequest(QStringLiteral("configAction.apply"),
                QJsonObject{{QStringLiteral("id"), id},
                            {QStringLiteral("params"), stringParams(params)},
                            {QStringLiteral("expected"), expectedFiles(expected)}});
}

bool CoreClient::dispatchConfigActionResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("configAction.list")) {
        emit configActionsListed(
            result.value(QStringLiteral("actions")).toArray().toVariantList(),
            stringsOf(result.value(QStringLiteral("activeBuildSystems")).toArray()));
        return true;
    }
    if (method == QStringLiteral("configAction.preview")) {
        emit configActionPreviewed(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("configAction.apply")) {
        const QString jobId = result.value(QStringLiteral("jobId")).toString();
        appendLog(QStringLiteral("configuration action aplicada (%1): %2")
                      .arg(result.value(QStringLiteral("id")).toString(),
                           result.value(QStringLiteral("message")).toString()));
        emit configActionApplied(result.value(QStringLiteral("id")).toString(),
                                 result.value(QStringLiteral("message")).toString(),
                                 stringsOf(result.value(QStringLiteral("files")).toArray()), jobId);
        return true;
    }
    return false;
}

} // namespace kinein
