#include "cli_args.h"

#include <QDir>
#include <QFileInfo>

namespace kinein::cli {

namespace {

// O `--` separa nossos argumentos dos do Qt. Depois dele, nada e' pasta.
constexpr auto kSeparador = "--";

Argumentos recusa(const QString& mensagem)
{
    return Argumentos{.acao = Acao::Recusa, .pasta = QString{}, .mensagem = mensagem};
}

} // namespace

QString textoDeAjuda()
{
    return QStringLiteral("kinein-vectis — IDE para C, C++, Rust e Python com embarcados.\n"
                          "\n"
                          "uso:\n"
                          "  kinein                 abre a pasta atual como projeto\n"
                          "  kinein .               idem, explicito\n"
                          "  kinein <pasta>         abre essa pasta (caminho relativo vale)\n"
                          "  kinein --help          mostra isto e sai, sem subir a IDE\n"
                          "  kinein --version       mostra a versao e sai, sem subir a IDE\n"
                          "\n"
                          "Pasta vazia abre normalmente: a IDE nao exige manifesto nem cria nada\n"
                          "por conta propria. Caminho que nao existe e' recusado com o motivo, em\n"
                          "vez de abrir sem projeto.\n");
}

Argumentos interpretar(const QStringList& argumentos, const QString& diretorioAtual)
{
    QString pasta;
    for (const QString& bruto : argumentos) {
        if (bruto == QLatin1String(kSeparador)) {
            break;
        }
        if (bruto == QLatin1String("--help") || bruto == QLatin1String("-h")) {
            return Argumentos{.acao = Acao::Ajuda, .pasta = QString{}, .mensagem = textoDeAjuda()};
        }
        if (bruto == QLatin1String("--version") || bruto == QLatin1String("-V")) {
            return Argumentos{.acao = Acao::Versao, .pasta = QString{}, .mensagem = QString{}};
        }
        if (bruto.startsWith(QLatin1Char('-'))) {
            // Opcao desconhecida e' recusa, nao algo a ignorar: ignorar faria a
            // IDE abrir fingindo que entendeu o que a pessoa pediu.
            return recusa(
                QStringLiteral("nao conheco a opcao `%1`. Veja `kinein --help`.").arg(bruto));
        }
        if (!pasta.isEmpty()) {
            // Varias raizes e' decisao aberta na §7 da especificacao, nao um
            // caso a resolver por conta propria aqui.
            return recusa(
                QStringLiteral("dois caminhos de uma vez (`%1` e `%2`), e a IDE abre uma pasta "
                               "por janela. Abra a segunda numa janela nova.")
                    .arg(pasta, bruto));
        }
        pasta = bruto;
    }

    if (pasta.isEmpty()) {
        return Argumentos{.acao = Acao::SemPasta, .pasta = QString{}, .mensagem = QString{}};
    }
    // `QDir` resolve `.`, `..` e relativo contra o diretorio de onde a pessoa
    // chamou, que e' o que um comando de terminal tem de fazer.
    const QString absoluta = QDir::isAbsolutePath(pasta)
                                 ? QDir::cleanPath(pasta)
                                 : QDir::cleanPath(QDir(diretorioAtual).absoluteFilePath(pasta));
    return Argumentos{.acao = Acao::Abrir, .pasta = absoluta, .mensagem = QString{}};
}

QString validarPasta(const QString& pasta)
{
    const QFileInfo info{pasta};
    if (!info.exists()) {
        // NAO criar: quem pede para abrir nao pediu para criar, e um erro de
        // digitacao criaria lixo com nome errado.
        return QStringLiteral("nao existe: %1").arg(pasta);
    }
    if (!info.isDir()) {
        return QStringLiteral("isto e' um arquivo, e a IDE abre uma PASTA: %1").arg(pasta);
    }
    if (!info.isReadable()) {
        return QStringLiteral("sem permissao de leitura em: %1").arg(pasta);
    }
    return QString{};
}

} // namespace kinein::cli
