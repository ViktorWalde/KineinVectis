// Entry point of the Kinein Vectis UI process.

#include "cli_args.h"
#include "qt_message_log.h"

#include "single_instance.h"
#include "typing_perf_harness.h"
#include <QTextStream>
#include <fcntl.h>
#include <span>
#include <unistd.h>

#include <QDir>
#include <QElapsedTimer>
#include <QFile>
#include <QFileInfo>
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
//
// KINEIN_PERF_MARKER_FILE=<arquivo> grava a MESMA linha num arquivo (G0.5,
// 2026-10-01). O processo desacoplado do terminal tem stdout/stderr em
// /dev/null, e quem o testava so' via o PID sumir — e sumir tambem e' crash:
// com um abort() depois do engine.load o G0.5 imprimia "a IDE abre sozinha".
// O arquivo e' a unica prova de primeiro frame que atravessa o desacoplamento.
void installStartupPerfMarker(QGuiApplication& app, QQmlApplicationEngine& engine,
                              const QElapsedTimer& perfTimer)
{
    const bool printMarker = qEnvironmentVariableIsSet("KINEIN_PERF_MARKER");
    const QString markerFile = qEnvironmentVariable("KINEIN_PERF_MARKER_FILE");
    if ((!printMarker && markerFile.isEmpty()) || engine.rootObjects().isEmpty()) {
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        return;
    }
    const bool exitAfter = qEnvironmentVariableIsSet("KINEIN_PERF_EXIT");
    QObject::connect(
        window, &QQuickWindow::frameSwapped, &app,
        [&perfTimer, exitAfter, printMarker, markerFile]() {
            const QString line =
                QStringLiteral("KINEIN_PERF first_frame_ms=%1").arg(perfTimer.elapsed());
            if (printMarker) {
                qInfo().noquote() << line;
            }
            if (!markerFile.isEmpty()) {
                QFile file(markerFile);
                if (file.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text)) {
                    file.write(line.toUtf8() + '\n');
                }
            }
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
    const QByteArray screenshotEnv = qgetenv("KINEIN_SCREENSHOT");
    if (screenshotEnv.isEmpty() || engine.rootObjects().isEmpty()) {
        return;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    if (window == nullptr) {
        return;
    }
    bool ok = false;
    int delayMs = qEnvironmentVariableIntValue("KINEIN_SCREENSHOT_DELAY_MS", &ok);
    if (!ok || delayMs < 0) {
        delayMs = 4000;
    }
    // KINEIN_SCREENSHOT_SIZE=<largura>x<altura> (fechamento da Etapa 2,
    // 2026-09-19): a foto num tamanho que nao e' o padrao — a status bar a
    // 1024 px foi a divida dita na F2. Sem a env, a janela fica como esta'.
    const QByteArray sizeEnv = qgetenv("KINEIN_SCREENSHOT_SIZE");
    if (!sizeEnv.isEmpty()) {
        const QList<QByteArray> sizeParts = sizeEnv.split('x');
        if (sizeParts.size() == 2) {
            const int shotWidth = sizeParts[0].toInt();
            const int shotHeight = sizeParts[1].toInt();
            if (shotWidth >= 640 && shotHeight >= 400) {
                window->resize(shotWidth, shotHeight);
            }
        }
    }
    const bool exitAfter = qEnvironmentVariableIsSet("KINEIN_PERF_EXIT");
    const QString path = QString::fromUtf8(screenshotEnv);
    QObject::connect(
        window, &QQuickWindow::frameSwapped, &app,
        [window, path, delayMs, exitAfter]() {
            QTimer::singleShot(delayMs, window, [window, path, exitAfter]() {
                const QImage image = window->grabWindow();
                const bool saved = image.save(path);
                qInfo().noquote().nospace()
                    << "KINEIN_SCREENSHOT " << (saved ? "salvo=" : "FALHOU=") << path << " "
                    << image.width() << "x" << image.height();
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
    const std::span<char*> arguments{argv, static_cast<std::size_t>(argc)};
    QStringList rawArguments;
    rawArguments.reserve(static_cast<qsizetype>(arguments.size()) - 1);
    for (char* const raw : arguments.subspan(1)) {
        rawArguments.append(QString::fromLocal8Bit(raw));
    }
    const kinein::cli::Arguments request = kinein::cli::parse(rawArguments, QDir::currentPath());
    switch (request.action) {
    case kinein::cli::Action::Help: {
        QTextStream out{stdout};
        out << request.message;
        return 0;
    }
    case kinein::cli::Action::Version: {
        QTextStream out{stdout};
        out << QStringLiteral("kinein-vectis %1\n").arg(QLatin1String(KINEIN_VERSION));
        return 0;
    }
    case kinein::cli::Action::Refusal: {
        QTextStream err{stderr};
        err << request.message << '\n';
        return 2;
    }
    case kinein::cli::Action::Open: {
        // Caminho ruim e' recusa COM MOTIVO, e nao uma IDE que abre sem projeto
        // deixando a pessoa adivinhar. Nada e' criado.
        const QString reason = kinein::cli::validateFolder(request.folder);
        if (!reason.isEmpty()) {
            QTextStream err{stderr};
            err << reason << '\n';
            return 2;
        }
        // UMA JANELA POR PASTA (decisao do autor, 2026-09-26). Se ja' ha' uma
        // janela com ESTE projeto, ela e' avisada e este processo sai sem
        // abrir a segunda.
        //
        // NAO E' CONFORTO: nao ha' lock de workspace, e duas janelas na mesma
        // pasta sao duas donas do `.kinein/` — a ultima a fechar apaga o que a
        // outra gravou, sem erro e sem aviso. Medido em 2026-09-26.
        //
        // Antes do QGuiApplication de proposito: se alguem ja' responde por
        // esta pasta, nem a UI nem o core chegam a subir.
        const QString canonicalFolder = QFileInfo{request.folder}.canonicalFilePath();
        const QString socket =
            kinein::socketPathFor(qEnvironmentVariable("XDG_RUNTIME_DIR"), canonicalFolder);
        // O TOKEN DO XDG E' A UNICA AUTORIZACAO que o compositor Wayland
        // aceita para uma janela subir por pedido de outro processo. O
        // terminal o poe no ambiente quando sabe fazer isso; quando nao poe,
        // o pedido segue sem ele e a janela pode apenas piscar na barra.
        if (kinein::handOff(socket, canonicalFolder, qEnvironmentVariable("XDG_ACTIVATION_TOKEN")))
        {
            QTextStream out{stdout};
            out << QStringLiteral("este projeto já está aberto: %1\n").arg(canonicalFolder);
            return 0;
        }
        break;
    }
    case kinein::cli::Action::NoFolder:
        break;
    }

    // COMO `code .` (roadmap 53 §5.1): chamado de um terminal, o processo do
    // terminal solta a IDE e devolve o prompt na hora. Fica DEPOIS da
    // validacao e do encaminhamento, que sao as unicas respostas que pertencem
    // ao terminal (erro de caminho, "ja' aberto"). Antes do QGuiApplication de
    // proposito: nenhuma thread do Qt existe ainda, e o fork e' seguro.
    const bool alreadyDetached = qEnvironmentVariable("KINEIN_DETACHED") == QLatin1String("1");
    if (kinein::cli::shouldDetach(request, isatty(STDERR_FILENO) == 1, alreadyDetached)) {
        const pid_t child = fork();
        if (child > 0) {
            return 0;
        }
        if (child == 0) {
            setsid();
            qputenv("KINEIN_DETACHED", "1");
            // O filho nao tem mais terminal: entrada e saidas vao para
            // /dev/null. O que ainda escapar do Qt vai para o log de
            // diagnostico (installQtMessageLog, abaixo).
            // `open` e' variadica: e' a unica forma de obter o descritor
            // para o `dup2` antes de existir qualquer Qt.
            // NOLINTNEXTLINE(cppcoreguidelines-pro-type-vararg,hicpp-vararg)
            const int devNull = open("/dev/null", O_RDWR | O_CLOEXEC);
            if (devNull >= 0) {
                dup2(devNull, STDIN_FILENO);
                dup2(devNull, STDOUT_FILENO);
                dup2(devNull, STDERR_FILENO);
                close(devNull);
            }
        }
        // fork falhou (filho < 0): segue ligado ao terminal, como antes.
    }

    kinein::installQtMessageLog();

    QElapsedTimer perfTimer;
    perfTimer.start();

    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setOrganizationName(QStringLiteral("Kinein Vectis"));
    // Vem do CMake: escrita a mao, ela ficou em `0.1.0` enquanto o projeto
    // estava em 0.2.0 — e nada reprovava, porque ninguem a lia.
    QGuiApplication::setApplicationVersion(QLatin1String(KINEIN_VERSION));

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
