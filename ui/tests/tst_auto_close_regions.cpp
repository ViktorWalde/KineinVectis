// O rastreio de posicao e' a logica mais sutil do auto-close (E6): um int nao
// sobrevive a uma edicao, e e' o QTextCursor quem acompanha o documento. Cada
// caso aqui fixa um comportamento de ajuste/invalidacao que, errado, engoliria
// ou duplicaria um caractere do usuario em silencio — a fatia E6 ficou
// BLOQUEADA ate existir onde escrever exatamente este arquivo (E5).

#include "auto_close_regions.h"

#include <QObject>
#include <QTest>
#include <QTextCursor>
#include <QTextDocument>

using kinein::AutoCloseRegions;

class TestAutoCloseRegions : public QObject
{
    Q_OBJECT

private slots:
    void registraEConsome()
    {
        QTextDocument doc(QStringLiteral("()"));
        AutoCloseRegions regions;
        regions.setDocument(&doc);

        regions.notePairInserted(1);
        QVERIFY(regions.isAutoClosedAt(1));
        QVERIFY(!regions.isAutoClosedAt(0));

        regions.consumeAt(1);
        QVERIFY(!regions.isAutoClosedAt(1));
        QCOMPARE(regions.trackedCount(), 0);
    }

    // O caso que mata a implementacao por int: editar ANTES do fechador tem
    // que mover o registro junto. E' o Qt quem move o QTextCursor.
    void edicaoAntesDoFechadorDeslocaORegistro()
    {
        QTextDocument doc(QStringLiteral("()"));
        AutoCloseRegions regions;
        regions.setDocument(&doc);
        regions.notePairInserted(1);

        QTextCursor typing(&doc);
        typing.setPosition(1);
        typing.insertText(QStringLiteral("abc")); // "(abc)"

        QVERIFY(regions.isAutoClosedAt(4));
        QVERIFY(!regions.isAutoClosedAt(1));
    }

    void apagarOFechadorMataORegistro()
    {
        QTextDocument doc(QStringLiteral("()"));
        AutoCloseRegions regions;
        regions.setDocument(&doc);
        regions.notePairInserted(1);

        QTextCursor eraser(&doc);
        eraser.setPosition(1);
        eraser.setPosition(2, QTextCursor::KeepAnchor);
        eraser.removeSelectedText(); // "("

        QVERIFY(!regions.isAutoClosedAt(1));
        QCOMPARE(regions.trackedCount(), 0);
    }

    // Undo devolve o TEXTO, nao o registro: a selecao colapsada nao reabre.
    // Semantica deliberada (ver comentario em isAlive) — este caso a FIXA para
    // que uma mudanca futura seja decisao, nao acidente.
    void undoNaoRessuscitaORegistro()
    {
        QTextDocument doc;
        AutoCloseRegions regions;
        regions.setDocument(&doc);
        doc.setUndoRedoEnabled(true);

        QTextCursor typing(&doc);
        typing.insertText(QStringLiteral("()"));
        regions.notePairInserted(1);

        typing.setPosition(1);
        typing.setPosition(2, QTextCursor::KeepAnchor);
        typing.removeSelectedText();
        QCOMPARE(regions.trackedCount(), 0);

        doc.undo();
        QCOMPARE(doc.toPlainText(), QStringLiteral("()"));
        QVERIFY(!regions.isAutoClosedAt(1));
        QCOMPARE(regions.trackedCount(), 0);
    }

    void trocarDeDocumentoLimpaTudo()
    {
        QTextDocument primeiro(QStringLiteral("()"));
        QTextDocument segundo(QStringLiteral("[]"));
        AutoCloseRegions regions;
        regions.setDocument(&primeiro);
        regions.notePairInserted(1);

        regions.setDocument(&segundo);
        QVERIFY(!regions.isAutoClosedAt(1));
        QCOMPARE(regions.trackedCount(), 0);
    }

    void documentoDestruidoNaoDeixaCursorOrfao()
    {
        AutoCloseRegions regions;
        {
            QTextDocument doc(QStringLiteral("()"));
            regions.setDocument(&doc);
            regions.notePairInserted(1);
            QCOMPARE(regions.trackedCount(), 1);
        }
        // Consultar depois da morte do documento nao pode tocar cursor morto.
        QCOMPARE(regions.trackedCount(), 0);
        QVERIFY(!regions.isAutoClosedAt(1));
    }

    void limiteDeRegistrosNaoCresceParaSempre()
    {
        QTextDocument doc(QString(200, QLatin1Char(')')));
        AutoCloseRegions regions;
        regions.setDocument(&doc);
        for (int i = 0; i < 200; ++i) {
            regions.notePairInserted(i);
        }
        QVERIFY(regions.trackedCount() <= 64);
        // Os mais RECENTES sobrevivem: e' neles que o usuario esta digitando.
        QVERIFY(regions.isAutoClosedAt(199));
    }

    void posicaoInvalidaEDocumentoNuloSaoInofensivos()
    {
        AutoCloseRegions regions;
        regions.notePairInserted(0); // sem documento: nao explode
        QVERIFY(!regions.isAutoClosedAt(0));

        QTextDocument doc(QStringLiteral("()"));
        regions.setDocument(&doc);
        regions.notePairInserted(-1);
        regions.notePairInserted(99);
        QCOMPARE(regions.trackedCount(), 0);
    }
};

QTEST_MAIN(TestAutoCloseRegions)
#include "tst_auto_close_regions.moc"
