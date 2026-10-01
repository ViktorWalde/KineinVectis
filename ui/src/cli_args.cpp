#include "cli_args.h"

#include <QDir>
#include <QFileInfo>

namespace kinein::cli {

namespace {

// O `--` separa nossos argumentos dos do Qt. Depois dele, nada e' pasta.
constexpr auto kSeparator = "--";

Arguments refusal(const QString& message)
{
    return Arguments{.action = Action::Refusal, .folder = QString{}, .message = message};
}

} // namespace

QString helpText()
{
    return QStringLiteral(
        "kinein-vectis — IDE para C, C++, Rust e Python com embarcados.\n"
        "\n"
        "uso:\n"
        "  kinein                 abre a pasta atual como projeto\n"
        "  kinein .               idem, explicito\n"
        "  kinein <pasta>         abre essa pasta (caminho relativo vale)\n"
        "  kinein --help          mostra isto e sai, sem subir a IDE\n"
        "  kinein --version       mostra a versao e sai, sem subir a IDE\n"
        "  kinein -w, --wait      espera a janela fechar antes de devolver\n"
        "                         o terminal (para usar como editor)\n"
        "  kinein --verbose       mantem o terminal ligado e mostra o log\n"
        "                         da IDE (diagnostico)\n"
        "\n"
        "Pasta vazia abre normalmente: a IDE nao exige manifesto nem cria nada\n"
        "por conta propria. Caminho que nao existe e' recusado com o motivo, em\n"
        "vez de abrir sem projeto.\n"
        "\n"
        "Pelo terminal, a IDE abre e o prompt volta na hora, sem imprimir nada.\n");
}

bool shouldDetach(const Arguments& request, bool stderrIsTerminal, bool alreadyDetached)
{
    const bool opensWindow = request.action == Action::Open || request.action == Action::NoFolder;
    return opensWindow && stderrIsTerminal && !alreadyDetached && !request.waitForClose &&
           !request.verbose;
}

Arguments parse(const QStringList& arguments, const QString& currentDirectory)
{
    QString folder;
    bool waitForClose = false;
    bool verbose = false;
    for (const QString& raw : arguments) {
        if (raw == QLatin1String(kSeparator)) {
            break;
        }
        if (raw == QLatin1String("--help") || raw == QLatin1String("-h")) {
            return Arguments{.action = Action::Help, .folder = QString{}, .message = helpText()};
        }
        if (raw == QLatin1String("--version") || raw == QLatin1String("-V")) {
            return Arguments{.action = Action::Version, .folder = QString{}, .message = QString{}};
        }
        if (raw == QLatin1String("--wait") || raw == QLatin1String("-w")) {
            waitForClose = true;
            continue;
        }
        if (raw == QLatin1String("--verbose")) {
            verbose = true;
            continue;
        }
        if (raw.startsWith(QLatin1Char('-'))) {
            // Opcao desconhecida e' recusa, nao algo a ignorar: ignorar faria a
            // IDE abrir fingindo que entendeu o que a pessoa pediu.
            return refusal(
                QStringLiteral("nao conheco a opcao `%1`. Veja `kinein --help`.").arg(raw));
        }
        if (!folder.isEmpty()) {
            // Varias raizes e' decisao aberta na §7 da especificacao, nao um
            // caso a resolver por conta propria aqui.
            return refusal(
                QStringLiteral("dois caminhos de uma vez (`%1` e `%2`), e a IDE abre uma pasta "
                               "por janela. Abra a segunda numa janela nova.")
                    .arg(folder, raw));
        }
        folder = raw;
    }

    if (folder.isEmpty()) {
        return Arguments{.action = Action::NoFolder,
                         .folder = QString{},
                         .message = QString{},
                         .waitForClose = waitForClose,
                         .verbose = verbose};
    }
    // `QDir` resolve `.`, `..` e relativo contra o diretorio de onde a pessoa
    // chamou, que e' o que um comando de terminal tem de fazer.
    const QString absolute = QDir::isAbsolutePath(folder)
                                 ? QDir::cleanPath(folder)
                                 : QDir::cleanPath(QDir(currentDirectory).absoluteFilePath(folder));
    return Arguments{.action = Action::Open,
                     .folder = absolute,
                     .message = QString{},
                     .waitForClose = waitForClose,
                     .verbose = verbose};
}

QString validateFolder(const QString& folder)
{
    const QFileInfo info{folder};
    if (!info.exists()) {
        // NAO criar: quem pede para abrir nao pediu para criar, e um erro de
        // digitacao criaria lixo com nome errado.
        return QStringLiteral("nao existe: %1").arg(folder);
    }
    if (!info.isDir()) {
        return QStringLiteral("isto e' um arquivo, e a IDE abre uma PASTA: %1").arg(folder);
    }
    if (!info.isReadable()) {
        return QStringLiteral("sem permissao de leitura em: %1").arg(folder);
    }
    return QString{};
}

} // namespace kinein::cli
