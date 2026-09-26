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
constexpr auto kPrefixo = "KINEIN-INSTANCIA-1";
constexpr auto kSim = "MEU";
constexpr auto kNao = "NAO";

/// O separador dos campos. TAB porque caminho de arquivo pode ter espaco —
/// e tem, no teste e na vida.
constexpr QChar kSeparador = QLatin1Char('\t');

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
    QString caminho = workspacePath;
    QString token = activationToken;
    caminho.remove(QLatin1Char('\t')).remove(QLatin1Char('\n'));
    token.remove(QLatin1Char('\t')).remove(QLatin1Char('\n'));
    return QStringLiteral("%1\t%2\t%3\n").arg(QLatin1String(kPrefixo), caminho, token);
}

std::optional<Request> parseRequest(const QString& line)
{
    QString limpa = line;
    while (limpa.endsWith(QLatin1Char('\n')) || limpa.endsWith(QLatin1Char('\r'))) {
        limpa.chop(1);
    }
    const QStringList campos = limpa.split(kSeparador);
    // Tres campos exatos: prefixo, caminho, token (que pode ser vazio).
    if (campos.size() != 3 || campos.at(0) != QLatin1String(kPrefixo)) {
        return std::nullopt;
    }
    if (campos.at(1).trimmed().isEmpty()) {
        return std::nullopt;
    }
    return Request{.workspacePath = campos.at(1), .activationToken = campos.at(2)};
}

bool sameWorkspace(const QString& um, const QString& outro)
{
    const auto normalizar = [](const QString& bruto) {
        QString caminho = QDir::cleanPath(bruto.trimmed());
        while (caminho.size() > 1 && caminho.endsWith(QLatin1Char('/'))) {
            caminho.chop(1);
        }
        return caminho;
    };
    const QString a = normalizar(um);
    const QString b = normalizar(outro);
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
sockaddr* comoGenerico(sockaddr_un& endereco)
{
    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
    return reinterpret_cast<sockaddr*>(&endereco);
}

bool preencherEndereco(sockaddr_un& endereco, const QString& socketPath)
{
    const QByteArray bytes = socketPath.toUtf8();
    if (bytes.isEmpty() || static_cast<std::size_t>(bytes.size()) >= sizeof(endereco.sun_path)) {
        return false;
    }
    endereco = {};
    endereco.sun_family = AF_UNIX;
    std::memcpy(static_cast<void*>(endereco.sun_path), bytes.constData(),
                static_cast<std::size_t>(bytes.size()));
    return true;
}

/// Um tempo limite curto nos dois sentidos.
///
/// Sem isto, um dono TRAVADO — nao morto, travado — seguraria o terminal de
/// quem so' quis abrir um projeto. Preferimos abrir uma janela a mais.
void limitarTempo(int fd, int timeoutMs)
{
    timeval prazo{};
    prazo.tv_sec = timeoutMs / 1000;
    prazo.tv_usec = static_cast<suseconds_t>(timeoutMs % 1000) * 1000;
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &prazo, sizeof(prazo));
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &prazo, sizeof(prazo));
}

/// Le' ate' a primeira quebra de linha, ou ate' o limite.
QString lerLinha(int fd, int limite = 8192)
{
    QByteArray acumulado;
    std::array<char, 512> pedaco{};
    while (acumulado.size() < limite) {
        const ssize_t lidos = ::read(fd, pedaco.data(), pedaco.size());
        if (lidos <= 0) {
            break;
        }
        acumulado.append(pedaco.data(), static_cast<int>(lidos));
        if (acumulado.contains('\n')) {
            break;
        }
    }
    return QString::fromUtf8(acumulado);
}

bool escrever(int fd, const QString& texto)
{
    const QByteArray bytes = texto.toUtf8();
    // `span` em vez de somar no ponteiro: e' o mesmo remedio que o `main` usa
    // com o `argv`, e pelo mesmo motivo — somar no ponteiro a mao e' o lugar
    // classico de escrever um byte a mais.
    std::span<const char> restante{bytes.constData(), static_cast<std::size_t>(bytes.size())};
    while (!restante.empty()) {
        const ssize_t agora = ::write(fd, restante.data(), restante.size());
        if (agora <= 0) {
            return false;
        }
        restante = restante.subspan(static_cast<std::size_t>(agora));
    }
    return true;
}

} // namespace

bool handOff(const QString& socketPath, const QString& workspacePath,
             const QString& activationToken, int timeoutMs)
{
    sockaddr_un endereco{};
    if (socketPath.isEmpty() || !preencherEndereco(endereco, socketPath)) {
        return false;
    }
    const int fd = ::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) {
        return false;
    }
    limitarTempo(fd, timeoutMs);
    if (::connect(fd, comoGenerico(endereco), sizeof(endereco)) != 0) {
        // Ninguem atende: ou nunca houve, ou o dono morreu. Nos dois casos
        // quem chegou agora e' o dono.
        ::close(fd);
        return false;
    }
    const bool mandou = escrever(fd, encodeRequest(workspacePath, activationToken));
    const QString resposta = mandou ? lerLinha(fd) : QString{};
    ::close(fd);
    return answerIsMine(resposta);
}

int listenFor(const QString& socketPath)
{
    sockaddr_un endereco{};
    if (socketPath.isEmpty() || !preencherEndereco(endereco, socketPath)) {
        return -1;
    }
    const QFileInfo informacao{socketPath};
    QDir pasta = informacao.dir();
    if (!pasta.exists() && !pasta.mkpath(QStringLiteral("."))) {
        return -1;
    }
    // ORFAO: o arquivo existe e ninguem atende. E' o que sobra de um crash, e
    // deixar a coordenacao quebrada ate' o proximo reboot seria pior.
    if (informacao.exists()) {
        // VIVO OU ORFAO, decidido por `connect` e mais nada. A primeira versao
        // disto mandava um pedido de verdade para sondar, o que jogava lixo no
        // fio de um dono legitimo — e nao respondia melhor a pergunta.
        const int teste = ::socket(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
        if (teste >= 0) {
            limitarTempo(teste, 150);
            const bool vivo = ::connect(teste, comoGenerico(endereco), sizeof(endereco)) == 0;
            ::close(teste);
            if (vivo) {
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
    if (::bind(fd, comoGenerico(endereco), sizeof(endereco)) != 0 || ::listen(fd, 4) != 0) {
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
    const int cliente = ::accept4(listenFd, nullptr, nullptr, SOCK_CLOEXEC);
    if (cliente < 0) {
        return std::nullopt;
    }
    limitarTempo(cliente, 400);
    std::optional<Request> pedido = parseRequest(lerLinha(cliente));
    if (!pedido.has_value() || !sameWorkspace(pedido->workspacePath, myWorkspacePath)) {
        // QUEM NAO E' DAQUI RECEBE UM NAO, e nao o silencio: o outro lado
        // esperaria o tempo inteiro do prazo antes de seguir.
        escrever(cliente, notMineAnswer());
        ::close(cliente);
        return std::nullopt;
    }
    escrever(cliente, mineAnswer());
    ::close(cliente);
    return pedido;
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
