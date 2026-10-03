// Os spans de realce acompanham a edicao (0.3.9, 40.7 §7.198).
//
// O defeito que isto previne: apagar os tokens semanticos a cada tecla
// refazia o realce do documento inteiro (~500 ms na primeira tecla de um
// arquivo de 2.463 linhas). A correcao move os spans junto com o texto e
// repinta so' as linhas que mudaram; aqui se prova a parte pura.
#include "line_spans.h"

#include <QtTest>

using kinein::highlight::changedLines;
using kinein::highlight::followEdit;

namespace {
using Spans = QHash<int, QList<int>>;

Spans sample()
{
    // linha -> "spans" (inteiros bastam: so' a posicao importa aqui)
    return Spans{{0, {10}}, {3, {13}}, {4, {14}}, {8, {18}}};
}
} // namespace

class TestLineSpans : public QObject
{
    Q_OBJECT

private slots:
    void typing_inside_a_line_moves_nothing()
    {
        const Spans moved = followEdit(sample(), 3, 0, false);
        QCOMPARE(moved, (Spans{{0, {10}}, {4, {14}}, {8, {18}}}));
    }

    void the_typed_line_keeps_its_spans_only_when_asked()
    {
        const Spans moved = followEdit(sample(), 3, 0, true);
        QCOMPARE(moved, sample());
    }

    void inserted_lines_push_the_ones_below()
    {
        // Enter na linha 3: duas linhas novas; 4 e 8 descem duas.
        const Spans moved = followEdit(sample(), 3, 2, true);
        QCOMPARE(moved, (Spans{{0, {10}}, {3, {13}}, {6, {14}}, {10, {18}}}));
    }

    void removed_lines_vanish_and_the_rest_climbs()
    {
        // Apagar da linha 3 ate' o fim da 4 (uma linha a menos): a 4 some,
        // a 8 sobe para a 7.
        const Spans moved = followEdit(sample(), 3, -1, true);
        QCOMPARE(moved, (Spans{{0, {10}}, {3, {13}}, {7, {18}}}));
    }

    void lines_above_the_edit_never_move()
    {
        const Spans moved = followEdit(sample(), 5, 3, false);
        QCOMPARE(moved.value(0), QList<int>{10});
        QCOMPARE(moved.value(3), QList<int>{13});
        QCOMPARE(moved.value(4), QList<int>{14});
        QCOMPARE(moved.value(11), QList<int>{18});
    }

    void only_lines_that_differ_need_repainting()
    {
        Spans after = sample();
        after[4] = {99};
        after.remove(8);
        after.insert(12, {1});
        const QSet<int> changed = changedLines(sample(), after);
        QCOMPARE(changed, (QSet<int>{4, 8, 12}));
    }

    void the_same_map_repaints_nothing()
    {
        QVERIFY(changedLines(sample(), sample()).isEmpty());
    }
};

QTEST_GUILESS_MAIN(TestLineSpans)

#ifdef __clang__
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wctad-maybe-unsupported"
#endif
#include "tst_line_spans.moc"
#ifdef __clang__
#pragma clang diagnostic pop
#endif
