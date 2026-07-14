// Entry point of the Kinein Vectis UI process.

#include <QElapsedTimer>
#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
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

} // namespace

int main(int argc, char* argv[])
{
    QElapsedTimer perfTimer;
    perfTimer.start();

    QGuiApplication app(argc, argv);
    QGuiApplication::setApplicationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setOrganizationName(QStringLiteral("Kinein Vectis"));
    QGuiApplication::setApplicationVersion(QStringLiteral("0.1.0"));
    QGuiApplication::setWindowIcon(QIcon(QStringLiteral(":/KineinVectis/assets/app-icon.png")));

    QQmlApplicationEngine engine;
    // Qt 6.4: qt_add_qml_module places the module under qrc:/ (no /qt/qml prefix).
    engine.addImportPath(QStringLiteral("qrc:/"));
    QObject::connect(
        &engine, &QQmlApplicationEngine::objectCreationFailed, &app,
        []() { QCoreApplication::exit(1); }, Qt::QueuedConnection);
    engine.load(QUrl(QStringLiteral("qrc:/KineinVectis/qml/Main.qml")));

    installStartupPerfMarker(app, engine, perfTimer);

    return QGuiApplication::exec();
}
