// Dominio da toolchain no lado da UI: pedidos e dispatch.
//
// Arquivo proprio pelo mesmo motivo do `core_client_configaction.cpp`: os dois
// lugares onde este codigo caberia naturalmente — `core_client_requests.cpp` e
// `core_client_dispatch.cpp` — estao na catraca, e a §5 da ARCHITECTURE ja
// manda dividir o `CoreClient` por dominio. Dominio novo, arquivo novo.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::toolchainGet()
{
    sendRequest(QStringLiteral("toolchain.get"), QJsonObject{});
}

void CoreClient::toolchainSet(const QString& role, const QString& id)
{
    QJsonObject params{{QStringLiteral("role"), role}};
    // `id` vazio significa AUTOMATICO, e o contrato pede o campo AUSENTE — nao
    // uma string vazia, que o core recusaria como candidato desconhecido.
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("toolchain.set"), params);
}

bool CoreClient::dispatchToolchainResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("toolchain.get") && method != QStringLiteral("toolchain.set")) {
        return false;
    }
    // Os dois metodos respondem o MESMO shape, entao ha um sinal so: a UI nunca
    // precisa casar resposta com pedido para saber o que mudou.
    emit toolchainResolved(result.value(QStringLiteral("selections")).toArray().toVariantList(),
                           result.value(QStringLiteral("candidates")).toArray().toVariantList());
    return true;
}

} // namespace kinein
