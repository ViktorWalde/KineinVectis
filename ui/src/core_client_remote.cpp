// O ALVO LINUX POR SSH no lado da UI (P6 fatia 1 do roadmaps/42, 2026-09-17):
// o catalogo sem segredo (remote.list/save/remove), a sonda e o deploy como
// jobs (remote.probe/deploy) e a linha `ssh …` que vira configuracao de
// execucao ou kit (remote.command). Arquivo proprio pela regra "dominio novo,
// arquivo novo". A UI so' mostra e pede; senha nao passa por aqui.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::remoteList()
{
    sendRequest(QStringLiteral("remote.list"), QJsonObject{});
}

void CoreClient::remoteSave(const QVariantMap& target)
{
    // Campos vazios ficam AUSENTES: o contrato e' `deny_unknown_fields` e
    // `Option`; a normalizacao (porta 22, espacos) e' do core.
    QJsonObject alvo{};
    for (auto it = target.constBegin(); it != target.constEnd(); ++it) {
        const QVariant& valor = it.value();
        if (it.key() == QStringLiteral("port")) {
            const int porta = valor.toInt();
            if (porta > 0) {
                alvo.insert(it.key(), porta);
            }
            continue;
        }
        const QString texto = valor.toString().trimmed();
        if (!texto.isEmpty()) {
            alvo.insert(it.key(), texto);
        }
    }
    sendRequest(QStringLiteral("remote.save"), QJsonObject{{QStringLiteral("target"), alvo}});
}

void CoreClient::remoteRemove(const QString& name)
{
    sendRequest(QStringLiteral("remote.remove"), QJsonObject{{QStringLiteral("name"), name}});
}

void CoreClient::remoteProbe(const QString& name)
{
    sendRequest(QStringLiteral("remote.probe"), QJsonObject{{QStringLiteral("name"), name}});
}

void CoreClient::remoteDeploy(const QString& name, const QString& source, const QString& dest)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (!source.trimmed().isEmpty()) {
        params.insert(QStringLiteral("source"), source.trimmed());
    }
    if (!dest.trimmed().isEmpty()) {
        params.insert(QStringLiteral("dest"), dest.trimmed());
    }
    sendRequest(QStringLiteral("remote.deploy"), params);
}

void CoreClient::remoteCommand(const QString& name, const QString& kind, const QString& program,
                               int port)
{
    QJsonObject params{{QStringLiteral("name"), name}, {QStringLiteral("kind"), kind}};
    if (!program.trimmed().isEmpty()) {
        params.insert(QStringLiteral("program"), program.trimmed());
    }
    if (port > 0) {
        params.insert(QStringLiteral("port"), port);
    }
    sendRequest(QStringLiteral("remote.command"), params);
}

void CoreClient::toolchainSetKitRemote(const QString& remoteTarget, const QString& debugServer)
{
    // So' os dois campos do alvo remoto; o resto do kit fica como esta'
    // (campo ausente preserva). Vazio LIMPA, como nos outros campos do kit.
    sendRequest(QStringLiteral("toolchain.setKit"),
                QJsonObject{{QStringLiteral("remoteTarget"), remoteTarget},
                            {QStringLiteral("debugServer"), debugServer}});
}

bool CoreClient::dispatchRemoteResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("remote.list") || method == QStringLiteral("remote.save") ||
        method == QStringLiteral("remote.remove"))
    {
        emit remoteTargetsResolved(
            result.value(QStringLiteral("targets")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("remote.probe") || method == QStringLiteral("remote.deploy")) {
        // O job foi aceito; o desfecho chega por event.remote.probed/deployed.
        emit remoteJobAccepted(method, result.value(QStringLiteral("jobId")).toString(),
                               result.value(QStringLiteral("command")).toString());
        return true;
    }
    if (method == QStringLiteral("remote.command")) {
        emit remoteCommandResolved(result.toVariantMap());
        return true;
    }
    return false;
}

} // namespace kinein
