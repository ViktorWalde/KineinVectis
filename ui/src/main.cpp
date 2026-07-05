// Entry point of the Kinein Vectis UI process.

#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QUrl>

int main(int argc, char* argv[])
{
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

    return QGuiApplication::exec();
}
