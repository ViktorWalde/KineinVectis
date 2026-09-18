// A COBERTURA dos testes no lado da UI (D8 do roadmaps/41, 2026-09-17): o
// job que escreve o LCOV (coverage.run) e as linhas de um arquivo do ultimo
// relatorio (coverage.lines), que a calha do editor pinta. Arquivo proprio
// pela regra "dominio novo, arquivo novo". A UI so' mostra e pede.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::coverageRun()
{
    sendRequest(QStringLiteral("coverage.run"), QJsonObject{});
}

void CoreClient::coverageLines(const QString& file)
{
    sendRequest(QStringLiteral("coverage.lines"), QJsonObject{{QStringLiteral("file"), file}});
}

bool CoreClient::dispatchCoverageResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("coverage.lines")) {
        emit coverageLinesResolved(
            result.value(QStringLiteral("file")).toString(),
            result.value(QStringLiteral("known")).toBool(false),
            result.value(QStringLiteral("covered")).toArray().toVariantList(),
            result.value(QStringLiteral("missed")).toArray().toVariantList());
        return true;
    }
    // coverage.run: so' o jobId; o desfecho chega por event.coverage.finished.
    return false;
}

} // namespace kinein
