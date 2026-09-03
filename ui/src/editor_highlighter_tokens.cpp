// Tokens vindos de FORA: semanticos (LSP) e sintaticos (Tree-sitter).
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03, etapa 16). Estas duas camadas sao
// as unicas que dependem de um produtor externo e que chegam com "relogio"
// proprio (syntaxVersion/semanticVersion). Manter as duas juntas e longe do
// fallback por regex deixa visivel a ordem de precedencia do realce:
// regex e' o piso, Tree-sitter cobre, LSP cobre por ultimo.
#include "editor_highlighter.h"
#include "editor_highlighter_palette.h"

#include <QTextBlock>

namespace kinein {

using namespace kinein::highlight;

void EditorHighlighter::setSemanticTokens(const QVariantList& tokens)
{
    m_semanticSpansByLine.clear();
    for (const QVariant& entry : tokens) {
        const QVariantMap map = entry.toMap();
        const int line = map.value(QStringLiteral("line")).toInt() - 1;
        SemanticSpan span;
        span.start = map.value(QStringLiteral("start")).toInt();
        span.length = map.value(QStringLiteral("length")).toInt();
        span.format = formatForSemanticKind(map.value(QStringLiteral("kind")).toString());
        if (line < 0 || span.start < 0 || span.length <= 0 ||
            span.format.foreground().style() == Qt::NoBrush)
        {
            continue;
        }
        auto spans = m_semanticSpansByLine.find(line);
        if (spans == m_semanticSpansByLine.end()) {
            spans = m_semanticSpansByLine.insert(line, QList<SemanticSpan>{});
        }
        spans.value().append(span);
    }
    rehighlight();
}

void EditorHighlighter::clearSemanticTokens()
{
    if (m_semanticSpansByLine.isEmpty()) {
        return;
    }
    m_semanticSpansByLine.clear();
    rehighlight();
}

void EditorHighlighter::setSyntaxTokens(const QVariantList& tokens)
{
    m_syntaxSpansByLine.clear();
    for (const QVariant& entry : tokens) {
        const QVariantMap map = entry.toMap();
        const int line = map.value(QStringLiteral("line")).toInt() - 1;
        SemanticSpan span;
        span.start = map.value(QStringLiteral("start")).toInt();
        span.length = map.value(QStringLiteral("length")).toInt();
        span.format = formatForSyntaxScope(map.value(QStringLiteral("scope")).toString());
        if (line < 0 || span.start < 0 || span.length <= 0 ||
            span.format.foreground().style() == Qt::NoBrush)
        {
            continue;
        }
        auto spans = m_syntaxSpansByLine.find(line);
        if (spans == m_syntaxSpansByLine.end()) {
            spans = m_syntaxSpansByLine.insert(line, QList<SemanticSpan>{});
        }
        spans.value().append(span);
    }
    rehighlight();
}

void EditorHighlighter::clearSyntaxTokens()
{
    if (m_syntaxSpansByLine.isEmpty()) {
        return;
    }
    m_syntaxSpansByLine.clear();
    rehighlight();
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
