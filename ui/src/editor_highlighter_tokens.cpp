// Tokens vindos de FORA: semanticos (LSP) e sintaticos (Tree-sitter).
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03, etapa 16). Estas duas camadas sao
// as unicas que dependem de um produtor externo e que chegam com "relogio"
// proprio (syntaxVersion/semanticVersion). Manter as duas juntas e longe do
// fallback por regex deixa visivel a ordem de precedencia do realce:
// regex e' o piso, Tree-sitter cobre, LSP cobre por ultimo.
#include "editor_highlighter.h"
#include "editor_highlighter_palette.h"
#include "line_spans.h"

#include <algorithm>

#include <QTextBlock>
#include <QTextDocument>

namespace kinein {

using namespace kinein::highlight;

void EditorHighlighter::setSemanticTokens(const QVariantList& tokens)
{
    QHash<int, QList<SemanticSpan>> next;
    for (const QVariant& entry : tokens) {
        const QVariantMap map = entry.toMap();
        // A linha vem 1-based do core, e ate' 2026-09-25 virava 0-based ANTES
        // de ser validada. No papel isso e' `INT_MIN - 1`, e e' o que o
        // `-Wstrict-overflow=5` do GCC acusa no preset release: ele reescreve
        // `x - 1 < 0` como `x < 1` assumindo que o overflow nao acontece.
        // Validar antes de subtrair tira a suposicao — a conta so' existe
        // depois de se saber que ela cabe.
        const int oneBasedLine = map.value(QStringLiteral("line")).toInt();
        SemanticSpan span;
        span.start = map.value(QStringLiteral("start")).toInt();
        span.length = map.value(QStringLiteral("length")).toInt();
        span.format = formatForSemanticKind(map.value(QStringLiteral("kind")).toString());
        if (oneBasedLine < 1 || span.start < 0 || span.length <= 0 ||
            span.format.foreground().style() == Qt::NoBrush)
        {
            continue;
        }
        const int line = oneBasedLine - 1;
        auto spans = next.find(line);
        if (spans == next.end()) {
            spans = next.insert(line, QList<SemanticSpan>{});
        }
        spans.value().append(span);
    }
    const QSet<int> changed = changedLines(m_semanticSpansByLine, next);
    m_semanticSpansByLine = std::move(next);
    rehighlightLines(changed);
}

void EditorHighlighter::clearSemanticTokens()
{
    if (m_semanticSpansByLine.isEmpty()) {
        return;
    }
    const QSet<int> changed = changedLines(m_semanticSpansByLine, {});
    m_semanticSpansByLine.clear();
    rehighlightLines(changed);
}

void EditorHighlighter::setSyntaxTokens(const QVariantList& tokens)
{
    QHash<int, QList<SemanticSpan>> next;
    for (const QVariant& entry : tokens) {
        const QVariantMap map = entry.toMap();
        // A linha vem 1-based do core, e ate' 2026-09-25 virava 0-based ANTES
        // de ser validada. No papel isso e' `INT_MIN - 1`, e e' o que o
        // `-Wstrict-overflow=5` do GCC acusa no preset release: ele reescreve
        // `x - 1 < 0` como `x < 1` assumindo que o overflow nao acontece.
        // Validar antes de subtrair tira a suposicao — a conta so' existe
        // depois de se saber que ela cabe.
        const int oneBasedLine = map.value(QStringLiteral("line")).toInt();
        SemanticSpan span;
        span.start = map.value(QStringLiteral("start")).toInt();
        span.length = map.value(QStringLiteral("length")).toInt();
        span.format = formatForSyntaxScope(map.value(QStringLiteral("scope")).toString());
        if (oneBasedLine < 1 || span.start < 0 || span.length <= 0 ||
            span.format.foreground().style() == Qt::NoBrush)
        {
            continue;
        }
        const int line = oneBasedLine - 1;
        auto spans = next.find(line);
        if (spans == next.end()) {
            spans = next.insert(line, QList<SemanticSpan>{});
        }
        spans.value().append(span);
    }
    const QSet<int> changed = changedLines(m_syntaxSpansByLine, next);
    m_syntaxSpansByLine = std::move(next);
    rehighlightLines(changed);
}

void EditorHighlighter::clearSyntaxTokens()
{
    if (m_syntaxSpansByLine.isEmpty()) {
        return;
    }
    const QSet<int> changed = changedLines(m_syntaxSpansByLine, {});
    m_syntaxSpansByLine.clear();
    rehighlightLines(changed);
}

// A edicao comecou no bloco de `position` e mudou a contagem de linhas em
// `delta`. Os spans andam com o texto; a linha editada perde o token
// semantico (pintaria a coluna errada ate' o LSP responder) e guarda o do
// Tree-sitter. O QSyntaxHighlighter ja' refez os blocos editados com os
// spans de ANTES; aqui so' se repinta o que esses spans velhos sujaram.
void EditorHighlighter::trackEdit(int position, int charsRemoved, int charsAdded)
{
    Q_UNUSED(charsRemoved)
    Q_UNUSED(charsAdded)
    const QTextDocument* textDocument = document();
    if (textDocument == nullptr) {
        return;
    }
    const int blocks = textDocument->blockCount();
    const int delta = blocks - m_lastBlockCount;
    m_lastBlockCount = blocks;
    if (m_semanticSpansByLine.isEmpty() && m_syntaxSpansByLine.isEmpty()) {
        return;
    }
    const QTextBlock first = textDocument->findBlock(position);
    if (!first.isValid()) {
        return;
    }
    const int start = first.blockNumber();
    QSet<int> dirty;
    for (int line = start; line <= start + std::max(0, delta); ++line) {
        if (m_semanticSpansByLine.contains(line) || (line != start && m_syntaxSpansByLine.contains(line))) {
            dirty.insert(line);
        }
    }
    m_semanticSpansByLine = followEdit(m_semanticSpansByLine, start, delta, false);
    m_syntaxSpansByLine = followEdit(m_syntaxSpansByLine, start, delta, true);
    rehighlightLines(dirty);
}

// Repinta so' estas linhas. Quando sao a maior parte do documento (o
// primeiro lote de tokens), uma passada inteira sai mais barata que
// bloco a bloco.
void EditorHighlighter::rehighlightLines(const QSet<int>& lines)
{
    const QTextDocument* textDocument = document();
    if (lines.isEmpty() || textDocument == nullptr) {
        return;
    }
    if (lines.size() * 2 > textDocument->blockCount()) {
        rehighlight();
        return;
    }
    for (const int line : lines) {
        const QTextBlock block = textDocument->findBlockByNumber(line);
        if (block.isValid()) {
            rehighlightBlock(block);
        }
    }
}

void EditorHighlighter::applySemanticSpans()
{
    const auto spans = m_semanticSpansByLine.constFind(currentBlock().blockNumber());
    if (spans == m_semanticSpansByLine.constEnd()) {
        return;
    }
    for (const SemanticSpan& span : *spans) {
        setFormat(span.start, span.length, span.format);
    }
}

void EditorHighlighter::applySyntaxSpans()
{
    const auto spans = m_syntaxSpansByLine.constFind(currentBlock().blockNumber());
    if (spans == m_syntaxSpansByLine.constEnd()) {
        return;
    }
    for (const SemanticSpan& span : *spans) {
        setFormat(span.start, span.length, span.format);
    }
}

QTextCharFormat EditorHighlighter::formatForSemanticKind(const QString& kind)
{
    if (kind == u"function" || kind == u"method") {
        return colorFormat(kFunctionRgb);
    }
    if (kind == u"namespace" || kind == u"type" || kind == u"class" || kind == u"enum" ||
        kind == u"interface" || kind == u"struct" || kind == u"typeParameter" ||
        kind == u"typeAlias" || kind == u"builtinType" || kind == u"union" || kind == u"concept" ||
        kind == u"generic")
    {
        return colorFormat(kTypeRgb);
    }
    if (kind == u"parameter") {
        return colorFormat(kVariableRgb, false, true);
    }
    if (kind == u"variable") {
        return colorFormat(kVariableRgb);
    }
    if (kind == u"property" || kind == u"enumMember" || kind == u"enumConstant" ||
        kind == u"event" || kind == u"const" || kind == u"static")
    {
        return colorFormat(kPropertyRgb);
    }
    if (kind == u"lifetime") {
        return colorFormat(kMetaRgb, false, true);
    }
    if (kind == u"macro" || kind == u"decorator" || kind == u"derive" || kind == u"attribute" ||
        kind == u"attributeBracket" || kind == u"escapeSequence" || kind == u"formatSpecifier" ||
        kind == u"label")
    {
        return colorFormat(kMetaRgb);
    }
    if (kind == u"keyword" || kind == u"modifier" || kind == u"selfKeyword" || kind == u"boolean") {
        return colorFormat(kKeywordRgb, true);
    }
    if (kind == u"comment") {
        return colorFormat(kCommentRgb, false, true);
    }
    if (kind == u"string" || kind == u"regexp" || kind == u"character") {
        return colorFormat(kStringRgb);
    }
    if (kind == u"number") {
        return colorFormat(kNumberRgb);
    }
    // operator, bracket, punctuation e kinds desconhecidos ficam sem cor:
    // recolorir pontuacao e ruido visual sem ganho de leitura.
    return {};
}

QTextCharFormat EditorHighlighter::formatForSyntaxScope(const QString& scope)
{
    const QString base = scope.section(u'.', 0, 0);
    if (base == u"function" || base == u"method" || base == u"constructor" || base == u"destructor")
    {
        return colorFormat(kFunctionRgb);
    }
    if (base == u"type" || base == u"class" || base == u"struct" || base == u"enum" ||
        base == u"union" || base == u"interface" || base == u"namespace" || base == u"module")
    {
        return colorFormat(kTypeRgb);
    }
    if (base == u"parameter") {
        return colorFormat(kVariableRgb, false, true);
    }
    if (base == u"variable") {
        return colorFormat(kVariableRgb);
    }
    if (base == u"property" || base == u"field" || base == u"constant") {
        return colorFormat(kPropertyRgb);
    }
    if (base == u"attribute" || base == u"macro" || base == u"label") {
        return colorFormat(kMetaRgb);
    }
    if (base == u"keyword" || base == u"boolean") {
        return colorFormat(kKeywordRgb, true);
    }
    if (base == u"comment") {
        return colorFormat(kCommentRgb, false, true);
    }
    if (base == u"string" || base == u"character") {
        return colorFormat(kStringRgb);
    }
    if (base == u"number" || base == u"float") {
        return colorFormat(kNumberRgb);
    }
    return {};
}

} // namespace kinein
