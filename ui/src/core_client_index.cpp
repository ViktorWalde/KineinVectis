// O indice do projeto INTEIRO no lado da UI (pilar 0 do roadmaps/42, decisao
// do autor em 2026-09-12): totais e busca por nome. Arquivo proprio pela regra
// "dominio novo, arquivo novo". A UI so' mostra e pergunta; quem le o projeto
// e' o core.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::indexStatus()
{
    sendRequest(QStringLiteral("index.status"), QJsonObject{});
}

void CoreClient::indexSymbols(const QString& query, int limit)
{
    QJsonObject params{{QStringLiteral("query"), query}};
    if (limit > 0) {
        params.insert(QStringLiteral("limit"), limit);
    }
    sendRequest(QStringLiteral("index.symbols"), params);
}

void CoreClient::indexContext(const QString& path)
{
    sendRequest(QStringLiteral("index.context"), QJsonObject{{QStringLiteral("path"), path}});
}

bool CoreClient::dispatchIndexResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("index.status")) {
        emit indexStatusResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("index.context")) {
        emit indexContextResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("index.symbols")) {
        emit indexSymbolsResolved(result.value(QStringLiteral("symbols")).toArray().toVariantList(),
                                  result.value(QStringLiteral("total")).toInt(),
                                  result.value(QStringLiteral("state")).toString());
        return true;
    }
    return dispatchPythonResult(method, result);
}

} // namespace kinein
