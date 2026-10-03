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
#include <QTextDocument>

namespace kinein {

using namespace kinein::highlight;

namespace {

// As linhas que um conjunto de diagnosticos toca: so' elas mudam de
// sublinhado quando a lista troca (0.3.9: antes, cada publicacao do LSP —
// ate' a lista vazia que continuava vazia — repintava o documento inteiro).
template <typename Diagnostics>
void addDiagnosticLines(const Diagnostics& diagnostics, QSet<int>& lines)
{
    for (const auto& span : diagnostics) {
        for (int line = span.startLine; line <= span.endLine; ++line) {
            lines.insert(line);
        }
    }
}

} // namespace

// As linhas onde caem as ocorrencias da busca (offsets absolutos).
QSet<int> EditorHighlighter::searchLines() const
{
    QSet<int> lines;
    const QTextDocument* textDocument = document();
    if (textDocument == nullptr) {
        return lines;
    }
    for (const SearchSpan& span : m_searchMatches) {
        const int first = textDocument->findBlock(span.start).blockNumber();
        const int last = textDocument->findBlock(qMax(span.start, span.end - 1)).blockNumber();
        for (int line = first; line <= last; ++line) {
            lines.insert(line);
        }
    }
    return lines;
}

void EditorHighlighter::setSearchMatches(const QVariantList& matches, int current)
{
    QSet<int> dirty = searchLines();
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
    dirty.unite(searchLines());
    rehighlightLines(dirty);
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
    QSet<int> dirty;
    addDiagnosticLines(m_diagnostics, dirty);
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
    addDiagnosticLines(m_diagnostics, dirty);
    rehighlightLines(dirty);
}

void EditorHighlighter::clearDiagnostics()
{
    if (m_diagnostics.isEmpty()) {
        return;
    }
    QSet<int> dirty;
    addDiagnosticLines(m_diagnostics, dirty);
    m_diagnostics.clear();
    rehighlightLines(dirty);
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
