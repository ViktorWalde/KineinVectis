// O aplicativo da prova: uma janela Qt Quick com o NvimView, o supervisor
// como processo filho e, para o driver dos testes, o canal de controle.
//
// Uso: kinein-prova-ui --controle <socket> -- <comando do supervisor...>

#include "nvim_view.h"
#include "prova_control.h"
#include "supervisor_link.h"

#include <QElapsedTimer>
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>

int main(int argc, char* argv[])
{
    QElapsedTimer clock;
    clock.start();
    QGuiApplication app(argc, argv);

    QStringList args = QCoreApplication::arguments();
    QString controlPath;
    QStringList supervisor;
    const qsizetype dash = args.indexOf(QStringLiteral("--"));
    if (dash >= 0) {
        supervisor = args.mid(dash + 1);
        args = args.first(dash);
    }
    const qsizetype c = args.indexOf(QStringLiteral("--controle"));
    if (c >= 0 && c + 1 < args.size()) {
        controlPath = args.at(c + 1);
    }

    // Antes do QML: com a acessibilidade ativa, a carga já cria (e guarda no
    // cache do QAccessible) a interface de cada item; a do editor tem de sair
    // desta fábrica, não da genérica do Qt Quick.
    prova::installNvimAccessibility();

    prova::SupervisorLink link;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("supervisor"), &link);
    engine.loadFromModule("KineinProva", "Main");
    if (engine.rootObjects().isEmpty()) {
        return 1;
    }
    auto* window = qobject_cast<QQuickWindow*>(engine.rootObjects().constFirst());
    auto* view = window->findChild<prova::NvimView*>(QStringLiteral("editor"));
    if (view == nullptr) {
        return 1;
    }
    prova::ProvaControl control(window, view, &link, &clock);
    if (!controlPath.isEmpty() && !control.listen(controlPath)) {
        qWarning("canal de controle indisponível: %s", qPrintable(controlPath));
        return 1;
    }
    QObject::connect(&app, &QCoreApplication::aboutToQuit, &link, &prova::SupervisorLink::stop);
    link.start(supervisor);
    return QGuiApplication::exec();
}
