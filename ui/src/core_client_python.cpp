// O AMBIENTE Python do projeto no lado da UI (bloco B do roadmaps/41, fatia 1
// em 2026-09-12): o que o core resolveu (interpretador, ambiente proprio ou
// sistema, ferramenta que cria o .venv) e o pedido de criar. Arquivo proprio
// pela regra "dominio novo, arquivo novo". A UI so' mostra e pede; quem
// resolve e roda e' o core.
#include "core_client.h"

namespace kinein {

void CoreClient::pythonStatus()
{
    sendRequest(QStringLiteral("python.status"), QJsonObject{});
}

void CoreClient::pythonCreateEnvironment(const QString& tool)
{
    QJsonObject params;
    if (!tool.isEmpty()) {
        params.insert(QStringLiteral("tool"), tool);
    }
    sendRequest(QStringLiteral("python.createEnvironment"), params);
}

void CoreClient::pythonStubs(const QString& port, const QString& board)
{
    QJsonObject params;
    // Campo ausente = o modelo do projeto sugere (chip do kit/identidade).
    if (!port.isEmpty()) {
        params.insert(QStringLiteral("port"), port);
    }
    if (!board.isEmpty()) {
        params.insert(QStringLiteral("board"), board);
    }
    sendRequest(QStringLiteral("python.stubs"), params);
}

bool CoreClient::dispatchPythonResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("python.status")) {
        emit pythonStatusResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("python.createEnvironment")) {
        // O job foi aceito; o que aconteceu chega por event.python.finished.
        emit pythonEnvironmentAccepted(result.value(QStringLiteral("jobId")).toString());
        return true;
    }
    if (method == QStringLiteral("python.stubs")) {
        // Idem: o desfecho chega por event.python.stubs.
        emit pythonStubsAccepted(result.value(QStringLiteral("jobId")).toString(),
                                 result.value(QStringLiteral("package")).toString());
        return true;
    }
    return dispatchCoverageResult(method, result);
}

} // namespace kinein
