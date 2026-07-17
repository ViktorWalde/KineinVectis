// Dispatch das respostas e notificacoes de BUILD (CMake + workspace).
//
// Extraido de core_client_dispatch.cpp em 2026-07-17 (Fase 1.2): a §5 da
// ARCHITECTURE manda dividir o dispatch POR DOMINIO. Mesma classe CoreClient,
// metodos ja declarados em core_client.h — movimento puro, zero mudanca de
// logica. O roteador central (dispatchResult/handleNotification) fica no
// arquivo original e continua chamando estes metodos.

#include "core_client.h"

#include <QJsonArray>
#include <QJsonObject>
#include <QJsonValue>

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
        emit cmakeStatusResolved(result.value(QStringLiteral("configured")).toBool(),
                                 result.value(QStringLiteral("hasCompileCommands")).toBool());
        return true;
    }
    if (method == QStringLiteral("cmake.configure")) {
        appendLog(QStringLiteral("job aceito (cmake.configure): %1")
                      .arg(result.value(QStringLiteral("jobId")).toString()));
        return true;
    }
    return false;
}

bool CoreClient::handleCmakeNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.cmake.started")) {
        appendLog(QStringLiteral("cmake configure iniciado: %1")
                      .arg(params.value(QStringLiteral("command")).toString()));
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

bool CoreClient::dispatchWorkspaceResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("workspace.open") ||
        method == QStringLiteral("workspace.createProject"))
    {
        handleWorkspaceOpened(result);
        return true;
    }
    if (method.startsWith(QStringLiteral("workspace.recent."))) {
        emit recentWorkspacesResolved(
            result.value(QStringLiteral("workspaces")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("workspace.createFolder")) {
        emit workspaceFolderCreated(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("workspace.browse")) {
        emit workspaceBrowseListed(
            result.value(QStringLiteral("path")).toString(),
            result.value(QStringLiteral("parent")).toString(),
            result.value(QStringLiteral("entries")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("workspace.close")) {
        m_workspaceRoot.clear();
        m_workspaceName.clear();
        m_workspaceKind.clear();
        m_workspaceBuildSystems.clear();
        m_terminalIds.clear();
        setTerminalActive(false);
        emit workspaceChanged();
        return true;
    }
    return false;
}

void CoreClient::handleWorkspaceOpened(const QJsonObject& result)
{
    m_workspaceRoot = result.value(QStringLiteral("root")).toString();
    m_workspaceName = result.value(QStringLiteral("name")).toString();
    m_workspaceKind = result.value(QStringLiteral("kind")).toString();
    m_workspaceBuildSystems.clear();
    const QJsonArray buildSystems = result.value(QStringLiteral("capabilities"))
                                        .toObject()
                                        .value(QStringLiteral("buildSystems"))
                                        .toArray();
    for (const QJsonValue buildSystem : buildSystems) {
        const QString value = buildSystem.toString();
        if (!value.isEmpty() && !m_workspaceBuildSystems.contains(value)) {
            m_workspaceBuildSystems.append(value);
        }
    }
    // Tolerate an older core response during crash recovery; protocol version
    // negotiation still prevents unsupported requests in normal operation.
    if (m_workspaceBuildSystems.isEmpty()) {
        if (m_workspaceKind == QStringLiteral("rustCargo")) {
            m_workspaceBuildSystems.append(QStringLiteral("cargo"));
        }
        else if (m_workspaceKind == QStringLiteral("cmake")) {
            m_workspaceBuildSystems.append(QStringLiteral("cmake"));
        }
    }
    // M4.3: lembra o root para recuperar de um crash futuro.
    m_lastWorkspaceRoot = m_workspaceRoot;
    emit workspaceChanged();
    listRecentWorkspaces();
    listDir(m_workspaceRoot);
    if (m_workspaceBuildSystems.contains(QStringLiteral("cmake"))) {
        cmakeStatus();
    }
    if (m_workspaceBuildSystems.contains(QStringLiteral("cargo"))) {
        cargoMetadata();
    }
    runConfigList();
    gitStatus();
    // Na RECUPERACAO, NAO restaurar a sessao (as abas/edicoes ja estao na
    // UI; reler do disco sobrescreveria edicoes nao salvas). Reabrir o mesmo
    // root nao limpa a UI (WorkspaceController).
    if (m_recovering) {
        setRecovering(false);
        m_recoveryAttempts = 0;
        appendLog(QStringLiteral("core recuperado; workspace reconectado"));
        emit recovered();
        return;
    }
    if (result.contains(QStringLiteral("session"))) {
        const QJsonObject session = result.value(QStringLiteral("session")).toObject();
        QStringList files;
        const QJsonArray sessionFiles = session.value(QStringLiteral("openFiles")).toArray();
        files.reserve(sessionFiles.size());
        for (const QJsonValue file : sessionFiles) {
            files.append(file.toString());
        }
        if (!files.isEmpty()) {
            emit sessionRestored(files, session.value(QStringLiteral("activeFile")).toString());
        }
    }
    // M-S1: rascunhos não salvos recuperados de um crash (docs/seguranca/23). Só em
    // abertura normal — a recuperação de crash do core (acima) sai antes.
    if (result.contains(QStringLiteral("drafts"))) {
        const QVariantList drafts =
            result.value(QStringLiteral("drafts")).toArray().toVariantList();
        if (!drafts.isEmpty()) {
            emit draftsRecovered(drafts);
        }
    }
}

} // namespace kinein
