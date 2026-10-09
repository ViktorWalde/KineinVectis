#include "single_instance.h"

#include <QCryptographicHash>
#include <QDir>

// O PROTOCOLO da instancia unica, igual nos dois sistemas. O transporte mora
// num arquivo por sistema (`single_instance_unix.cpp`,
// `single_instance_windows.cpp`).
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

} // namespace kinein
