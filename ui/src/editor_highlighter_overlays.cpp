// Sobreposicoes: busca (Find/Replace) e diagnosticos do LSP.
//
// POR QUE JUNTAS (2026-09-03, etapa 16). Nao e' "duas coisas pequenas num
// arquivo so": as duas sao a MESMA responsabilidade — pintar por cima do
// realce base algo que nao vem da linguagem, e sim do estado da sessao (o que
// procuro, o que o servidor reclamou). Ambas rodam DEPOIS do realce e nenhuma
// participa da decisao de cor do token.
#include "editor_highlighter.h"
#include "editor_highlighter_palette.h"

#include <QTextBlock>

namespace kinein {

using namespace kinein::highlight;

void EditorHighlighter::setSearchMatches(const QVariantList& matches, int current)
{
    const bool hadMatches = !m_searchMatches.isEmpty();
    m_searchMatches.clear();
    for (const QVariant& entry : matches) {
        const QVariantMap map = entry.toMap();
        SearchSpan span;
        span.start = map.value(QStringLiteral("start")).toInt();
        span.end = map.value(QStringLiteral("end")).toInt();
        if (span.end > span.start) {
            m_searchMatches.append(span);
        }
    }
    m_currentSearchMatch = current;
    if (hadMatches || !m_searchMatches.isEmpty()) {
        rehighlight();
    }
}

void EditorHighlighter::applySearchSpans(const QString& text)
{
    if (m_searchMatches.isEmpty()) {
        return;
    }
    // Os offsets vem ABSOLUTOS; converter para o bloco atual e recortar o
    // que cai fora dele (uma ocorrencia nao cruza linhas na busca literal,
    // mas uma regex pode — o recorte trata os dois casos).
    const int blockStart = currentBlock().position();
    const int blockEnd = blockStart + static_cast<int>(text.length());
    for (int i = 0; i < m_searchMatches.size(); ++i) {
        const SearchSpan& span = m_searchMatches.at(i);
        const int start = qMax(span.start, blockStart);
        const int end = qMin(span.end, blockEnd);
        if (start >= end) {
            continue;
        }
        const QColor background = i == m_currentSearchMatch ? QColor::fromRgb(kSearchCurrentRgb)
                                                            : QColor::fromRgb(kSearchMatchRgb);
        // setFormat SUBSTITUI o formato; mesclar por caractere preserva a
        // cor do texto (realce/semantic) e so acrescenta o fundo.
        for (int position = start; position < end; ++position) {
            const int offset = position - blockStart;
            QTextCharFormat merged = format(offset);
            merged.setBackground(background);
            setFormat(offset, 1, merged);
        }
    }
}

void EditorHighlighter::setDiagnostics(const QVariantList& diagnostics)
{
    m_diagnostics.clear();
    for (const QVariant& entry : diagnostics) {
        const QVariantMap map = entry.toMap();
        DiagnosticSpan span;
        span.startLine = map.value(QStringLiteral("startLine")).toInt();
        span.startChar = map.value(QStringLiteral("startChar")).toInt();
        span.endLine = map.value(QStringLiteral("endLine")).toInt();
        span.endChar = map.value(QStringLiteral("endChar")).toInt();
        span.severity = map.value(QStringLiteral("severity")).toString();
        if (span.endLine < span.startLine) {
            continue;
        }
        m_diagnostics.append(span);
    }
    rehighlight();
}

void EditorHighlighter::clearDiagnostics()
{
    if (m_diagnostics.isEmpty()) {
        return;
    }
    m_diagnostics.clear();
    rehighlight();
}

void EditorHighlighter::applyDiagnosticSpans(const QString& text)
{
    if (m_diagnostics.isEmpty()) {
        return;
    }
    const int block = currentBlock().blockNumber();
    const int length = static_cast<int>(text.length());
    if (length == 0) {
        return;
    }
    for (const DiagnosticSpan& diagnostic : m_diagnostics) {
        if (block < diagnostic.startLine || block > diagnostic.endLine) {
            continue;
        }
        // Range por bloco: da coluna inicial (na primeira linha) ate a
        // coluna final (na ultima), cobrindo a linha toda no meio.
        int start = block == diagnostic.startLine ? diagnostic.startChar : 0;
        int end = block == diagnostic.endLine ? diagnostic.endChar : length;
        start = qBound(0, start, length - 1);
        end = qBound(0, end, length);
        // Range vazio (ex.: ';' faltando) ainda marca 1 caractere.
        if (end <= start) {
            end = qMin(start + 1, length);
        }
        const QColor color = underlineFormatForSeverity(diagnostic.severity).underlineColor();
        // setFormat do QSyntaxHighlighter SUBSTITUI o formato; para
        // preservar a cor do texto (realce/semantic) somamos o
        // sublinhado por caractere, mesclando com o formato atual.
        for (int position = start; position < end; ++position) {
            QTextCharFormat merged = format(position);
            merged.setUnderlineStyle(QTextCharFormat::SpellCheckUnderline);
            merged.setUnderlineColor(color);
            setFormat(position, 1, merged);
        }
    }
}

QTextCharFormat EditorHighlighter::underlineFormatForSeverity(const QString& severity)
{
    QTextCharFormat format;
    format.setUnderlineStyle(QTextCharFormat::SpellCheckUnderline);
    QRgb color = kMetaRgb; // error soft (vermelho) por padrao
    if (severity == QStringLiteral("warning")) {
        color = kKeywordRgb; // ambar/accent
    }
    else if (severity == QStringLiteral("note")) {
        color = kNumberRgb; // info soft (azul)
    }
    format.setUnderlineColor(QColor::fromRgb(color));
    return format;
}

} // namespace kinein
