// Entry point of the Kinein Vectis UI process.

#include "cli_args.h"
#include "typing_perf_harness.h"
#include <QTextStream>
#include <span>

#include <QDir>
#include <QElapsedTimer>
#include <QGuiApplication>
#include <QImage>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>
#include <QUrl>

namespace {

// M4.2: marcador de startup GATED por env, 100% local (stderr, sem
// telemetria). Com KINEIN_PERF_MARKER setado, imprime o tempo do início do
// processo até o PRIMEIRO frame renderizado; com KINEIN_PERF_EXIT, sai em
// seguida (para o scripts/medir-performance.sh coletar e encerrar). Sem a
// env, zero efeito no uso normal.
void installStartupPerfMarker(QGuiApplication& app, QQmlApplicationEngine& engine,
                              const QElapsedTimer& perfTimer)
{
    if (!qEnvironmentVariableIsSet("KINEIN_PERF_MARKER") || engine.rootObjects().isEmpty()) {
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        return;
    }
    const bool exitAfter = qEnvironmentVariableIsSet("KINEIN_PERF_EXIT");
    QObject::connect(
        window, &QQuickWindow::frameSwapped, &app,
        [&perfTimer, exitAfter]() {
            qInfo().noquote().nospace() << "KINEIN_PERF first_frame_ms=" << perfTimer.elapsed();
            if (exitAfter) {
                QCoreApplication::quit();
            }
        },
        Qt::SingleShotConnection);
}

// Etapa 2 (2026-09-18): uma FOTO da janela, gated por env, para o desenho
// medido e para o gate ver a IDE com projeto aberto sem tela. Com
// KINEIN_SCREENSHOT=<arquivo.png> a janela e' capturada KINEIN_SCREENSHOT_DELAY_MS
// (padrao 4000) depois do primeiro frame; com KINEIN_PERF_EXIT sai em
// seguida. Sem a env, zero efeito.
void installScreenshotHook(QGuiApplication& app, QQmlApplicationEngine& engine)
{
    const QByteArray destino = qgetenv("KINEIN_SCREENSHOT");
    if (destino.isEmpty() || engine.rootObjects().isEmpty()) {
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        return;
    }
    bool ok = false;
    int atraso = qEnvironmentVariableIntValue("KINEIN_SCREENSHOT_DELAY_MS", &ok);
    if (!ok || atraso < 0) {
        atraso = 4000;
    }
    // KINEIN_SCREENSHOT_SIZE=<largura>x<altura> (fechamento da Etapa 2,
    // 2026-09-19): a foto num tamanho que nao e' o padrao — a status bar a
    // 1024 px foi a divida dita na F2. Sem a env, a janela fica como esta'.
    const QByteArray tamanho = qgetenv("KINEIN_SCREENSHOT_SIZE");
    if (!tamanho.isEmpty()) {
        const QList<QByteArray> partes = tamanho.split('x');
        if (partes.size() == 2) {
            const int largura = partes[0].toInt();
            const int altura = partes[1].toInt();
            if (largura >= 640 && altura >= 400) {
                window->resize(largura, altura);
            }
        }
    }
    const bool exitAfter = qEnvironmentVariableIsSet("KINEIN_PERF_EXIT");
    const QString caminho = QString::fromUtf8(destino);
    QObject::connect(
        window, &QQuickWindow::frameSwapped, &app,
        [window, caminho, atraso, exitAfter]() {
            QTimer::singleShot(atraso, window, [window, caminho, exitAfter]() {
                const QImage imagem = window->grabWindow();
                const bool salvo = imagem.save(caminho);
                qInfo().noquote().nospace()
                    << "KINEIN_SCREENSHOT " << (salvo ? "salvo=" : "FALHOU=") << caminho << " "
                    << imagem.width() << "x" << imagem.height();
                if (exitAfter) {
                    QCoreApplication::quit();
                }
            });
        },
        Qt::SingleShotConnection);
}

} // namespace

int main(int argc, char* argv[])
{
    // O CONTRATO DA LINHA DE COMANDO vem ANTES de qualquer Qt: a §3 da
    // `especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md` exige que
    // `--help` e `--version` respondam "sem iniciar UI/core nem fazer rede".
    // Criar o QGuiApplication ja' seria iniciar a UI.
    // `std::span` em vez de indexar `argv` na mao: o clang-tidy recusa
    // aritmetica de ponteiro, e com razao — e' o lugar classico de ler um a mais.
    const std::span<char*> argumentos{argv, static_cast<std::size_t>(argc)};
    QStringList brutos;
    brutos.reserve(static_cast<qsizetype>(argumentos.size()) - 1);
    for (char* const bruto : argumentos.subspan(1)) {
        brutos.append(QString::fromLocal8Bit(bruto));
    }
    const kinein::cli::Argumentos pedido = kinein::cli::interpretar(brutos, QDir::currentPath());
    switch (pedido.acao) {
    case kinein::cli::Acao::Ajuda: {
        QTextStream saida{stdout};
        saida << pedido.mensagem;
        return 0;
    }
    case kinein::cli::Acao::Versao: {
        QTextStream saida{stdout};
        saida << QStringLiteral("kinein-vectis %1\n").arg(QLatin1String(KINEIN_VERSAO));
        return 0;
    }
    case kinein::cli::Acao::Recusa: {
        QTextStream erro{stderr};
        erro << pedido.mensagem << '\n';
        return 2;
    }
    case kinein::cli::Acao::Abrir: {
        // Caminho ruim e' recusa COM MOTIVO, e nao uma IDE que abre sem projeto
        // deixando a pessoa adivinhar. Nada e' criado.
        const QString motivo = kinein::cli::validarPasta(pedido.pasta);
        if (!motivo.isEmpty()) {
            QTextStream erro{stderr};
            erro << motivo << '\n';
            return 2;
        }
        break;
    }
    case kinein::cli::Acao::SemPasta:
        break;
    }

    QElapsedTimer perfTimer;
    perfTimer.start();

    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setOrganizationName(QStringLiteral("Kinein Vectis"));
    // Vem do CMake: escrita a mao, ela ficou em `0.1.0` enquanto o projeto
    // estava em 0.2.0 — e nada reprovava, porque ninguem a lia.
    QGuiApplication::setApplicationVersion(QLatin1String(KINEIN_VERSAO));

    QQmlApplicationEngine engine;
    // Qt 6.4: qt_add_qml_module places the module under qrc:/ (no /qt/qml prefix).
    engine.addImportPath(QStringLiteral("qrc:/"));
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        []() { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/KineinVectis/qml/Main.qml")));

    installStartupPerfMarker(app, engine, perfTimer);
    installScreenshotHook(app, engine);
    kinein::installTypingPerfHarness(app, engine);

    return QGuiApplication::exec();
}
