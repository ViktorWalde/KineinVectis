// O FIO DA INSTANCIA UNICA NO UNIX: socket de dominio Unix no
// `XDG_RUNTIME_DIR`. O protocolo e as decisoes moram no `single_instance.cpp`;
// este arquivo e' o transporte de antes do porte para o Windows (2026-10-09),
// movido sem mudar (DocsPublic/roadmaps/60 §3.2, W3).
#include "single_instance.h"

#include <QDir>
#include <QFileInfo>
#include <QSocketNotifier>

#include <array>
#include <cstring>
#include <span>

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

namespace kinein {

QString runtimeDirectory()
{
    return qEnvironmentVariable("XDG_RUNTIME_DIR");
}

std::unique_ptr<QObject> watchIncoming(int listenFd, const std::function<void()>& onReady)
{
    if (listenFd < 0) {
        return nullptr;
    }
    auto notifier = std::make_unique<QSocketNotifier>(listenFd, QSocketNotifier::Read);
    QObject::connect(notifier.get(), &QSocketNotifier::activated, notifier.get(),
                     [onReady] { onReady(); });
    return notifier;
}

namespace {

/// Preenche o `sockaddr_un`, ou devolve `false` quando o caminho nao cabe.
///
/// NAO CABER E' UM CASO REAL, e nao uma formalidade: `sun_path` tem 108 bytes,
/// e um `XDG_RUNTIME_DIR` incomum mais o nome do arquivo podem passar disso.
/// Truncar silenciosamente faria dois projetos diferentes apontarem para o
/// mesmo socket.
/// O endereco generico que as chamadas POSIX exigem.
///
/// `connect` e `bind` recebem `sockaddr*` e leem `sa_family` para saber o que
/// veio de verdade. Nao ha' como dizer isso em C++ sem um cast que o
/// clang-tidy recusa — e ele recusa com razao, porque em codigo comum esse
/// cast quase sempre e' engano. Fica UM lugar, explicado, em vez de tres
/// espalhados.
sockaddr* asGeneric(sockaddr_un& address)
{
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<sockaddr*>(&address);
}

bool fillAddress(sockaddr_un& address, const QString& socketPath)
{
    const QByteArray bytes = socketPath.toUtf8();
    if (bytes.isEmpty() || static_cast<std::size_t>(bytes.size()) >= sizeof(address.sun_path)) {
        return false;
    }
    address = {};
    address.sun_family = AF_UNIX;
    std::memcpy(static_cast<void*>(address.sun_path), bytes.constData(),
                static_cast<std::size_t>(bytes.size()));
    return true;
}

/// Um tempo limite curto nos dois sentidos.
///
/// Sem isto, um dono TRAVADO — nao morto, travado — seguraria o terminal de
/// quem so' quis abrir um projeto. Preferimos abrir uma janela a mais.
void setTimeouts(int fd, int timeoutMs)
{
    timeval timeout{};
    timeout.tv_sec = timeoutMs / 1000;
    timeout.tv_usec = static_cast<suseconds_t>(timeoutMs % 1000) * 1000;
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));
}

/// Le' ate' a primeira quebra de linha, ou ate' o limite.
QString readLine(int fd, int limit = 8192)
{
    QByteArray accumulated;
    std::array<char, 512> chunk{};
    while (accumulated.size() < limit) {
        const ssize_t readCount = ::read(fd, chunk.data(), chunk.size());
        if (readCount <= 0) {
            break;
        }
        accumulated.append(chunk.data(), static_cast<int>(readCount));
        if (accumulated.contains('\n')) {
            break;
        }
    }
    return QString::fromUtf8(accumulated);
}

