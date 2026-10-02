#include "single_instance.h"

#include <QCryptographicHash>
#include <QDir>
#include <QFileInfo>

#include <array>
#include <cstring>
#include <span>

#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>

namespace kinein {

namespace {

/// O prefixo que identifica o protocolo e a sua versao.
///
/// Versao no proprio texto porque uma IDE velha e uma nova podem coexistir na
/// mesma sessao durante uma atualizacao, e falar linguas diferentes sem
/// perceber seria pior que nao se falarem.
constexpr auto kPrefix = "KINEIN-INSTANCIA-1";
constexpr auto kSim = "MEU";
constexpr auto kNao = "NAO";

/// O separador dos campos. TAB porque caminho de arquivo pode ter espaco —
/// e tem, no teste e na vida.
constexpr QChar kSeparator = QLatin1Char('\t');

} // namespace

QString socketPathFor(const QString& runtimeDir, const QString& workspacePath)
{
    if (runtimeDir.trimmed().isEmpty() || workspacePath.trimmed().isEmpty()) {
        return {};
    }
    // O NOME PRECISA CABER. `sun_path` tem 108 bytes no Linux, e caminho de
    // projeto nao tem limite nenhum: por isso o nome do arquivo e' um digest
    // de tamanho fixo, e nao o caminho escapado.
    const QByteArray digest =
        QCryptographicHash::hash(workspacePath.toUtf8(), QCryptographicHash::Sha256)
            .toHex()
            .left(32);
    return QDir(runtimeDir)
        .filePath(QStringLiteral("kinein-vectis/%1.sock").arg(QString::fromLatin1(digest)));
}

QString encodeRequest(const QString& workspacePath, const QString& activationToken)
{
    // Nenhum dos dois campos pode conter TAB ou quebra de linha. O caminho vem
    // de `QDir::canonicalPath`, e o token do XDG e' base64 — mas confiar nisso
    // seria confiar em quem manda a mensagem, que e' de onde vem o problema.
    QString path = workspacePath;
    QString token = activationToken;
    path.remove(QLatin1Char('\t')).remove(QLatin1Char('\n'));
    token.remove(QLatin1Char('\t')).remove(QLatin1Char('\n'));
    return QStringLiteral("%1\t%2\t%3\n").arg(QLatin1String(kPrefix), path, token);
}

std::optional<Request> parseRequest(const QString& line)
{
    QString cleanLine = line;
    while (cleanLine.endsWith(QLatin1Char('\n')) || cleanLine.endsWith(QLatin1Char('\r'))) {
        cleanLine.chop(1);
    }
    const QStringList fields = cleanLine.split(kSeparator);
    // Tres campos exatos: prefixo, caminho, token (que pode ser vazio).
    if (fields.size() != 3 || fields.at(0) != QLatin1String(kPrefix)) {
        return std::nullopt;
    }
    if (fields.at(1).trimmed().isEmpty()) {
        return std::nullopt;
    }
    return Request{.workspacePath = fields.at(1), .activationToken = fields.at(2)};
}

bool sameWorkspace(const QString& first, const QString& second)
{
    const auto normalize = [](const QString& raw) {
        QString path = QDir::cleanPath(raw.trimmed());
        while (path.size() > 1 && path.endsWith(QLatin1Char('/'))) {
            path.chop(1);
        }
        return path;
    };
    const QString a = normalize(first);
    const QString b = normalize(second);
    return !a.isEmpty() && a == b;
}

QString mineAnswer()
{
    return QStringLiteral("%1\n").arg(QLatin1String(kSim));
}

QString notMineAnswer()
{
    return QStringLiteral("%1\n").arg(QLatin1String(kNao));
}

bool answerIsMine(const QString& answer)
{
    return answer.trimmed() == QLatin1String(kSim);
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
