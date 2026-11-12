// A leitura das opcoes `grafanaWebView` e `platformioWebView` antes do QGuiApplication (16.6,
// DocsPublic/roadmaps/59 §7.1.6): ausente, ligada no global e o arquivo que o
// core nao leria. Desde o INF-14 (2026-11-12), so' o global conta, como no
// core, e com as regras dele: o FIFO no lugar do arquivo prendia a partida no
// `open`, e o arquivo sem `schemaVersion` ligava o contexto.

#include "startup_web.h"

#include <QDir>
#include <QFile>
#include <QProcess>
#include <QTemporaryDir>
#include <QTest>

#ifdef Q_OS_UNIX
#include <chrono>
#include <fcntl.h>
#include <future>
#include <sys/stat.h>
#include <unistd.h>
#endif

namespace {

constexpr qint64 kReadLimit = 1024 * 1024;

// Um corpo valido com a opcao ligada, completado com espacos ate' `size`.
QByteArray padded(qint64 size)
{
    QByteArray body("{\"schemaVersion\":1,\"grafanaWebView\":true}");
    body.append(QByteArray(size - body.size(), ' '));
    return body;
}

} // namespace

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

    // Um link de pasta: `ln -s` no Unix, a juncao no Windows, que nao pede
    // privilegio (60 §3.3), como nos testes do core.
    static bool linkFolder(const QString& target, const QString& link)
    {
#ifdef Q_OS_WIN
        return QProcess::execute(QStringLiteral("cmd"),
                                 {QStringLiteral("/D"), QStringLiteral("/C"),
                                  QStringLiteral("mklink"), QStringLiteral("/J"),
                                  QDir::toNativeSeparators(link),
                                  QDir::toNativeSeparators(target)}) == 0;
#else
        return QProcess::execute(QStringLiteral("ln"), {QStringLiteral("-s"), target, link}) == 0;
#endif
    }

private slots:
    void missingFilesMeanOff()
    {
        QVERIFY(!kinein::wantsWebView(m_dir.filePath("nada.json")));
        QVERIFY(!kinein::wantsWebView(QString()));
    }

    void theGlobalOptionTurnsItOn()
    {
        const QString global =
            write("global.json", "{\"schemaVersion\":1,\"grafanaWebView\":true}");
        QVERIFY(kinein::wantsWebView(global));
        const QString off =
            write("global-off.json", "{\"schemaVersion\":1,\"grafanaWebView\":false}");
        QVERIFY(!kinein::wantsWebView(off));
    }

    void thePlatformioOptionTurnsItOnToo()
    {
        const QString global =
            write("global-pio.json", "{\"schemaVersion\":1,\"platformioWebView\":true}");
        QVERIFY(kinein::wantsWebView(global));
        const QString both =
            write("global-ambas.json",
                  "{\"schemaVersion\":1,\"grafanaWebView\":false,\"platformioWebView\":true}");
        QVERIFY(kinein::wantsWebView(both));
    }

    // Os motivos do core (`settings/file.rs`), cada um com a opcao ligada.
    void whatTheCoreWouldNotReadMeansOff()
    {
        const QList<QPair<QString, QByteArray>> cases = {
            {"quebrado.json", "{ grafanaWebView: true"},
            {"texto.json", "{\"schemaVersion\":1,\"grafanaWebView\":\"sim\"}"},
            {"sem-versao.json", "{\"grafanaWebView\":true}"},
            {"versao-2.json", "{\"schemaVersion\":2,\"grafanaWebView\":true}"},
            {"versao-texto.json", "{\"schemaVersion\":\"1\",\"grafanaWebView\":true}"},
            {"lista.json", "[{\"schemaVersion\":1,\"grafanaWebView\":true}]"},
            {"nao-utf8.json", "{\"schemaVersion\":1,\"grafanaWebView\":true,\"x\":\"\xff\xfe\"}"},
        };
        for (const auto& [name, body] : cases) {
            QVERIFY2(!kinein::wantsWebView(write(name, body)), qPrintable(name));
        }
    }

    // O limite de leitura do core: 1 MiB passa, um byte a mais nao.
    void theReadLimitIsTheCoresOne()
    {
        QVERIFY(kinein::wantsWebView(write("no-limite.json", padded(kReadLimit))));
        QVERIFY(!kinein::wantsWebView(write("acima.json", padded(kReadLimit + 1))));
    }

    void aFolderInPlaceOfTheFileMeansOff()
    {
        QVERIFY(QDir(m_dir.path()).mkpath(QStringLiteral("pasta/settings.json")));
        QVERIFY(!kinein::wantsWebView(m_dir.filePath("pasta/settings.json")));
    }

    // O global segue o link, como no core: a pasta `kinein-vectis` ligada a
    // outra, com o arquivo dentro.
    void theGlobalFollowsALinkedFolder()
    {
        QVERIFY(QDir(m_dir.path()).mkpath(QStringLiteral("real")));
        write("real/settings.json", "{\"schemaVersion\":1,\"grafanaWebView\":true}");
        QVERIFY(linkFolder(m_dir.filePath("real"), m_dir.filePath("ligada")));
        QVERIFY(kinein::wantsWebView(m_dir.filePath("ligada/settings.json")));
    }

#ifdef Q_OS_UNIX
    // O FIFO no lugar do arquivo (no Windows o caminho nao tem FIFO): a
    // leitura volta desligada sem abrir. Se ela ficasse presa no `open`, o
    // prazo reprova, e quem abre o outro lado a solta.
    void aFifoDoesNotHoldTheStartup()
    {
        const QByteArray fifo = m_dir.filePath("fifo.json").toLocal8Bit();
        QCOMPARE(::mkfifo(fifo.constData(), 0600), 0);
        std::future<bool> wants = std::async(std::launch::async, [path = QString::fromLocal8Bit(fifo)] {
            return kinein::wantsWebView(path);
        });
        if (wants.wait_for(std::chrono::seconds(5)) != std::future_status::ready) {
            const int writer = ::open(fifo.constData(), O_WRONLY);
            ::close(writer);
            QFAIL("a leitura ficou presa no open do FIFO");
        }
        QVERIFY(!wants.get());
    }
#endif

    void theGlobalFileFollowsXdgConfigHome()
    {
        qputenv("XDG_CONFIG_HOME", m_dir.path().toUtf8());
        QCOMPARE(kinein::globalSettingsFile(), m_dir.filePath("kinein-vectis/settings.json"));
        qputenv("XDG_CONFIG_HOME", "relativo");
        QVERIFY(kinein::globalSettingsFile().endsWith("/.config/kinein-vectis/settings.json"));
    }
};

QTEST_GUILESS_MAIN(StartupWebTest)
#ifdef __clang__
// O moc do Qt 6.12 usa CTAD; mesma fronteira dos demais testes (52 §14.50).
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_startup_web.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
