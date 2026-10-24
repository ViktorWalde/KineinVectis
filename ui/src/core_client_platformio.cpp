// Dominio `platformio.*` no lado da UI (0.191.0, roadmaps/52 §14.45; o
// homeRelease desde a 0.192.0, §14.46): o PIO
// Home que a IDE exibe. Arquivo proprio pela regra da §5 da ARCHITECTURE: um
// dominio novo ganha arquivo novo.
//
// A URL da sessao passa por aqui e vai so' para a janela do PlatformIO: ela e'
// a chave do servidor local (a raiz responde 403), e o log nao a registra
// porque a resposta nao e' logada por campo.
#include "core_client.h"

namespace kinein {

void CoreClient::platformioHomeStart()
{
    sendRequest(QStringLiteral("platformio.homeStart"), QJsonObject{});
}

void CoreClient::platformioHomeStop()
{
    sendRequest(QStringLiteral("platformio.homeStop"), QJsonObject{});
}

// A pagina saiu da tela: o core desliga o servidor depois do prazo, salvo um
// homeStart ou homeStop antes (0.192.0, roadmaps/52 §14.46).
void CoreClient::platformioHomeRelease(int afterSeconds)
{
    sendRequest(QStringLiteral("platformio.homeRelease"),
                QJsonObject{{QStringLiteral("after"), afterSeconds}});
}

bool CoreClient::dispatchPlatformioResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("platformio.homeStart")) {
        emit platformioHomeStarted(result.value(QStringLiteral("url")).toString(),
                                   result.value(QStringLiteral("port")).toInt());
        return true;
    }
    if (method == QStringLiteral("platformio.homeStop")) {
        emit platformioHomeStopped(result.value(QStringLiteral("stopped")).toBool());
        return true;
    }
    if (method == QStringLiteral("platformio.homeRelease")) {
        emit platformioHomeReleased(result.value(QStringLiteral("releasing")).toBool());
        return true;
    }
    // A cadeia continua: grafana -> platformio -> probe -> ...
    return dispatchProbeResult(method, result);
}

} // namespace kinein
