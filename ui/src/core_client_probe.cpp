// Dominio de embarcados no lado da UI: a sonda conectada, o tamanho do ELF e
// as portas seriais USB.
//
// Arquivo proprio pela regra do `core_client_toolchain.cpp`: dominio novo,
// arquivo novo. O `probe.list` era roteado no core desde 2026-09-03 e nenhuma
// tela o pedia (roadmaps/40 §8.2); este arquivo e' o fio que faltava
// (roadmaps/35 §5.7, fatia 1). O `serial.list` (E1 do integracoes/38 §6)
// nasceu ja' com consumidor, para nao repetir aquele buraco.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::probeList()
{
    sendRequest(QStringLiteral("probe.list"), QJsonObject{});
}

void CoreClient::serialList()
{
    sendRequest(QStringLiteral("serial.list"), QJsonObject{});
}

void CoreClient::buildSize(const QString& program)
{
    QJsonObject params{};
    // Campo ausente: o core resolve o ELF como o debug.start. Vazio seria
    // pedir para medir um caminho em branco.
    if (!program.isEmpty()) {
        params.insert(QStringLiteral("program"), program);
    }
    sendRequest(QStringLiteral("build.size"), params);
}

bool CoreClient::dispatchProbeResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("probe.list")) {
        return dispatchBuildSizeResult(method, result);
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

bool CoreClient::dispatchBuildSizeResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("build.size")) {
        return dispatchSerialResult(method, result);
    }
    emit buildSizeResolved(result.value(QStringLiteral("sections")).toArray().toVariantList(),
                           result.value(QStringLiteral("regions")).toArray().toVariantList(),
                           result.value(QStringLiteral("toolAvailable")).toBool(),
                           result.value(QStringLiteral("tool")).toString(),
                           result.value(QStringLiteral("rawOutput")).toString());
    return true;
}

bool CoreClient::dispatchSerialResult(const QString& method, const QJsonObject& result)
{
    if (method != QStringLiteral("serial.list")) {
        return false;
    }
    // O core nunca ABRIU a porta para responder isto (abrir reseta a placa);
    // cada entrada ja' traz o veredito de permissao MEDIDO e o estado do
    // ModemManager. A UI so' mostra.
    emit serialPortsResolved(result.value(QStringLiteral("ports")).toArray().toVariantList(),
                             result.value(QStringLiteral("hint")).toString());
    return true;
}

} // namespace kinein
