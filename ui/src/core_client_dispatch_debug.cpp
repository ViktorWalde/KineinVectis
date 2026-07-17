// Dispatch das respostas e notificacoes de DEBUG (DAP).
//
// Extraido de core_client_dispatch.cpp em 2026-07-17 (Fase 1.2): a §5 da
// ARCHITECTURE manda dividir o dispatch POR DOMINIO. Mesma classe CoreClient,
// metodos ja declarados em core_client.h — movimento puro, zero mudanca de
// logica. O roteador central (dispatchResult/handleNotification) fica no
// arquivo original e continua chamando estes metodos.

#include "core_client.h"

#include <QJsonArray>

#include <QJsonObject>

namespace kinein {

bool CoreClient::handleDebugNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.debug.started")) {
        const QString program = params.value(QStringLiteral("program")).toString();
        appendLog(QStringLiteral("debug iniciado: %1").arg(program));
        setDebugging(true);
        emit debugStarted(program);
        return true;
    }
    if (method == QStringLiteral("event.debug.output")) {
        emit debugOutput(params.value(QStringLiteral("category")).toString(),
                         params.value(QStringLiteral("line")).toString());
        return true;
    }
    if (method == QStringLiteral("event.debug.stopped")) {
        emit debugStopped(params.value(QStringLiteral("reason")).toString(),
                          params.value(QStringLiteral("file")).toString(),
                          params.value(QStringLiteral("line")).toInt(0),
                          params.value(QStringLiteral("threadId")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("event.debug.continued")) {
        emit debugContinued();
        return true;
    }
    if (method == QStringLiteral("event.debug.finished")) {
        setDebugging(false);
        emit debugFinished(params.value(QStringLiteral("exitCode")).toInt(-1));
        return true;
    }
    return false;
}

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
    // O estado da UI vem dos eventos event.debug.*; as respostas de
    // controle/breakpoints nao carregam nada que a UI ja nao saiba.
    return method == QStringLiteral("debug.setBreakpoints") ||
           method == QStringLiteral("debug.continue") || method == QStringLiteral("debug.next") ||
           method == QStringLiteral("debug.stepIn") || method == QStringLiteral("debug.stepOut") ||
           method == QStringLiteral("debug.pause") || method == QStringLiteral("debug.stop");
}

} // namespace kinein
