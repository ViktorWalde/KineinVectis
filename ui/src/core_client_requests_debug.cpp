// Dominio de DEBUG no lado da UI: os pedidos `debug.*`.
//
// Arquivo proprio pelo mesmo motivo do `core_client_toolchain.cpp` e do
// `core_client_configaction.cpp`: os dois lugares onde este codigo caberia
// naturalmente — `core_client_requests.cpp` e `core_client_dispatch.cpp` —
// estao na catraca, e a §5 da ARCHITECTURE ja manda dividir o `CoreClient` por
// dominio. O gatilho foi o `debug.evaluate` da etapa 15 do roadmaps/34, que
// fez o requests.cpp crescer de 660 para 672 estando em debito.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::debugStart(const QString& program, const QVariantMap& connect)
{
    QJsonObject params;
    if (!program.isEmpty()) {
        params.insert(QStringLiteral("program"), program);
    }
    if (!connect.isEmpty()) {
        params.insert(QStringLiteral("connect"), QJsonObject::fromVariantMap(connect));
    }
    sendRequest(QStringLiteral("debug.start"), params);
}

void CoreClient::debugSetBreakpoints(const QString& file, const QVariantList& breakpoints)
{
    sendRequest(
        QStringLiteral("debug.setBreakpoints"),
        QJsonObject{{QStringLiteral("file"), file},
                    {QStringLiteral("breakpoints"), QJsonArray::fromVariantList(breakpoints)}});
}

void CoreClient::debugEvaluate(const QString& expression, double frameId)
{
    QJsonObject params{{QStringLiteral("expression"), expression}};
    // frameId negativo = "use o frame do topo": o core resolve, a UI nao
    // precisa saber qual frame esta selecionado para pedir um watch.
    if (frameId >= 0) {
        params.insert(QStringLiteral("frameId"), frameId);
    }
    sendRequest(QStringLiteral("debug.evaluate"), params);
}

void CoreClient::debugContinue()
{
    sendRequest(QStringLiteral("debug.continue"), QJsonObject{});
}

void CoreClient::debugNext()
{
    sendRequest(QStringLiteral("debug.next"), QJsonObject{});
}

void CoreClient::debugStepIn()
{
    sendRequest(QStringLiteral("debug.stepIn"), QJsonObject{});
}

void CoreClient::debugStepOut()
{
    sendRequest(QStringLiteral("debug.stepOut"), QJsonObject{});
}

void CoreClient::debugPause()
{
    sendRequest(QStringLiteral("debug.pause"), QJsonObject{});
}

void CoreClient::debugStop()
{
    sendRequest(QStringLiteral("debug.stop"), QJsonObject{});
}

void CoreClient::debugStackTrace()
{
    sendRequest(QStringLiteral("debug.stackTrace"), QJsonObject{});
}

void CoreClient::debugVariablesForFrame(double frameId)
{
    sendRequest(QStringLiteral("debug.variables"),
                QJsonObject{{QStringLiteral("frameId"), static_cast<qint64>(frameId)}});
}

void CoreClient::debugVariablesForRef(double ref)
{
    sendRequest(QStringLiteral("debug.variables"),
                QJsonObject{{QStringLiteral("ref"), static_cast<qint64>(ref)}});
}

void CoreClient::debugScopes(double frameId)
{
    sendRequest(QStringLiteral("debug.scopes"),
                QJsonObject{{QStringLiteral("frameId"), static_cast<qint64>(frameId)}});
}

void CoreClient::debugReadMemory(const QString& memoryReference, double count, double offset)
{
    QJsonObject params{{QStringLiteral("memoryReference"), memoryReference},
                       {QStringLiteral("count"), static_cast<qint64>(count)}};
    // Campo ausente = zero para o DAP; so' vai quando ha' deslocamento.
    if (static_cast<qint64>(offset) != 0) {
        params.insert(QStringLiteral("offset"), static_cast<qint64>(offset));
    }
    sendRequest(QStringLiteral("debug.readMemory"), params);
}

void CoreClient::debugDisassemble(const QString& memoryReference, double instructionCount,
                                  double instructionOffset)
{
    QJsonObject params{{QStringLiteral("memoryReference"), memoryReference},
                       {QStringLiteral("instructionCount"), static_cast<qint64>(instructionCount)}};
    if (static_cast<qint64>(instructionOffset) != 0) {
        params.insert(QStringLiteral("instructionOffset"), static_cast<qint64>(instructionOffset));
    }
    sendRequest(QStringLiteral("debug.disassemble"), params);
}

} // namespace kinein
