// Dominio de BIBLIOTECAS no lado da UI: pedidos e dispatch.
//
// Arquivo proprio pelo mesmo motivo do `core_client_toolchain.cpp` e do
// `core_client_configaction.cpp`: os dois lugares onde este codigo caberia
// naturalmente — `core_client_requests.cpp` e `core_client_dispatch.cpp` —
// estao na catraca, e a §5 da ARCHITECTURE ja manda dividir o `CoreClient` por
// dominio. Dominio novo, arquivo novo.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::libraryList()
{
    sendRequest(QStringLiteral("library.list"), QJsonObject{});
}

void CoreClient::libraryPlan(const QString& id, const QString& target)
{
    sendRequest(QStringLiteral("library.plan"),
                QJsonObject{{QStringLiteral("id"), id}, {QStringLiteral("target"), target}});
}

bool CoreClient::dispatchLibraryResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("library.list")) {
        emit libraryListResolved(
            result.value(QStringLiteral("libraries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("library.plan")) {
        emit libraryPlanResolved(result.toVariantMap());
        return true;
    }
    return false;
}

} // namespace kinein
