#include "qt_message_log.h"

#include <QFile>
#include <QStandardPaths>
#include <QTemporaryDir>
#include <QTest>

// A rede de seguranca grava aviso no arquivo de diagnostico e NAO grava
// debug/info (roadmap 53 §5.1).
class TestQtMessageLog : public QObject
{
    Q_OBJECT

private slots:
    void warning_goes_to_file_and_debug_does_not();
};

void TestQtMessageLog::warning_goes_to_file_and_debug_does_not()
{
#ifdef Q_OS_WIN
    // O Windows nao le `XDG_CACHE_HOME`: a pasta de cache vem da API do
    // sistema, e o teste escreveria no cache REAL do usuario. O modo de teste
    // do Qt a desvia para uma pasta so' de teste (60 §3.2, W3b).
    QStandardPaths::setTestModeEnabled(true);
    const QString base = QStandardPaths::writableLocation(QStandardPaths::GenericCacheLocation);
    QFile::remove(base + QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt"));
#else
    QTemporaryDir cache;
    QVERIFY(cache.isValid());
    qputenv("XDG_CACHE_HOME", cache.path().toUtf8());
    const QString base = cache.path();
#endif
    QCOMPARE(kinein::diagnosticLogPath(),
             base + QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt"));

    kinein::installQtMessageLog();
    qWarning("aviso de teste 1234");
    qDebug("debug de teste 5678");

    QFile file(kinein::diagnosticLogPath());
    QVERIFY(file.open(QIODevice::ReadOnly));
    const QString text = QString::fromUtf8(file.readAll());
    QVERIFY2(text.contains(QStringLiteral("qt-aviso aviso de teste 1234")), qPrintable(text));
    QVERIFY(!text.contains(QStringLiteral("5678")));
}

QTEST_GUILESS_MAIN(TestQtMessageLog)
// O moc gerado usa CTAD que o `-Wctad-maybe-unsupported` do Clang 21 reprova
// sob `-Werror` (mesma isencao do tst_cli_args, presa ao include gerado).
#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_qt_message_log.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
