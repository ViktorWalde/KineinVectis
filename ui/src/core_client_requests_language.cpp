#include "core_client.h"

#include <QJsonArray>
#include <QJsonObject>

// Pedidos do dominio de LINGUAGEM (lsp.*, syntaxTree.*): extraidos do
// core_client_requests.cpp em 2026-07-19, quando a fatia 3 do L2 precisou de
// espaco no arquivo em debito (551/500) — mesmo corte por dominio ja aplicado
// ao dispatch (§5, core_client_dispatch_language.cpp).

namespace kinein {

void CoreClient::notifyFileChanged(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.didChange"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::requestDefinition(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.definition"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestHover(const QString& path, const QString& content, int line, int column)
{
    sendRequest(QStringLiteral("lsp.hover"), QJsonObject{{QStringLiteral("path"), path},
                                                         {QStringLiteral("content"), content},
                                                         {QStringLiteral("line"), line},
                                                         {QStringLiteral("column"), column}});
}

void CoreClient::requestCompletion(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.completion"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestReferences(const QString& path, const QString& content, int line,
                                   int column)
{
    sendRequest(QStringLiteral("lsp.references"), QJsonObject{{QStringLiteral("path"), path},
                                                              {QStringLiteral("content"), content},
                                                              {QStringLiteral("line"), line},
                                                              {QStringLiteral("column"), column}});
}

void CoreClient::requestCodeActions(const QString& path, const QString& content, int line,
                                    int column)
{
    sendRequest(QStringLiteral("lsp.codeActions"), QJsonObject{{QStringLiteral("path"), path},
                                                               {QStringLiteral("content"), content},
                                                               {QStringLiteral("line"), line},
                                                               {QStringLiteral("column"), column}});
}

void CoreClient::applyCodeAction(const QString& path, const QString& content, int actionIndex)
{
    sendRequest(QStringLiteral("lsp.applyCodeAction"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("actionIndex"), actionIndex}});
}

void CoreClient::applyWorkspaceEdit(const QString& transactionId)
{
    sendRequest(QStringLiteral("lsp.workspaceEdit.apply"),
                QJsonObject{{QStringLiteral("transactionId"), transactionId}});
}

void CoreClient::cancelWorkspaceEdit(const QString& transactionId)
{
    sendRequest(QStringLiteral("lsp.workspaceEdit.cancel"),
                QJsonObject{{QStringLiteral("transactionId"), transactionId}});
}

void CoreClient::requestDocumentSymbols(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.documentSymbols"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::requestWorkspaceSymbols(const QString& path, const QString& content,
                                         const QString& query)
{
    sendRequest(QStringLiteral("lsp.workspaceSymbols"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("query"), query}});
}

void CoreClient::requestRename(const QString& path, const QString& content, int line, int column,
                               const QString& newName)
{
    sendRequest(QStringLiteral("lsp.rename"), QJsonObject{{QStringLiteral("path"), path},
                                                          {QStringLiteral("content"), content},
                                                          {QStringLiteral("line"), line},
                                                          {QStringLiteral("column"), column},
                                                          {QStringLiteral("newName"), newName}});
}

void CoreClient::requestSemanticTokens(const QString& path, const QString& content, int version)
{
    sendRequest(QStringLiteral("lsp.semanticTokens"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("version"), version}});
}

void CoreClient::requestSyntaxTree(const QString& path, const QString& content, int version)
{
    sendRequest(QStringLiteral("syntaxTree.update"),
                QJsonObject{{QStringLiteral("path"), path},
                            {QStringLiteral("content"), content},
                            {QStringLiteral("version"), version}});
}

void CoreClient::requestSwitchSourceHeader(const QString& path, const QString& content)
{
    sendRequest(QStringLiteral("lsp.switchSourceHeader"),
                QJsonObject{{QStringLiteral("path"), path}, {QStringLiteral("content"), content}});
}

void CoreClient::requestFileContext(const QString& path)
{
    if (m_process.state() != QProcess::Running || path.isEmpty()) {
        return;
    }
    sendRequest(QStringLiteral("project.fileContext"), QJsonObject{{QStringLiteral("path"), path}});
}

void CoreClient::lspRestart(const QString& language)
{
    // language vazio = reinicia todos os servidores vivos (M4.3b).
    QJsonObject params;
    if (!language.isEmpty()) {
        params.insert(QStringLiteral("language"), language);
    }
    sendRequest(QStringLiteral("lsp.restart"), params);
}

} // namespace kinein