bool writeAll(int fd, const QString& text)
{
    const QByteArray bytes = text.toUtf8();
    // `span` em vez de somar no ponteiro: e' o mesmo remedio que o `main` usa
    // com o `argv`, e pelo mesmo motivo — somar no ponteiro a mao e' o lugar
    // classico de escrever um byte a mais.
    std::span<const char> remaining{bytes.constData(), static_cast<std::size_t>(bytes.size())};
    while (!remaining.empty()) {
        const ssize_t written = ::write(fd, remaining.data(), remaining.size());
        if (written <= 0) {
            return false;
        }
        remaining = remaining.subspan(static_cast<std::size_t>(written));
    }
    return true;
}

} // namespace

bool handOff(const QString& socketPath, const QString& workspacePath,
             const QString& activationToken, int timeoutMs)
{
    sockaddr_un address{};
    if (socketPath.isEmpty() || !fillAddress(address, socketPath)) {
        return false;
    }
    const int fd = ::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) {
        return false;
    }
    setTimeouts(fd, timeoutMs);
    if (::connect(fd, asGeneric(address), sizeof(address)) != 0) {
        // Ninguem atende: ou nunca houve, ou o dono morreu. Nos dois casos
        // quem chegou agora e' o dono.
        ::close(fd);
        return false;
    }
    const bool sent = writeAll(fd, encodeRequest(workspacePath, activationToken));
    const QString reply = sent ? readLine(fd) : QString{};
    ::close(fd);
    return answerIsMine(reply);
}

int listenFor(const QString& socketPath)
{
    sockaddr_un address{};
    if (socketPath.isEmpty() || !fillAddress(address, socketPath)) {
        return -1;
    }
    const QFileInfo info{socketPath};
    QDir folder = info.dir();
    if (!folder.exists() && !folder.mkpath(QStringLiteral("."))) {
        return -1;
    }
    // ORFAO: o arquivo existe e ninguem atende. E' o que sobra de um crash, e
    // deixar a coordenacao quebrada ate' o proximo reboot seria pior.
    if (info.exists()) {
        // VIVO OU ORFAO, decidido por `connect` e mais nada. A primeira versao
        // disto mandava um pedido de verdade para sondar, o que jogava lixo no
        // fio de um dono legitimo — e nao respondia melhor a pergunta.
        const int probe = ::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
        if (probe >= 0) {
            setTimeouts(probe, 150);
            const bool alive = ::connect(probe, asGeneric(address), sizeof(address)) == 0;
            ::close(probe);
            if (alive) {
                return -1;
            }
        }
        // O caminho ja' esta' aqui como texto; usar o campo cru do endereco
        // so' acrescentaria um array decaindo em ponteiro.
        ::unlink(socketPath.toUtf8().constData());
    }
    const int fd = ::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC | SOCK_NONBLOCK, 0);
    if (fd < 0) {
        return -1;
    }
    if (::bind(fd, asGeneric(address), sizeof(address)) != 0 || ::listen(fd, 4) != 0) {
        ::close(fd);
        return -1;
    }
    return fd;
}

std::optional<Request> acceptOne(int listenFd, const QString& myWorkspacePath)
{
    if (listenFd < 0) {
        return std::nullopt;
    }
    const int client = ::accept4(listenFd, nullptr, nullptr, SOCK_CLOEXEC);
    if (client < 0) {
        return std::nullopt;
    }
    setTimeouts(client, 400);
    std::optional<Request> request = parseRequest(readLine(client));
    if (!request.has_value() || !sameWorkspace(request->workspacePath, myWorkspacePath)) {
        // QUEM NAO E' DAQUI RECEBE UM NAO, e nao o silencio: o outro lado
        // esperaria o tempo inteiro do prazo antes de seguir.
        writeAll(client, notMineAnswer());
        ::close(client);
        return std::nullopt;
    }
    writeAll(client, mineAnswer());
    ::close(client);
    return request;
}

void releaseSocket(int listenFd, const QString& socketPath)
{
    if (listenFd >= 0) {
        ::close(listenFd);
    }
    if (!socketPath.isEmpty()) {
        ::unlink(socketPath.toUtf8().constData());
    }
}

} // namespace kinein
