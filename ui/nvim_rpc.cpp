#include "nvim_rpc.h"

#include "msgpack.h"

namespace prova {

NvimRpc::NvimRpc(QObject* parent) : QObject(parent)
{
    connect(&m_socket, &QLocalSocket::connected, this, &NvimRpc::connected);
    connect(&m_socket, &QLocalSocket::disconnected, this, [this] {
        // Quem esperava resposta recebe erro: nenhum pedido fica pendurado.
        const auto pending = std::exchange(m_pending, {});
        for (const auto& done : pending) {
            if (done) {
                done(QStringLiteral("canal fechado"), {});
            }
        }
        m_buffer.clear();
        emit disconnected();
    });
    connect(&m_socket, &QLocalSocket::readyRead, this, &NvimRpc::onReadyRead);
}

void NvimRpc::connectTo(const QString& path)
{
    m_socket.abort();
    m_buffer.clear();
    m_socket.connectToServer(path);
}

void NvimRpc::disconnectFrom()
{
    m_socket.abort();
}

bool NvimRpc::isConnected() const
{
    return m_socket.state() == QLocalSocket::ConnectedState;
}

void NvimRpc::request(const QString& method, const QVariantList& args, Callback done)
{
    const qint64 id = m_nextId++;
    m_pending.insert(id, std::move(done));
    send({0, id, method, QVariant(args)});
}

void NvimRpc::notify(const QString& method, const QVariantList& args)
{
    send({2, method, QVariant(args)});
}

void NvimRpc::respond(qint64 id, const QVariant& error, const QVariant& result)
{
    send({1, id, error, result});
}

void NvimRpc::send(const QVariantList& message)
{
    QByteArray out;
    msgpack::encode(out, message);
    m_socket.write(out);
}

void NvimRpc::onReadyRead()
{
    m_buffer.append(m_socket.readAll());
    qsizetype pos = 0;
    while (pos < m_buffer.size()) {
        qsizetype at = pos;
        const auto message = msgpack::decode(m_buffer, at);
        if (!message) {
            break;
        }
        pos = at;
        if (message->typeId() != QMetaType::QVariantList) {
            ++m_protocolErrors;
            continue;
        }
        dispatch(message->toList());
    }
    m_buffer.remove(0, pos);
}

void NvimRpc::dispatch(const QVariantList& message)
{
    if (message.isEmpty()) {
        ++m_protocolErrors;
        return;
    }
    switch (message.at(0).toInt()) {
    case 0:
        if (message.size() == 4) {
            emit requestReceived(message.at(1).toLongLong(), message.at(2).toString(),
                                 message.at(3).toList());
        } else {
            ++m_protocolErrors;
        }
        return;
    case 1: {
        if (message.size() != 4) {
            ++m_protocolErrors;
            return;
        }
        const auto done = m_pending.take(message.at(1).toLongLong());
        if (done) {
            done(message.at(2), message.at(3));
        }
        return;
    }
    case 2:
        if (message.size() == 3) {
            emit notificationReceived(message.at(1).toString(), message.at(2).toList());
        } else {
            ++m_protocolErrors;
        }
        return;
    default:
        ++m_protocolErrors;
        return;
    }
}

} // namespace prova
