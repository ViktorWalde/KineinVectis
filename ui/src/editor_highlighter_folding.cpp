// Dobra de codigo (folding): quais linhas estao VISIVEIS.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03, etapa 16). Dobra nao e' realce.
// Ela nao decide cor de nada — decide se um bloco de texto aparece. Morava no
// highlighter por acidente de hospedagem (o QSyntaxHighlighter ja tinha o
// QTextDocument na mao), e esse acidente e' o que a §4 regra 9 chama de
// responsabilidade misturada. O roadmaps/34 §3 descrevia este arquivo como
// "quatro camadas de composicao" e nao mencionava a dobra: ela so apareceu na
// medicao.
#include "editor_highlighter.h"

#include <QSet>
#include <QTextBlock>
#include <QTextDocument>

namespace kinein {

void EditorHighlighter::setFoldingRanges(const QVariantList& ranges)
{
    m_foldingRanges.clear();
    QSet<int> validStarts;
    for (const QVariant& entry : ranges) {
        const QVariantMap map = entry.toMap();
        FoldingRange range;
        range.startLine = map.value(QStringLiteral("startLine")).toInt();
        range.endLine = map.value(QStringLiteral("endLine")).toInt();
        if (range.startLine <= 0 || range.endLine <= range.startLine) {
            continue;
        }
        m_foldingRanges.append(range);
        validStarts.insert(range.startLine);
    }
    m_foldedStartLines.intersect(validStarts);
    applyFoldVisibility();
    emit foldingChanged();
}

bool EditorHighlighter::toggleFoldAtLine(int line)
{
    if (!isFoldableLine(line)) {
        return false;
    }
    if (m_foldedStartLines.contains(line)) {
        m_foldedStartLines.remove(line);
    }
    else {
        m_foldedStartLines.insert(line);
    }
    applyFoldVisibility();
    emit foldingChanged();
    return true;
}

bool EditorHighlighter::isFoldableLine(int line) const
{
    return std::ranges::any_of(
        m_foldingRanges, [line](const FoldingRange& range) { return range.startLine == line; });
}

bool EditorHighlighter::isFoldedLine(int line) const
{
    return m_foldedStartLines.contains(line);
}

QVariantList EditorHighlighter::visibleLineNumbers() const
{
    QVariantList lines;
    if (document() == nullptr) {
        return lines;
    }
    for (QTextBlock block = document()->begin(); block.isValid(); block = block.next()) {
        if (block.isVisible()) {
            lines.append(block.blockNumber() + 1);
        }
    }
    return lines;
}

void EditorHighlighter::applyFoldVisibility()
{
    if (document() == nullptr) {
        return;
    }
    for (QTextBlock block = document()->begin(); block.isValid(); block = block.next()) {
        const int line = block.blockNumber() + 1;
        bool visible = true;
        for (const FoldingRange& range : m_foldingRanges) {
            if (m_foldedStartLines.contains(range.startLine) && line > range.startLine &&
                line <= range.endLine)
            {
                visible = false;
                break;
            }
        }
        block.setVisible(visible);
        block.setLineCount(visible ? 1 : 0);
    }
    document()->markContentsDirty(0, document()->characterCount());
}

void EditorHighlighter::showAllBlocks()
{
    if (document() == nullptr) {
        return;
    }
    for (QTextBlock block = document()->begin(); block.isValid(); block = block.next()) {
        block.setVisible(true);
        block.setLineCount(1);
    }
    document()->markContentsDirty(0, document()->characterCount());
}

void EditorHighlighter::resetFolding()
{
    showAllBlocks();
    m_foldingRanges.clear();
    m_foldedStartLines.clear();
}

} // namespace kinein
