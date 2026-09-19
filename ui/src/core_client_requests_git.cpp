// Dominio GIT no lado da UI: os pedidos `git.*`.
//
// Arquivo proprio pelo mesmo motivo do `core_client_requests_debug.cpp`, do
// `_toolchain.cpp` e do `_library.cpp`: o `core_client_requests.cpp` esta na
// catraca e a §5 da ARCHITECTURE ja manda dividir o `CoreClient` por dominio.
// Git era o MAIOR bloco la dentro — 15 pedidos, mais que qualquer outro.
#include "core_client.h"

#include <QJsonArray>

namespace kinein {

void CoreClient::gitStatus()
{
    sendRequest(QStringLiteral("git.status"), QJsonObject{});
}

void CoreClient::gitBranches()
{
    sendRequest(QStringLiteral("git.branches"), QJsonObject{});
}

void CoreClient::gitCheckout(const QString& branch)
{
    sendRequest(QStringLiteral("git.checkout"), QJsonObject{{QStringLiteral("branch"), branch}});
}

void CoreClient::gitCreateBranch(const QString& name, bool checkout)
{
    sendRequest(
        QStringLiteral("git.branchCreate"),
        QJsonObject{{QStringLiteral("name"), name}, {QStringLiteral("checkout"), checkout}});
}

void CoreClient::gitPull()
{
    sendRequest(QStringLiteral("git.pull"), QJsonObject{});
}

void CoreClient::gitPush()
{
    sendRequest(QStringLiteral("git.push"), QJsonObject{});
}

void CoreClient::gitStash(const QString& action, const QString& message)
{
    QJsonObject params{{QStringLiteral("action"), action}};
    if (!message.trimmed().isEmpty()) {
        params.insert(QStringLiteral("message"), message.trimmed());
    }
    sendRequest(QStringLiteral("git.stash"), params);
}

void CoreClient::gitFileDiff(const QString& path)
{
    sendRequest(QStringLiteral("git.fileDiff"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::gitStage(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.stage"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitUnstage(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.unstage"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitDiscard(const QStringList& paths)
{
    sendRequest(QStringLiteral("git.discard"),
                QJsonObject{{QStringLiteral("paths"), QJsonArray::fromStringList(paths)}});
}

void CoreClient::gitCommit(const QString& message, bool amend)
{
    QJsonObject params{{QStringLiteral("message"), message}};
    // Reescrever o ultimo commit (0.126.0): so' vai quando a tela pediu.
    if (amend) {
        params.insert(QStringLiteral("amend"), true);
    }
    sendRequest(QStringLiteral("git.commit"), params);
}

void CoreClient::gitBlame(const QString& path)
{
    sendRequest(QStringLiteral("git.blame"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::gitLog(const QString& ref)
{
    QJsonObject params;
    // O historico de OUTRO branch/tag (0.127.0): so' vai quando escolhido.
    if (!ref.trimmed().isEmpty()) {
        params.insert(QStringLiteral("ref"), ref.trimmed());
    }
    sendRequest(QStringLiteral("git.log"), params);
}

void CoreClient::gitCommitDiff(const QString& sha)
{
    sendRequest(QStringLiteral("git.commitDiff"), QJsonObject{{QStringLiteral("sha"), sha}});
}

} // namespace kinein
