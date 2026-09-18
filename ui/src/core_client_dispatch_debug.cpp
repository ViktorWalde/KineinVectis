// Dispatch do dominio DEBUG (ARCHITECTURE.md §5): as respostas dos metodos
// `debug.*`.
//
// POR QUE ESTE ARQUIVO EXISTE. Mesmo motivo e mesmo precedente do
// `core_client_dispatch_lsp.cpp` e do `_cmake.cpp`: o `core_client_dispatch.cpp`
// esta na catraca e a entrada dele na §5 ja manda dividir por dominio. O
// gatilho de hoje foi concreto: acrescentar `debug.evaluate` (etapa 15 do
// roadmaps/34) fez um arquivo em debito CRESCER de 640 para 656. Espremer a
// linha ou subir o baseline sao as duas trapacas que a §4 regra 9 nomeia; a
// terceira e devolver o dominio ao dono.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

bool CoreClient::dispatchDebugResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("debug.start")) {
        appendLog(QStringLiteral("debug aceito: %1")
                      .arg(result.value(QStringLiteral("program")).toString()));
        return true;
    }
    if (method == QStringLiteral("git.status") || method == QStringLiteral("git.stage") ||
        method == QStringLiteral("git.unstage") || method == QStringLiteral("git.discard") ||
        method == QStringLiteral("git.commit") || method == QStringLiteral("git.checkout") ||
        method == QStringLiteral("git.branchCreate") || method == QStringLiteral("git.stash"))
    {
        emit gitStatusResolved(result.value(QStringLiteral("repo")).toBool(),
                               result.value(QStringLiteral("branch")).toString(),
                               result.value(QStringLiteral("detached")).toBool(),
                               result.value(QStringLiteral("shortSha")).toString(),
                               result.value(QStringLiteral("ahead")).toInt(0),
                               result.value(QStringLiteral("behind")).toInt(0),
                               result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.branches")) {
        emit gitBranchesResolved(
            result.value(QStringLiteral("repo")).toBool(),
            result.value(QStringLiteral("branches")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.fileDiff")) {
        emit gitFileDiffResolved(result.value(QStringLiteral("path")).toString(),
                                 result.value(QStringLiteral("repo")).toBool(),
                                 result.value(QStringLiteral("tracked")).toBool(),
                                 result.value(QStringLiteral("hunks")).toArray().toVariantList(),
                                 result.value(QStringLiteral("text")).toString());
        return true;
    }
    if (method == QStringLiteral("git.blame")) {
        emit gitBlameResolved(result.value(QStringLiteral("path")).toString(),
                              result.value(QStringLiteral("repo")).toBool(),
                              result.value(QStringLiteral("tracked")).toBool(),
                              result.value(QStringLiteral("groups")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.log")) {
        emit gitLogResolved(result.value(QStringLiteral("repo")).toBool(),
                            result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("git.commitDiff")) {
        emit gitCommitDiffResolved(result.value(QStringLiteral("sha")).toString(),
                                   result.value(QStringLiteral("text")).toString());
        return true;
    }
    if (method == QStringLiteral("settings.get") || method == QStringLiteral("settings.set")) {
        emit settingsResolved(result.value(QStringLiteral("settings")).toObject().toVariantMap(),
                              result.value(QStringLiteral("global")).toObject().toVariantMap(),
                              result.value(QStringLiteral("workspace")).toObject().toVariantMap());
        return true;
    }
    if (method == QStringLiteral("debug.stackTrace")) {
        emit debugStackTraceResolved(
            result.value(QStringLiteral("frames")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("debug.variables")) {
        emit debugVariablesResolved(
            result.value(QStringLiteral("frameId")).toDouble(-1),
            result.value(QStringLiteral("ref")).toDouble(-1),
            result.value(QStringLiteral("variables")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("debug.scopes")) {
        emit debugScopesResolved(result.value(QStringLiteral("frameId")).toDouble(-1),
                                 result.value(QStringLiteral("scopes")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("debug.readMemory")) {
        // address, data (base64) e unreadableBytes viajam juntos: a tela
        // decodifica e mostra em hexadecimal.
        emit debugMemoryResolved(result.toVariantMap());
        return true;
    }
    if (method == QStringLiteral("debug.disassemble")) {
        emit debugDisassemblyResolved(
            result.value(QStringLiteral("instructions")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("debug.evaluate")) {
        emit debugEvaluateResolved(result.value(QStringLiteral("expression")).toString(),
                                   result.value(QStringLiteral("result")).toString(),
                                   result.value(QStringLiteral("typeName")).toString(),
                                   result.value(QStringLiteral("reference")).toDouble(0));
        return true;
    }
    // O estado da UI vem dos eventos event.debug.*; as respostas de
    // controle/breakpoints nao carregam nada que a UI ja nao saiba.
    return method == QStringLiteral("debug.setBreakpoints") ||
           method == QStringLiteral("debug.continue") || method == QStringLiteral("debug.next") ||
           method == QStringLiteral("debug.stepIn") || method == QStringLiteral("debug.stepOut") ||
           method == QStringLiteral("debug.pause") || method == QStringLiteral("debug.stop");
}

} // namespace kinein
