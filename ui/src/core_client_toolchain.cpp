// Dominio da toolchain no lado da UI: pedidos e dispatch.
//
// Arquivo proprio pelo mesmo motivo do `core_client_configaction.cpp`: os dois
// lugares onde este codigo caberia naturalmente — `core_client_requests.cpp` e
// `core_client_dispatch.cpp` — estao na catraca, e a §5 da ARCHITECTURE ja
// manda dividir o `CoreClient` por dominio. Dominio novo, arquivo novo.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::toolchainGet(const QString& preset)
{
    // O kit que a tela escolheu e' o que o configure automatico deve usar:
    // guardado aqui, na ponte, para os cinco chamadores de cmakeConfigure()
    // nao terem de saber de kit (P0 do 40 §4.1, 2026-09-17).
    m_kitPreset = preset;
    QJsonObject params{};
    if (!preset.isEmpty()) {
        params.insert(QStringLiteral("preset"), preset);
    }
    sendRequest(QStringLiteral("toolchain.get"), params);
}

void CoreClient::toolchainInstallable()
{
    sendRequest(QStringLiteral("toolchain.installable"), QJsonObject{});
}

void CoreClient::toolchainInstall(const QString& id)
{
    // O clique E' o consentimento (integracoes/39 §5): a tela ja' mostrou a
    // URL, o tamanho e o sha256 antes; o core baixa como job cancelavel.
    sendRequest(QStringLiteral("toolchain.install"), QJsonObject{{QStringLiteral("id"), id}});
}

void CoreClient::toolchainInspectSysroot(const QString& path)
{
    sendRequest(QStringLiteral("toolchain.inspectSysroot"),
                QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::toolchainImportKit(const QString& path)
{
    sendRequest(QStringLiteral("toolchain.importKit"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::toolchainSetKit(const QString& preset, const QString& sysroot,
                                 const QString& targetTriple, const QString& chip,
                                 const QString& toolchainFile, const QString& svdFile)
{
    QJsonObject params{};
    if (!preset.isEmpty()) {
        params.insert(QStringLiteral("preset"), preset);
    }
    // Campo AUSENTE preserva o valor atual; string vazia LIMPA. E por isso que
    // um QString nulo e um QString vazio significam coisas diferentes aqui.
    if (!sysroot.isNull()) {
        params.insert(QStringLiteral("sysroot"), sysroot);
    }
    if (!targetTriple.isNull()) {
        params.insert(QStringLiteral("targetTriple"), targetTriple);
    }
    // O chip existia no protocolo desde 2026-09-03 e a ponte o omitia: so' a
    // CLI conseguia gravar um. Fio ligado em 2026-09-11 (roadmaps/35 §5.7).
    if (!chip.isNull()) {
        params.insert(QStringLiteral("chip"), chip);
    }
    // O arquivo de toolchain do kit (0.104.0): a mesma regra — nulo preserva,
    // vazio limpa.
    if (!toolchainFile.isNull()) {
        params.insert(QStringLiteral("toolchainFile"), toolchainFile);
    }
    // O SVD do chip (0.118.0): idem.
    if (!svdFile.isNull()) {
        params.insert(QStringLiteral("svdFile"), svdFile);
    }
    sendRequest(QStringLiteral("toolchain.setKit"), params);
}

void CoreClient::toolchainSet(const QString& role, const QString& id, const QString& preset)
{
    QJsonObject params{{QStringLiteral("role"), role}};
    if (!preset.isEmpty()) {
        params.insert(QStringLiteral("preset"), preset);
    }
    // `id` vazio significa AUTOMATICO, e o contrato pede o campo AUSENTE — nao
    // uma string vazia, que o core recusaria como candidato desconhecido.
    if (!id.isEmpty()) {
        params.insert(QStringLiteral("id"), id);
    }
    sendRequest(QStringLiteral("toolchain.set"), params);
}

bool CoreClient::dispatchToolchainResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("toolchain.installable")) {
        emit toolchainInstallableResolved(
            result.value(QStringLiteral("toolchains")).toArray().toVariantList(),
            result.value(QStringLiteral("installRoot")).toString(),
            result.value(QStringLiteral("projectFamily")).toString());
        return true;
    }
    if (method == QStringLiteral("toolchain.install")) {
        // A resposta e' so' o jobId: o andamento chega pelos event.job.* e o
        // desfecho por event.toolchain.installed.
        return true;
    }
    if (method == QStringLiteral("toolchain.inspectSysroot")) {
        emit sysrootInspected(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("toolchain.importKit")) {
        emit kitImported(result.toVariantMap());
        return true;
    }
    if (method != QStringLiteral("toolchain.get") && method != QStringLiteral("toolchain.set") &&
        method != QStringLiteral("toolchain.setKit"))
    {
        return false;
    }
    // Os dois metodos respondem o MESMO shape, entao ha um sinal so: a UI nunca
    // precisa casar resposta com pedido para saber o que mudou.
    emit toolchainResolved(result.value(QStringLiteral("selections")).toArray().toVariantList(),
                           result.value(QStringLiteral("candidates")).toArray().toVariantList(),
                           result.value(QStringLiteral("preset")).toString(),
                           result.value(QStringLiteral("sysroot")).toString(),
                           result.value(QStringLiteral("targetTriple")).toString(),
                           result.value(QStringLiteral("chip")).toString(),
                           result.value(QStringLiteral("presetToolchainFile")).toString());
    emit toolchainKitFileResolved(result.value(QStringLiteral("toolchainFile")).toString());
    emit toolchainKitSvdResolved(result.value(QStringLiteral("svdFile")).toString());
    // O que so' um processo responde (integracoes/39, 0.97.0): os alvos Rust
    // instalados (ausente = sem rustup) e a dica de sysroot. Sinal proprio para
    // o de cima nao crescer em argumento posicional.
    const QJsonValue alvosRust = result.value(QStringLiteral("rustTargets"));
    emit toolchainAdvice(result.value(QStringLiteral("sysrootHint")).toString(),
                         alvosRust.isArray() ? alvosRust.toArray().toVariantList() : QVariantList{},
                         alvosRust.isArray());
    return true;
}

} // namespace kinein
