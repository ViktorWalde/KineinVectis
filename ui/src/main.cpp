// Entry point of the Kinein Vectis UI process.

#include "typing_perf_harness.h"

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
    QElapsedTimer perfTimer;
    perfTimer.start();

    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setOrganizationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));

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
