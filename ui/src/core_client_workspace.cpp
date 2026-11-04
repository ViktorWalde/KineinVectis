#include "core_client.h"

#include <QJsonArray>
#include <QJsonObject>

namespace kinein {

void CoreClient::openWorkspace(const QString& path)
{
    sendRequest(QStringLiteral("workspace.open"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::listRecentWorkspaces()
{
    sendRequest(QStringLiteral("workspace.recent.list"), QJsonObject{});
}

void CoreClient::pinRecentWorkspace(const QString& root, bool pinned)
{
    sendRequest(QStringLiteral("workspace.recent.pin"),
                QJsonObject{{QStringLiteral("root"), root}, {QStringLiteral("pinned"), pinned}});
}

void CoreClient::removeRecentWorkspace(const QString& root)
{
    sendRequest(QStringLiteral("workspace.recent.remove"),
                QJsonObject{{QStringLiteral("root"), root}});
}

void CoreClient::clearRecentWorkspaces()
{
    sendRequest(QStringLiteral("workspace.recent.clear"), QJsonObject{});
}

void CoreClient::browseWorkspaceFolders(const QString& path)
{
    sendRequest(QStringLiteral("workspace.browse"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::createWorkspaceFolder(const QString& parent, const QString& name)
{
    sendRequest(QStringLiteral("workspace.createFolder"),
                QJsonObject{{QStringLiteral("parent"), parent}, {QStringLiteral("name"), name}});
}

void CoreClient::createWorkspaceProject(const QString& parent, const QString& name,
                                        const QString& templateId, const QVariantMap& options)
{
    sendRequest(QStringLiteral("workspace.createProject"),
                QJsonObject{{QStringLiteral("parent"), parent},
                            {QStringLiteral("name"), name},
                            {QStringLiteral("template"), templateId},
                            {QStringLiteral("options"), QJsonObject::fromVariantMap(options)}});
}

void CoreClient::projectEnvironments()
{
    sendRequest(QStringLiteral("workspace.projectEnvironments"), QJsonObject{});
}

void CoreClient::closeWorkspace()
{
    // M4.3: fechar de proposito nao deve recuperar "vazio" num crash futuro.
    m_lastWorkspaceRoot.clear();
    sendRequest(QStringLiteral("workspace.close"), QJsonObject{});
}

void CoreClient::saveSession(const QStringList& openFiles, const QString& activeFile)
{
    QJsonArray files;
    for (const QString& file : openFiles) {
        files.append(file);
    }
    QJsonObject params{{QStringLiteral("openFiles"), files}};
    if (!activeFile.isEmpty()) {
        params.insert(QStringLiteral("activeFile"), activeFile);
    }
    sendRequest(QStringLiteral("workspace.saveSession"), params);
}

} // namespace kinein
