#include <QCoreApplication>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QTimer>
#include <QUrl>

int main(int argc, char** argv)
{
    QGuiApplication app(argc, argv);
    app.setQuitOnLastWindowClosed(false);
    QQmlApplicationEngine engine;
    engine.addImportPath(QStringLiteral("qrc:/"));
    engine.rootContext()->setContextProperty(
        QStringLiteral("proofWorkspace"), QString::fromLocal8Bit(qgetenv("KINEIN_PROOF_WORKSPACE")));
    QObject::connect(&engine, &QQmlEngine::quit, &app, &QCoreApplication::quit);
    QObject::connect(&engine, &QQmlEngine::exit, &app, &QCoreApplication::exit);
    engine.load(QUrl::fromLocalFile(QString::fromLocal8Bit(qgetenv("KINEIN_PROOF_QML"))));
    if (engine.rootObjects().isEmpty()) {
        return 2;
    }
    QTimer::singleShot(10000, &app, [] { QCoreApplication::exit(3); });
    return app.exec();
}
