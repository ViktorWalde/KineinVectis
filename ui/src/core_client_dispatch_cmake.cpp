// Dispatch do dominio CMake/build (ARCHITECTURE.md §5).
//
// POR QUE ESTE ARQUIVO EXISTE. Estas duas funcoes moravam no
// `core_client_dispatch.cpp`, que esta na catraca em 804 linhas e cuja propria
// entrada na ARCHITECTURE.md §5 diz "ja manda dividir por dominio". O gatilho
// foi concreto: acrescentar `cdbStale` ao `cmake.status` faria um arquivo em
// debito CRESCER. As duas saidas faceis seriam espremer o emit numa linha ou
// subir o baseline — a §4 regra 9 chama as duas de trapaca. A terceira e' a
// certa e e' a que o documento ja mandava: devolver o dominio ao dono.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

bool CoreClient::dispatchCmakeResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("runConfig.list") || method == QStringLiteral("runConfig.save") ||
        method == QStringLiteral("runConfig.delete") ||
        method == QStringLiteral("runConfig.setActive"))
    {
        emit runConfigsResolved(result.value(QStringLiteral("configs")).toArray().toVariantList(),
                                result.value(QStringLiteral("activeId")).toString());
        return true;
    }
    if (method == QStringLiteral("runConfig.flashProposal")) {
        // Mapa inteiro: nome, comando, motor, evidencias e avisos viajam
        // juntos e a tela os mostra como PREVIA — nada foi salvo nem rodou.
        emit flashProposalResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("cargo.metadata")) {
        emit cargoMetadataResolved(
            static_cast<int>(result.value(QStringLiteral("packages")).toArray().size()));
        return true;
    }
    if (method == QStringLiteral("cargo.check")) {
        appendLog(QStringLiteral("job aceito (cargo.check): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
        return true;
    }
    if (method == QStringLiteral("cmake.status")) {
        // `cdbStale`/`cdbStaleBecause` sao OMITIDOS pelo core quando falsos
        // (skip_serializing_if no kinein-protocol), entao ausente vira false /
        // string vazia — que e' exatamente o que a UI deve mostrar.
        emit cmakeStatusResolved(result.value(QStringLiteral("configured")).toBool(),
                                 result.value(QStringLiteral("hasCompileCommands")).toBool(),
                                 result.value(QStringLiteral("cdbStale")).toBool(),
                                 result.value(QStringLiteral("cdbStaleBecause")).toString(),
                                 result.value(QStringLiteral("preset")).toString());
        return true;
    }
    if (method == QStringLiteral("cmake.presets.list")) {
        emit cmakePresetsResolved(
            result.value(QStringLiteral("presets")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("cmake.configure")) {
        appendLog(QStringLiteral("job aceito (cmake.configure): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
        return true;
    }
    // As Configuration Actions sao a configuracao de CMake/Cargo e entram no
    // fim desta mesma cadeia: assim o `core_client_dispatch.cpp` — que esta na
    // catraca e cuja entrada na §5 ja manda dividir por dominio — nao ganha
    // uma linha por causa de um dominio novo. O codigo delas vive no arquivo
    // proprio (`core_client_configaction.cpp`).
    if (dispatchConfigActionResult(method, result)) {
        return true;
    }
    if (method == QStringLiteral("cmake.targets.list")) {
        emit cmakeTargetsResolved(result.value(QStringLiteral("targets")).toArray().toVariantList(),
                                  result.value(QStringLiteral("origin")).toString());
        return true;
    }
    if (dispatchToolchainResult(method, result)) {
        return true;
    }
    if (dispatchLibraryResult(method, result)) {
        return true;
    }
    return dispatchDataSourceResult(method, result);
}

bool CoreClient::handleCmakeNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.cmake.started")) {
        // `presetSource` (0.115.0) diz DE ONDE veio o preset: o kit, o
        // CMakeUserPresets.json ou o CMakePresets.json — e' a evidencia de
        // "com que a IDE configurou".
        const QString source = params.value(QStringLiteral("presetSource")).toString();
        appendLog(QStringLiteral("cmake configure iniciado: %1%2")
                      .arg(params.value(QStringLiteral("command")).toString(),
                           source.isEmpty() ? QString()
                                            : QStringLiteral(" (preset de: %1)").arg(source)));
        return true;
    }
    if (method == QStringLiteral("event.cmake.finished")) {
        const bool success = params.value(QStringLiteral("success")).toBool();
        appendLog(QStringLiteral("cmake configure finalizado (sucesso: %1)")
                      .arg(success ? QStringLiteral("sim") : QStringLiteral("nao")));
        emit cmakeConfigureFinished(success);
        if (m_workspaceBuildSystems.contains(QStringLiteral("cmake"))) {
            cmakeStatus();
        }
        return true;
    }
    return false;
}

} // namespace kinein
