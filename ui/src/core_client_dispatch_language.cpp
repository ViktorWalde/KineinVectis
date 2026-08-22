// Dispatch das respostas e notificacoes de LINGUAGEM (LSP + Tree-sitter).
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

bool CoreClient::handleLspNotification(const QString& method, const QJsonObject& params)
{
    if (method == QStringLiteral("event.lsp.diagnostics")) {
        emit lspDiagnostics(params.value(QStringLiteral("path")).toString(),
                            params.value(QStringLiteral("diagnostics")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("event.lsp.status")) {
        const QString language = params.value(QStringLiteral("language")).toString();
        const QString status = params.value(QStringLiteral("status")).toString();
        const QString message = params.value(QStringLiteral("message")).toString();
        if (status == QStringLiteral("failed")) {
            appendErrorLog(QStringLiteral("lsp %1: %2 (%3)").arg(language, status, message));
        }
        else {
            appendLog(QStringLiteral("lsp %1: %2").arg(language, status));
        }
        return true;
    }
    if (method == QStringLiteral("event.lsp.restarted")) {
        // M4.3b: servidor reiniciou (auto por timeouts ou lsp.restart) — a UI
        // re-sincroniza o arquivo ativo (mesmo caminho do recovered()).
        emit lspRestarted(params.value(QStringLiteral("language")).toString());
        return true;
    }
    return false;
}

bool CoreClient::dispatchSyntaxResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("project.fileContext")) {
        // L3: o contexto de compilacao efetivo do arquivo. Vive aqui, no
        // dispatch de LINGUAGEM, porque e a mesma pergunta que o clangd faz
        // para decidir como interpretar o arquivo.
        emit fileContextResolved(result.toVariantMap());
        return true;
    }
    if (method != QStringLiteral("syntaxTree.update")) {
        return false;
    }
    emit syntaxTreeResolved(result.value(QStringLiteral("path")).toString(),
                            result.value(QStringLiteral("version")).toInt(),
                            result.value(QStringLiteral("language")).toString(),
                            result.value(QStringLiteral("hasErrors")).toBool(),
                            result.value(QStringLiteral("highlights")).toArray().toVariantList(),
                            result.value(QStringLiteral("foldingRanges")).toArray().toVariantList(),
                            result.value(QStringLiteral("outline")).toArray().toVariantList(),
                            result.value(QStringLiteral("locals")).toArray().toVariantList());
    return true;
}

bool CoreClient::dispatchLspResult(const QString& method, const QJsonObject& result)
{
    if (method == QStringLiteral("lsp.definition")) {
        const QString path = result.value(QStringLiteral("path")).toString();
        if (!path.isEmpty()) {
            emit lspDefinitionResolved(path, result.value(QStringLiteral("line")).toInt(1),
                                       result.value(QStringLiteral("column")).toInt(1));
        }
        return true;
    }
    if (method == QStringLiteral("lsp.hover")) {
        emit lspHoverResolved(result.value(QStringLiteral("content")).toString());
        return true;
    }
    if (method == QStringLiteral("lsp.completion")) {
        emit lspCompletionResolved(result.value(QStringLiteral("items")).toArray().toVariantList(),
                                   result.value(QStringLiteral("isIncomplete")).toBool());
        return true;
    }
    if (method == QStringLiteral("lsp.references")) {
        emit lspReferencesResolved(
            result.value(QStringLiteral("references")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.semanticTokens")) {
        emit lspSemanticTokensResolved(
            result.value(QStringLiteral("path")).toString(),
            result.value(QStringLiteral("version")).toInt(),
            result.value(QStringLiteral("tokens")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.documentSymbols") ||
        method == QStringLiteral("lsp.workspaceSymbols"))
    {
        emit lspSymbolsResolved(result.value(QStringLiteral("symbols")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.switchSourceHeader")) {
        // path ausente = clangd nao achou contraparte (nao e erro).
        emit lspSwitchSourceHeaderResolved(result.value(QStringLiteral("path")).toString());
        return true;
    }
    if (method == QStringLiteral("lsp.restart")) {
        // A re-sincronizacao vem por event.lsp.restarted; aqui so registramos.
        const QStringList restarted =
            result.value(QStringLiteral("restarted")).toVariant().toStringList();
        appendLog(
            restarted.isEmpty()
                ? QStringLiteral("lsp.restart: nenhum servidor ativo")
                : QStringLiteral("lsp.restart: %1").arg(restarted.join(QStringLiteral(", "))));
        return true;
    }
    if (method == QStringLiteral("lsp.codeActions")) {
        emit lspCodeActionsResolved(
            result.value(QStringLiteral("actions")).toArray().toVariantList());
        return true;
    }
    if (method == QStringLiteral("lsp.applyCodeAction") || method == QStringLiteral("lsp.rename")) {
        emit lspWorkspaceEditPreviewResolved(
            result.value(QStringLiteral("transactionId")).toString(),
            result.value(QStringLiteral("title")).toString(),
            result.value(QStringLiteral("files")).toArray().toVariantList(),
            result.value(QStringLiteral("edits")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("lsp.workspaceEdit.apply")) {
        QStringList files;
        const QJsonArray fileArray = result.value(QStringLiteral("files")).toArray();
        files.reserve(fileArray.size());
        for (const QJsonValue file : fileArray) {
            files.append(file.toString());
        }
        emit lspWorkspaceEditApplied(files, result.value(QStringLiteral("title")).toString(),
                                     result.value(QStringLiteral("edits")).toInt(0));
        return true;
    }
    if (method == QStringLiteral("lsp.workspaceEdit.cancel")) {
        emit lspWorkspaceEditCancelled(result.value(QStringLiteral("transactionId")).toString());
        return true;
    }
    return false;
}

} // namespace kinein
