// Dispatch do dominio LSP (ARCHITECTURE.md §5): eventos do servidor e
// respostas dos metodos `lsp.*`.
//
// POR QUE ESTE ARQUIVO EXISTE. Mesma razao — e mesmo precedente — do
// `core_client_dispatch_cmake.cpp` em 2026-08-30: o
// `core_client_dispatch.cpp` esta na catraca e a propria entrada dele na §5
// diz "ja manda dividir por dominio". O gatilho de hoje foi concreto:
// acrescentar `event.lsp.documentsClosed` (etapa 4 do roadmap 30) faria um
// arquivo em debito CRESCER. Espremer a linha ou subir o baseline sao as duas
// trapacas que a §4 regra 9 nomeia; a terceira e devolver o dominio ao dono.
#include "core_client.h"

#include <QJsonArray>

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
        else if (!message.isEmpty()) {
            // `exited` traz a cauda do stderr do servidor (0.111.0): o motivo
            // de ele ter saido, que antes ia para /dev/null.
            appendLog(QStringLiteral("lsp %1: %2 (%3)").arg(language, status, message));
        }
        else {
            appendLog(QStringLiteral("lsp %1: %2").arg(language, status));
        }
        // E a tela (F2 da Etapa 2, 2026-09-18): ate' aqui o estado do servidor
        // so' existia no log — um clangd morto ficava invisivel.
        emit lspStatusChanged(language, status, message);
        return true;
    }
    if (method == QStringLiteral("event.lsp.log")) {
        // O stderr do servidor, linha a linha (0.111.0): vai para a aba IDE,
        // sem interpretacao — e' o que o clangd/rust-analyzer/basedpyright
        // contam de si mesmos.
        appendLog(QStringLiteral("lsp %1 · %2")
                      .arg(params.value(QStringLiteral("language")).toString(),
                           params.value(QStringLiteral("line")).toString()));
        return true;
    }
    if (method == QStringLiteral("event.lsp.restarted")) {
        // M4.3b: servidor reiniciou (auto por timeouts ou lsp.restart) — a UI
        // re-sincroniza o arquivo ativo (mesmo caminho do recovered()).
        emit lspRestarted(params.value(QStringLiteral("language")).toString());
        return true;
    }
    if (method == QStringLiteral("event.lsp.documentsClosed")) {
        // Etapa 4 do roadmap 30: um `cmake.configure` bem-sucedido invalidou a
        // compilacao em cache dos documentos C/C++ abertos, e o core os fechou.
        // A UI re-sincroniza o arquivo ativo — mesmo caminho do restarted —, e
        // so entao o `didOpen` seguinte chega ao servidor com as flags novas.
        const int count = params.value(QStringLiteral("count")).toInt(0);
        const QString language = params.value(QStringLiteral("language")).toString();
        appendLog(QStringLiteral("lsp %1: %2 documento(s) fechado(s) apos o configure")
                      .arg(language)
                      .arg(count));
        emit lspDocumentsClosed(language, count);
        return true;
    }
    return false;
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
