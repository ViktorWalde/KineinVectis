// Dominio de embarcados no lado da UI: a sonda conectada.
//
// Arquivo proprio pela regra do `core_client_toolchain.cpp`: dominio novo,
// arquivo novo. O `probe.list` era roteado no core desde 2026-09-03 e nenhuma
// tela o pedia (roadmaps/40 §8.2); este arquivo e' o fio que faltava
// (roadmaps/35 §5.7, fatia 1).
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::probeList()
{
    sendRequest(QStringLiteral("probe.list"), QJsonObject{});
}

bool CoreClient::dispatchProbeResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("probe.list")) {
        return false;
    }
    // A saida CRUA viaja sempre, nao so' no erro: quando o parser nao
    // reconheceu nada, ela e' o que deixa o usuario ver se ha' uma sonda ali e
    // o formato mudou. "Nao entendi, e aqui esta' o que veio" e' acionavel;
    // "nenhuma sonda" com uma plugada e' mentira (kinein-protocol/probe.rs).
    emit probesResolved(result.value(QStringLiteral("probes")).toArray().toVariantList(),
                        result.value(QStringLiteral("toolAvailable")).toBool(),
                        result.value(QStringLiteral("rawOutput")).toString(),
                        result.value(QStringLiteral("hint")).toString());
    return true;
}

} // namespace kinein
