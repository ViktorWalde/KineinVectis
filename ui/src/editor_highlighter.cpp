#include "editor_highlighter.h"

#include "editor_highlighter_palette.h"

#include <algorithm>

#include <QColor>
#include <QFileInfo>
#include <QRegularExpressionMatch>
#include <QRegularExpressionMatchIterator>
#include <QStringList>
#include <QTextBlock>
#include <QTextDocument>

using namespace kinein::highlighter_palette;

namespace kinein {

EditorHighlighter::EditorHighlighter(QObject* parent) : QSyntaxHighlighter(parent) {}

QQuickTextDocument* EditorHighlighter::quickDocument() const
{
    return m_quickDocument;
}

void EditorHighlighter::setQuickDocument(QQuickTextDocument* document)
{
    if (m_quickDocument == document) {
        return;
    }
    showAllBlocks();
    m_foldingRanges.clear();
    m_foldedStartLines.clear();
    m_quickDocument = document;
    setDocument(document != nullptr ? document->textDocument() : nullptr);
    emit documentChanged();
}

QString EditorHighlighter::filePath() const
{
    return m_filePath;
}

void EditorHighlighter::setFilePath(const QString& filePath)
{
    if (m_filePath == filePath) {
        return;
    }
    showAllBlocks();
    m_filePath = filePath;
    m_syntaxSpansByLine.clear();
    m_semanticSpansByLine.clear();
    m_foldingRanges.clear();
    m_foldedStartLines.clear();
    const QString newLanguage = languageForPath(filePath);
    if (newLanguage != m_language) {
        m_language = newLanguage;
        rebuildRules();
    }
    rehighlight();
    emit foldingChanged();
    emit filePathChanged();
}

QString EditorHighlighter::language() const
{
    return m_language;
}

void EditorHighlighter::highlightBlock(const QString& text)
{
    for (const Rule& rule : m_rules) {
        QRegularExpressionMatchIterator matches = rule.pattern.globalMatch(text);
        while (matches.hasNext()) {
            const QRegularExpressionMatch match = matches.next();
            setFormat(static_cast<int>(match.capturedStart()),
                      static_cast<int>(match.capturedLength()), rule.format);
        }
    }

    applyBlockSpans(text);
    applySyntaxSpans();
    applySemanticSpans();
    applyStdlibOverride(text);
    applyDiagnosticSpans(text);
    applySearchSpans(text);
}

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

void EditorHighlighter::applyStdlibOverride(const QString& text)
{
    // O clangd marca cout/cin/cerr/clog como "variable" (quase branco);
    // este conjunto icônico de streams merece destaque como callable da
    // stdlib. Único override PÓS-semantic de propósito (docs/diario/18, CR1).
    if (m_language != QStringLiteral("cpp")) {
        return;
    }
    static const QRegularExpression streams(
        QStringLiteral("\\b(?:cout|cin|cerr|clog|wcout|wcin|wcerr|wclog)\\b"));
    static const QTextCharFormat streamFormat = colorFormat(kFunctionRgb);
    QRegularExpressionMatchIterator matches = streams.globalMatch(text);
    while (matches.hasNext()) {
        const QRegularExpressionMatch match = matches.next();
        setFormat(static_cast<int>(match.capturedStart()), static_cast<int>(match.capturedLength()),
                  streamFormat);
    }
}

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

void EditorHighlighter::applyBlockSpans(const QString& text)
{
    setCurrentBlockState(0);
    if (!m_hasBlockSpans) {
        return;
    }

    qsizetype startIndex = 0;
    if (previousBlockState() != 1) {
        startIndex = text.indexOf(m_blockStart);
    }

    while (startIndex >= 0) {
        QRegularExpressionMatch endMatch;
        const qsizetype searchFrom =
            previousBlockState() == 1 && startIndex == 0 ? 0 : startIndex + 1;
        const qsizetype endIndex = text.indexOf(m_blockEnd, searchFrom, &endMatch);

        qsizetype spanLength = 0;
        if (endIndex < 0) {
            setCurrentBlockState(1);
            spanLength = text.length() - startIndex;
        }
        else {
            spanLength = endIndex - startIndex + endMatch.capturedLength();
        }
        setFormat(static_cast<int>(startIndex), static_cast<int>(spanLength), m_blockFormat);
        if (endIndex < 0) {
            break;
        }
        startIndex = text.indexOf(m_blockStart, startIndex + spanLength);
    }
}

} // namespace kinein
