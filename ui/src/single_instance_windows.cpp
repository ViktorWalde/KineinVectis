// O FIO DA INSTANCIA UNICA NO WINDOWS: named pipe pelo `QLocalServer` e o
// `QLocalSocket` (decisao D4 do autor, DocsPublic/roadmaps/60 §3.1). O
// protocolo de texto e as decisoes sao os mesmos do Unix e moram no
// `single_instance.cpp`; aqui so' muda o fio.
//
// O pipe e' da maquina inteira (`\\.\pipe\...`): o nome leva o usuario
// (`runtimeDirectory`), e o `UserAccessOption` deixa so' ele conectar — o
// mesmo efeito do socket no `XDG_RUNTIME_DIR`, que so' o dono le.
#include "single_instance.h"

#include <QLocalServer>
#include <QLocalSocket>

#include <map>

namespace kinein {

namespace {

/// O teto de uma linha do protocolo, como no Unix.
constexpr int kLineLimit = 8192;

/// Os servidores vivos, pelo numero que a API devolve no lugar do descritor.
std::map<int, std::unique_ptr<QLocalServer>>& servers()
{
    static std::map<int, std::unique_ptr<QLocalServer>> live;
    return live;
}

QLocalServer* serverFor(int listenFd)
{
    auto& live = servers();
    const auto found = live.find(listenFd);
    return found == live.end() ? nullptr : found->second.get();
}

/// Le' ate' a primeira quebra de linha, esperando no maximo `timeoutMs` por
/// vez — um dono travado nao segura o terminal de quem chegou.
QString readLine(QLocalSocket& socket, int timeoutMs)
{
    QByteArray accumulated;
    while (!accumulated.contains('\n') && accumulated.size() < kLineLimit) {
        if (socket.bytesAvailable() == 0 && !socket.waitForReadyRead(timeoutMs)) {
            break;
        }
        accumulated.append(socket.readAll());
    }
    return QString::fromUtf8(accumulated);
}

bool writeAll(QLocalSocket& socket, const QString& text, int timeoutMs)
{
    const QByteArray bytes = text.toUtf8();
    return socket.write(bytes) == bytes.size() && socket.waitForBytesWritten(timeoutMs);
}

} // namespace

QString runtimeDirectory()
{
    const QString user = qEnvironmentVariable("USERNAME");
    return user.isEmpty() ? QString{} : QStringLiteral("kinein-%1").arg(user);
}

std::unique_ptr<QObject> watchIncoming(int listenFd, const std::function<void()>& onReady)
{
    QLocalServer* server = serverFor(listenFd);
    if (server == nullptr) {
        return nullptr;
    }
    auto watcher = std::make_unique<QObject>();
    QObject::connect(server, &QLocalServer::newConnection, watcher.get(), [onReady] { onReady(); });
    return watcher;
}

bool handOff(const QString& socketPath, const QString& workspacePath,
             const QString& activationToken, int timeoutMs)
{
    if (socketPath.isEmpty()) {
        return false;
    }
    QLocalSocket socket;
    socket.connectToServer(socketPath);
    if (!socket.waitForConnected(timeoutMs)) {
        // Ninguem atende: quem chegou agora e' o dono.
        return false;
    }
    const bool sent = writeAll(socket, encodeRequest(workspacePath, activationToken), timeoutMs);
    const QString reply = sent ? readLine(socket, timeoutMs) : QString{};
    socket.disconnectFromServer();
    return answerIsMine(reply);
}

int listenFor(const QString& socketPath)
{
    if (socketPath.isEmpty()) {
        return -1;
    }
    // VIVO, decidido por conectar, como no Unix. Nao ha' orfao para limpar: o
    // Windows apaga o pipe junto com o ultimo handle, inclusive num crash.
    {
        QLocalSocket probe;
        probe.connectToServer(socketPath);
        if (probe.waitForConnected(150)) {
            probe.disconnectFromServer();
            return -1;
        }
    }
    auto server = std::make_unique<QLocalServer>();
    server->setSocketOptions(QLocalServer::UserAccessOption);
    if (!server->listen(socketPath)) {
        return -1;
    }
    static int nextId = 1;
    const int id = nextId++;
    servers().emplace(id, std::move(server));
    return id;
}

std::optional<Request> acceptOne(int listenFd, const QString& myWorkspacePath)
{
    QLocalServer* server = serverFor(listenFd);
    if (server == nullptr) {
        return std::nullopt;
    }
    const std::unique_ptr<QLocalSocket> client{server->nextPendingConnection()};
    if (!client) {
        return std::nullopt;
    }
    std::optional<Request> request = parseRequest(readLine(*client, 400));
    const bool mine = request.has_value() && sameWorkspace(request->workspacePath, myWorkspacePath);
    // QUEM NAO E' DAQUI RECEBE UM NAO, e nao o silencio, como no Unix.
    writeAll(*client, mine ? mineAnswer() : notMineAnswer(), 400);
    client->disconnectFromServer();
    return mine ? request : std::nullopt;
}

void releaseSocket(int listenFd, const QString& /*socketPath*/)
{
    servers().erase(listenFd);
}

} // namespace kinein
