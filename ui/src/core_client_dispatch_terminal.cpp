// Dispatch do dominio TERMINAL (ARCHITECTURE.md §5): as respostas dos metodos
// `terminal.*` que viram sinal; os eventos ficam em core_client_notifications.
//
// POR QUE ESTE ARQUIVO EXISTE. Mesmo precedente do `_cmake`, `_lsp` e
// `_debug`: o `terminal.pendingCommand` da INF-02 (0.205.0) fez o
// `core_client_dispatch.cpp` passar do limite. O dominio volta ao dono em vez
// de espremer linhas ou subir o baseline (§4 regra 9).
#include "core_client.h"

#include <QJsonValue>

namespace kinein {

bool CoreClient::dispatchTerminalResult(const QString& method, const QJsonObject& result,
                                        qint64 requestId)
{
    if (method == QStringLiteral("terminal.copySelection")) {
        const QJsonValue text = result.value(QStringLiteral("text"));
        emit terminalSelectionCopied(result.value(QStringLiteral("id")).toString(),
                                     result.value(QStringLiteral("selectionId")).toString(),
                                     text.toString(), text.isString());
        return true;
    }
    if (method == QStringLiteral("terminal.open")) {
        const QString id = result.value(QStringLiteral("id")).toString();
        const QString shell = result.value(QStringLiteral("shell")).toString();
        const QString commandState = result.value(QStringLiteral("command")).toString();
        m_terminalIds.insert(id);
        setTerminalActive(!m_terminalIds.isEmpty());
        emit terminalOpened(id, shell, requestId, commandState);
        appendLog(QStringLiteral("terminal aberto (%1): %2").arg(id, shell));
        return true;
    }
    if (method == QStringLiteral("terminal.pendingCommand")) {
        // A mudanca de estado chega por `event.terminal.command`; aqui so' o
        // registro do que o pedido fez ("none" quando nada aguardava).
        appendLog(QStringLiteral("comando pendente (%1): %2")
                      .arg(result.value(QStringLiteral("id")).toString(),
                           result.value(QStringLiteral("state")).toString()));
        return true;
    }
    return false;
}

} // namespace kinein
