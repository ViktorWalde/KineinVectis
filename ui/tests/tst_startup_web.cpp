// A leitura da opcao `grafanaWebView` antes do QGuiApplication (16.6,
// DocsPublic/roadmaps/59 §7.1.6): ausente, global, o projeto vencendo e o
// arquivo invalido.

#include "startup_web.h"

#include <QFile>
#include <QTemporaryDir>
#include <QTest>

class StartupWebTest : public QObject
{
    Q_OBJECT

private:
    QTemporaryDir m_dir;

    QString write(const QString& name, const QByteArray& body)
    {
        const QString path = m_dir.filePath(name);
        QFile file(path);
        if (file.open(QIODevice::WriteOnly)) {
            file.write(body);
        }
        return path;
    }

private slots:
    void missingFilesMeanOff()
    {
        QVERIFY(!kinein::wantsWebView(m_dir.filePath("nada.json"), QString()));
    }

    void theGlobalOptionTurnsItOn()
    {
        const QString global =
            write("global.json", "{\"schemaVersion\":1,\"grafanaWebView\":true}");
        QVERIFY(kinein::wantsWebView(global, QString()));
        QVERIFY(kinein::wantsWebView(global, m_dir.filePath("sem-projeto.json")));
    }

    void theProjectWins()
    {
        const QString global =
            write("global2.json", "{\"schemaVersion\":1,\"grafanaWebView\":true}");
        const QString off =
            write("projeto-off.json", "{\"schemaVersion\":1,\"grafanaWebView\":false}");
        QVERIFY(!kinein::wantsWebView(global, off));
        const QString on =
            write("projeto-on.json", "{\"schemaVersion\":1,\"grafanaWebView\":true}");
        QVERIFY(kinein::wantsWebView(m_dir.filePath("nada.json"), on));
    }

    void invalidFilesMeanOff()
    {
        const QString broken = write("quebrado.json", "{ grafanaWebView: true");
        QVERIFY(!kinein::wantsWebView(broken, QString()));
        const QString text = write("texto.json", "{\"grafanaWebView\":\"sim\"}");
        QVERIFY(!kinein::wantsWebView(text, QString()));
    }

    void theGlobalFileFollowsXdgConfigHome()
    {
        qputenv("XDG_CONFIG_HOME", m_dir.path().toUtf8());
        QCOMPARE(kinein::globalSettingsFile(), m_dir.filePath("kinein-vectis/settings.json"));
        qputenv("XDG_CONFIG_HOME", "relativo");
        QVERIFY(kinein::globalSettingsFile().endsWith("/.config/kinein-vectis/settings.json"));
    }
};

QTEST_GUILESS_MAIN(StartupWebTest)
#include "tst_startup_web.moc"
