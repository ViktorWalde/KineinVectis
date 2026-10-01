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
    QTemporaryDir cache;
    QVERIFY(cache.isValid());
    qputenv("XDG_CACHE_HOME", cache.path().toUtf8());
    QCOMPARE(kinein::diagnosticLogPath(),
             cache.path() + QStringLiteral("/kinein-vectis/logs/kinein-ui-erros.txt"));

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
