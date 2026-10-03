// SPANS POR LINHA que acompanham a edicao (0.3.9, pente fino).
//
// As camadas de realce que vem de fora — Tree-sitter e LSP — guardam spans
// por NUMERO de linha (QHash<int, QList<Span>>). Ate' aqui a UI apagava todos
// os tokens semanticos a cada tecla (`clearSemanticTokens` no
// `invalidateForEdit`), e apagar refazia o realce do documento INTEIRO: numa
// fixture de 2.463 linhas, ~500 ms de tela parada na primeira tecla depois
// de os tokens chegarem (medido em 2026-10-03, 40.7 §7.198).
//
// Estas duas funcoes sao a parte pura da correcao, testada sem janela em
// ui/tests/tst_line_spans.cpp:
//
//   followEdit    a edicao comecou na linha `start` e mudou a contagem de
//                 linhas em `delta`. Antes de `start`: fica. Depois da regiao
//                 editada: anda `delta`. Linhas que a edicao engoliu: somem.
//                 A propria `start` fica so' com `keepStart` (o Tree-sitter
//                 mantem a cor da linha digitada ate' o proximo snapshot; o
//                 LSP solta a dela, porque o token velho pintaria a coluna
//                 errada).
//
//   changedLines  as linhas cujos spans diferem entre dois mapas: so' elas
//                 precisam ser repintadas quando um mapa novo chega.
#pragma once

#include <algorithm>

#include <QHash>
#include <QList>
#include <QSet>

namespace kinein::highlight {

template <typename Span>
[[nodiscard]] QHash<int, QList<Span>> followEdit(const QHash<int, QList<Span>>& spans, int start,
                                                 int delta, bool keepStart)
{
    QHash<int, QList<Span>> moved;
    moved.reserve(spans.size());
    // A ultima linha ANTIGA que a edicao tocou: com linhas removidas, as
    // `-delta` seguintes a `start` foram engolidas por ela.
    const int lastTouched = start + std::max(0, -delta);
    for (auto it = spans.constBegin(); it != spans.constEnd(); ++it) {
        const int line = it.key();
        if (line < start) {
            moved.insert(line, it.value());
        }
        else if (line == start) {
            if (keepStart) {
                moved.insert(line, it.value());
            }
        }
        else if (line > lastTouched) {
            moved.insert(line + delta, it.value());
        }
    }
    return moved;
}

template <typename Span>
[[nodiscard]] QSet<int> changedLines(const QHash<int, QList<Span>>& before,
                                     const QHash<int, QList<Span>>& after)
{
    QSet<int> lines;
    for (auto it = before.constBegin(); it != before.constEnd(); ++it) {
        const auto other = after.constFind(it.key());
        if (other == after.constEnd() || other.value() != it.value()) {
            lines.insert(it.key());
        }
    }
    for (auto it = after.constBegin(); it != after.constEnd(); ++it) {
        if (!before.contains(it.key())) {
            lines.insert(it.key());
        }
    }
    return lines;
}

} // namespace kinein::highlight
