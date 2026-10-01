#include "clipboard.h"

#include <QClipboard>
#include <QFile>
#include <QGuiApplication>
#include <QMimeData>
#include <QTemporaryDir>
#include <QUrl>
#include <QtTest>

class ClipboardFilesTest : public QObject
{
    Q_OBJECT

private slots:
    void localDropAcceptsOnlyLocalUrls()
    {
        kinein::Clipboard bridge;
        const QString first = QStringLiteral("/tmp/ação #1.txt");
        const QString second = QStringLiteral("/tmp/pasta com espaço");
        QCOMPARE(bridge.localFilePathsFromUrls(
                     QVariantList{QUrl::fromLocalFile(first), QUrl::fromLocalFile(second)}),
                 (QStringList{first, second}));
        QVERIFY(bridge.localFilePathsFromUrls(
                    QVariantList{QUrl::fromLocalFile(first), QUrl(QStringLiteral("https://site.test/x"))})
                    .isEmpty());
        QVERIFY(bridge.localFilePathsFromUrls(
                    QVariantList{QUrl(QStringLiteral("file://server.test/share/x"))})
                    .isEmpty());
        QVERIFY(bridge.localFilePathsFromUrls({}).isEmpty());
        QCOMPARE(bridge.localFileUrl(first).toLocalFile(), first);
        QVERIFY(bridge.localFileUrl(QStringLiteral("relative/path")).isEmpty());
    }

    void cutBatchKeepsOnlyUnmovedPaths()
    {
        const QTemporaryDir folder;
        QVERIFY(folder.isValid());
        const QString first = folder.path() + QStringLiteral("/um arquivo.txt");
        const QString second = folder.path() + QStringLiteral("/dois-ação.txt");
        const QStringList originals{first, second};

        kinein::Clipboard bridge;
        bridge.setFiles(originals, true);
        QCOMPARE(bridge.filePaths(), originals);
        QVERIFY(bridge.filesCut());
        const QMimeData* mime = QGuiApplication::clipboard()->mimeData();
        QVERIFY(mime != nullptr);
        QCOMPARE(mime->urls().size(), 2);
        QCOMPARE(mime->urls().at(0).toLocalFile(), first);
        QCOMPARE(mime->urls().at(1).toLocalFile(), second);
        QVERIFY(mime->hasFormat(QStringLiteral("x-special/gnome-copied-files")));

        bridge.removeCutFilesIfMatches(originals, QStringList{first});
        QCOMPARE(bridge.filePaths(), QStringList{second});
        QVERIFY(bridge.filesCut());

        // Uma resposta antiga não pode editar um clipboard que mudou.
        bridge.removeCutFilesIfMatches(originals, QStringList{second});
        QCOMPARE(bridge.filePaths(), QStringList{second});
        bridge.removeCutFilesIfMatches(QStringList{second}, QStringList{second});
        QVERIFY(!bridge.hasFiles());
    }

    void workspaceDropRequiresDirectory()
    {
        const QTemporaryDir folder;
        QVERIFY(folder.isValid());
        const QString file = folder.path() + QStringLiteral("/arquivo.txt");
        QFile handle(file);
        QVERIFY(handle.open(QIODevice::WriteOnly));
        handle.close();

        kinein::Clipboard bridge;
        QCOMPARE(bridge.localDirectoryPathFromUrls(QVariantList{QUrl::fromLocalFile(folder.path())}),
                 folder.path());
        QVERIFY(bridge.localDirectoryPathFromUrls(QVariantList{QUrl::fromLocalFile(file)}).isEmpty());
        QVERIFY(bridge.localDirectoryPathFromUrls(
                    QVariantList{QUrl::fromLocalFile(folder.path()), QUrl::fromLocalFile(file)})
                    .isEmpty());
    }
};

int main(int argc, char* argv[])
{
    QGuiApplication app(argc, argv);
    ClipboardFilesTest test;
    return QTest::qExec(&test, argc, argv);
}

#include "tst_clipboard_files.moc"
