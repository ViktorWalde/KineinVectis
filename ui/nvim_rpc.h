// O canal msgpack-RPC da UI com o Neovim, por socket local.
//
// Pedido: [0, id, método, args]; resposta: [1, id, erro, resultado];
// notificação: [2, método, args]. O Neovim também pede à UI (área de
// transferência): o pedido chega pelo sinal `requestReceived` e a resposta sai
// por `respond`.
#pragma once

#include <QHash>
#include <QLocalSocket>
#include <QObject>
#include <QVariant>

#include <functional>

namespace prova {

class NvimRpc : public QObject
{
    Q_OBJECT

public:
    using Callback = std::function<void(const QVariant& error, const QVariant& result)>;

    explicit NvimRpc(QObject* parent = nullptr);

    void connectTo(const QString& path);
    void disconnectFrom();
    [[nodiscard]] bool isConnected() const;

    void request(const QString& method, const QVariantList& args, Callback done = {});
    void notify(const QString& method, const QVariantList& args);
    void respond(qint64 id, const QVariant& error, const QVariant& result);

    // Mensagens que não seguem o msgpack-RPC (item 2 da prova: tem de ser zero).
    [[nodiscard]] qint64 protocolErrors() const { return m_protocolErrors; }

signals:
    void connected();
    void disconnected();
    void notificationReceived(const QString& method, const QVariantList& params);
    void requestReceived(qint64 id, const QString& method, const QVariantList& params);

private:
    void onReadyRead();
    void dispatch(const QVariantList& message);
    void send(const QVariantList& message);

    QLocalSocket m_socket;
    QByteArray m_buffer;
    qint64 m_nextId = 1;
    qint64 m_protocolErrors = 0;
    QHash<qint64, Callback> m_pending;
};

} // namespace prova
