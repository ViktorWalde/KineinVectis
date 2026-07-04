#include "editor_highlighter.h"

#include <QColor>
#include <QFileInfo>
#include <QRegularExpressionMatch>
#include <QRegularExpressionMatchIterator>
#include <QStringList>

namespace kernwerk {

namespace {

// Paleta de docs/05-design-system.md, em QRgb para inicializacao constexpr.
constexpr QRgb kKeywordRgb = 0xffffbb00; // accent
constexpr QRgb kStringRgb = 0xff7fbf7f;  // success soft
constexpr QRgb kCommentRgb = 0xff8f8a7c; // text muted
constexpr QRgb kNumberRgb = 0xff7aa2d8;  // info soft
constexpr QRgb kMetaRgb = 0xffd16d6d;    // error soft (macros, atributos, secoes)

// Cores semanticas (LSP): distinguir variaveis, funcoes, tipos e campos.
constexpr QRgb kTypeRgb = 0xff5fb3ac;     // tipos/classes/enums/namespaces
constexpr QRgb kFunctionRgb = 0xffd8a657; // funcoes/metodos
constexpr QRgb kVariableRgb = 0xffcdd6e4; // variaveis
constexpr QRgb kPropertyRgb = 0xffb48ead; // campos/propriedades/enum members

QTextCharFormat colorFormat(QRgb rgb, bool bold = false, bool italic = false)
{
    QTextCharFormat format;
    format.setForeground(QColor::fromRgb(rgb));
    if (bold) {
        format.setFontWeight(QFont::DemiBold);
    }
    format.setFontItalic(italic);
    return format;
}

QRegularExpression keywordPattern(const QStringList& keywords)
{
    return QRegularExpression(QStringLiteral("\\b(?:%1)\\b").arg(keywords.join(u'|')));
}

const QRegularExpression& numberPattern()
{
    static const QRegularExpression pattern(QStringLiteral(
        "\\b(?:0[xXbBoO][0-9a-fA-F_]+|\\d[\\d_]*(?:\\.\\d+)?(?:[eE][+-]?\\d+)?)\\b"));
    return pattern;
}

const QRegularExpression& doubleQuoteString()
{
    static const QRegularExpression pattern(QStringLiteral("\"(?:\\\\.|[^\"\\\\])*\""));
    return pattern;
}

const QRegularExpression& singleQuoteString()
{
    static const QRegularExpression pattern(QStringLiteral("'(?:\\\\.|[^'\\\\])*'"));
    return pattern;
}

} // namespace

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
    m_filePath = filePath;
    m_semanticSpansByLine.clear();
    const QString newLanguage = languageForPath(filePath);
    if (newLanguage != m_language) {
        m_language = newLanguage;
        rebuildRules();
    }
    rehighlight();
    emit filePathChanged();
}

QString EditorHighlighter::language() const
{
    return m_language;
}

QString EditorHighlighter::languageForPath(const QString& filePath)
{
    const QFileInfo info(filePath);
    if (info.fileName().compare(QStringLiteral("CMakeLists.txt"), Qt::CaseInsensitive) == 0) {
        return QStringLiteral("cmake");
    }

    const QString suffix = info.suffix().toLower();
    if (suffix == QStringLiteral("rs")) {
        return QStringLiteral("rust");
    }
    if (suffix == QStringLiteral("c") || suffix == QStringLiteral("h") ||
        suffix == QStringLiteral("cc") || suffix == QStringLiteral("cpp") ||
        suffix == QStringLiteral("cxx") || suffix == QStringLiteral("hpp") ||
        suffix == QStringLiteral("hh") || suffix == QStringLiteral("ipp"))
    {
        return QStringLiteral("cpp");
    }
    if (suffix == QStringLiteral("py")) {
        return QStringLiteral("python");
    }
    if (suffix == QStringLiteral("cmake")) {
        return QStringLiteral("cmake");
    }
    if (suffix == QStringLiteral("toml")) {
        return QStringLiteral("toml");
    }
    if (suffix == QStringLiteral("qml") || suffix == QStringLiteral("js") ||
        suffix == QStringLiteral("mjs"))
    {
        return QStringLiteral("js");
    }
    if (suffix == QStringLiteral("json")) {
        return QStringLiteral("json");
    }
    if (suffix == QStringLiteral("sh")) {
        return QStringLiteral("shell");
    }
    return QStringLiteral("plain");
}

void EditorHighlighter::rebuildRules()
{
    m_rules.clear();
    m_hasBlockSpans = false;

    const QTextCharFormat keywordFormat = colorFormat(kKeywordRgb, true);
    const QTextCharFormat stringFormat = colorFormat(kStringRgb);
    const QTextCharFormat commentFormat = colorFormat(kCommentRgb, false, true);
    const QTextCharFormat numberFormat = colorFormat(kNumberRgb);
    const QTextCharFormat metaFormat = colorFormat(kMetaRgb);

    const auto addRule = [this](const QRegularExpression& pattern, const QTextCharFormat& format) {
        m_rules.append(Rule{pattern, format});
    };

    if (m_language == QStringLiteral("rust")) {
        addRule(keywordPattern(
                    {QStringLiteral("as"),    QStringLiteral("async"),  QStringLiteral("await"),
                     QStringLiteral("break"), QStringLiteral("const"),  QStringLiteral("continue"),
                     QStringLiteral("crate"), QStringLiteral("dyn"),    QStringLiteral("else"),
                     QStringLiteral("enum"),  QStringLiteral("extern"), QStringLiteral("false"),
                     QStringLiteral("fn"),    QStringLiteral("for"),    QStringLiteral("if"),
                     QStringLiteral("impl"),  QStringLiteral("in"),     QStringLiteral("let"),
                     QStringLiteral("loop"),  QStringLiteral("match"),  QStringLiteral("mod"),
                     QStringLiteral("move"),  QStringLiteral("mut"),    QStringLiteral("pub"),
                     QStringLiteral("ref"),   QStringLiteral("return"), QStringLiteral("self"),
                     QStringLiteral("Self"),  QStringLiteral("static"), QStringLiteral("struct"),
                     QStringLiteral("super"), QStringLiteral("trait"),  QStringLiteral("true"),
                     QStringLiteral("type"),  QStringLiteral("unsafe"), QStringLiteral("use"),
                     QStringLiteral("where"), QStringLiteral("while")}),
                keywordFormat);
        addRule(QRegularExpression(QStringLiteral("#!?\\[[^\\]]*\\]")), metaFormat);
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("//[^\n]*")), commentFormat);
        m_blockStart = QRegularExpression(QStringLiteral("/\\*"));
        m_blockEnd = QRegularExpression(QStringLiteral("\\*/"));
        m_blockFormat = commentFormat;
        m_hasBlockSpans = true;
    }
    else if (m_language == QStringLiteral("cpp") || m_language == QStringLiteral("js")) {
        if (m_language == QStringLiteral("cpp")) {
            addRule(keywordPattern({QStringLiteral("auto"),      QStringLiteral("bool"),
                                    QStringLiteral("break"),     QStringLiteral("case"),
                                    QStringLiteral("catch"),     QStringLiteral("char"),
                                    QStringLiteral("class"),     QStringLiteral("const"),
                                    QStringLiteral("constexpr"), QStringLiteral("continue"),
                                    QStringLiteral("default"),   QStringLiteral("delete"),
                                    QStringLiteral("do"),        QStringLiteral("double"),
                                    QStringLiteral("else"),      QStringLiteral("enum"),
                                    QStringLiteral("explicit"),  QStringLiteral("false"),
                                    QStringLiteral("float"),     QStringLiteral("for"),
                                    QStringLiteral("friend"),    QStringLiteral("if"),
                                    QStringLiteral("inline"),    QStringLiteral("int"),
                                    QStringLiteral("long"),      QStringLiteral("namespace"),
                                    QStringLiteral("new"),       QStringLiteral("noexcept"),
                                    QStringLiteral("nullptr"),   QStringLiteral("operator"),
                                    QStringLiteral("override"),  QStringLiteral("private"),
                                    QStringLiteral("protected"), QStringLiteral("public"),
                                    QStringLiteral("return"),    QStringLiteral("short"),
                                    QStringLiteral("signed"),    QStringLiteral("sizeof"),
                                    QStringLiteral("static"),    QStringLiteral("struct"),
                                    QStringLiteral("switch"),    QStringLiteral("template"),
                                    QStringLiteral("this"),      QStringLiteral("throw"),
                                    QStringLiteral("true"),      QStringLiteral("try"),
                                    QStringLiteral("typedef"),   QStringLiteral("typename"),
                                    QStringLiteral("union"),     QStringLiteral("unsigned"),
                                    QStringLiteral("using"),     QStringLiteral("virtual"),
                                    QStringLiteral("void"),      QStringLiteral("while")}),
                    keywordFormat);
            addRule(QRegularExpression(QStringLiteral("^\\s*#\\s*\\w+")), metaFormat);
        }
        else {
            addRule(keywordPattern({QStringLiteral("break"),      QStringLiteral("case"),
                                    QStringLiteral("catch"),      QStringLiteral("class"),
                                    QStringLiteral("const"),      QStringLiteral("continue"),
                                    QStringLiteral("default"),    QStringLiteral("delete"),
                                    QStringLiteral("do"),         QStringLiteral("else"),
                                    QStringLiteral("export"),     QStringLiteral("false"),
                                    QStringLiteral("finally"),    QStringLiteral("for"),
                                    QStringLiteral("function"),   QStringLiteral("if"),
                                    QStringLiteral("import"),     QStringLiteral("in"),
                                    QStringLiteral("instanceof"), QStringLiteral("let"),
                                    QStringLiteral("new"),        QStringLiteral("null"),
                                    QStringLiteral("of"),         QStringLiteral("property"),
                                    QStringLiteral("readonly"),   QStringLiteral("required"),
                                    QStringLiteral("return"),     QStringLiteral("signal"),
                                    QStringLiteral("switch"),     QStringLiteral("this"),
                                    QStringLiteral("throw"),      QStringLiteral("true"),
                                    QStringLiteral("try"),        QStringLiteral("typeof"),
                                    QStringLiteral("undefined"),  QStringLiteral("var"),
                                    QStringLiteral("while")}),
                    keywordFormat);
        }
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(singleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("//[^\n]*")), commentFormat);
        m_blockStart = QRegularExpression(QStringLiteral("/\\*"));
        m_blockEnd = QRegularExpression(QStringLiteral("\\*/"));
        m_blockFormat = commentFormat;
        m_hasBlockSpans = true;
    }
    else if (m_language == QStringLiteral("python")) {
        addRule(
            keywordPattern(
                {QStringLiteral("and"),      QStringLiteral("as"),       QStringLiteral("assert"),
                 QStringLiteral("async"),    QStringLiteral("await"),    QStringLiteral("break"),
                 QStringLiteral("class"),    QStringLiteral("continue"), QStringLiteral("def"),
                 QStringLiteral("del"),      QStringLiteral("elif"),     QStringLiteral("else"),
                 QStringLiteral("except"),   QStringLiteral("False"),    QStringLiteral("finally"),
                 QStringLiteral("for"),      QStringLiteral("from"),     QStringLiteral("global"),
                 QStringLiteral("if"),       QStringLiteral("import"),   QStringLiteral("in"),
                 QStringLiteral("is"),       QStringLiteral("lambda"),   QStringLiteral("None"),
                 QStringLiteral("nonlocal"), QStringLiteral("not"),      QStringLiteral("or"),
                 QStringLiteral("pass"),     QStringLiteral("raise"),    QStringLiteral("return"),
                 QStringLiteral("True"),     QStringLiteral("try"),      QStringLiteral("while"),
                 QStringLiteral("with"),     QStringLiteral("yield")}),
            keywordFormat);
        addRule(QRegularExpression(QStringLiteral("@\\w[\\w.]*")), metaFormat);
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(singleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("#[^\n]*")), commentFormat);
        m_blockStart = QRegularExpression(QStringLiteral("\"\"\""));
        m_blockEnd = QRegularExpression(QStringLiteral("\"\"\""));
        m_blockFormat = stringFormat;
        m_hasBlockSpans = true;
    }
    else if (m_language == QStringLiteral("cmake")) {
        addRule(keywordPattern({QStringLiteral("add_executable"),
                                QStringLiteral("add_library"),
                                QStringLiteral("add_subdirectory"),
                                QStringLiteral("cmake_minimum_required"),
                                QStringLiteral("else"),
                                QStringLiteral("elseif"),
                                QStringLiteral("endforeach"),
                                QStringLiteral("endfunction"),
                                QStringLiteral("endif"),
                                QStringLiteral("find_package"),
                                QStringLiteral("foreach"),
                                QStringLiteral("function"),
                                QStringLiteral("if"),
                                QStringLiteral("include"),
                                QStringLiteral("install"),
                                QStringLiteral("list"),
                                QStringLiteral("message"),
                                QStringLiteral("option"),
                                QStringLiteral("project"),
                                QStringLiteral("set"),
                                QStringLiteral("target_compile_options"),
                                QStringLiteral("target_include_directories"),
                                QStringLiteral("target_link_libraries")}),
                keywordFormat);
        addRule(QRegularExpression(QStringLiteral("\\$\\{[^}]*\\}")), metaFormat);
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("#[^\n]*")), commentFormat);
    }
    else if (m_language == QStringLiteral("toml")) {
        addRule(QRegularExpression(QStringLiteral("^\\s*\\[[^\\]]*\\]")), metaFormat);
        addRule(QRegularExpression(QStringLiteral("^\\s*[\\w.-]+(?=\\s*=)")), keywordFormat);
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(singleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("#[^\n]*")), commentFormat);
    }
    else if (m_language == QStringLiteral("json")) {
        addRule(QRegularExpression(QStringLiteral("\"(?:\\\\.|[^\"\\\\])*\"(?=\\s*:)")),
                keywordFormat);
        addRule(numberPattern(), numberFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(keywordPattern(
                    {QStringLiteral("true"), QStringLiteral("false"), QStringLiteral("null")}),
                numberFormat);
    }
    else if (m_language == QStringLiteral("shell")) {
        addRule(keywordPattern(
                    {QStringLiteral("case"), QStringLiteral("do"), QStringLiteral("done"),
                     QStringLiteral("elif"), QStringLiteral("else"), QStringLiteral("esac"),
                     QStringLiteral("exit"), QStringLiteral("fi"), QStringLiteral("for"),
                     QStringLiteral("function"), QStringLiteral("if"), QStringLiteral("in"),
                     QStringLiteral("local"), QStringLiteral("return"), QStringLiteral("set"),
                     QStringLiteral("then"), QStringLiteral("while")}),
                keywordFormat);
        addRule(QRegularExpression(QStringLiteral("\\$\\{?\\w+\\}?")), metaFormat);
        addRule(doubleQuoteString(), stringFormat);
        addRule(singleQuoteString(), stringFormat);
        addRule(QRegularExpression(QStringLiteral("#[^\n]*")), commentFormat);
    }
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
    applySemanticSpans();
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
        m_semanticSpansByLine[line].append(span);
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

QTextCharFormat EditorHighlighter::formatForSemanticKind(const QString& kind)
{
    if (kind == u"function" || kind == u"method") {
        return colorFormat(kFunctionRgb);
    }
    if (kind == u"namespace" || kind == u"type" || kind == u"class" || kind == u"enum" ||
        kind == u"interface" || kind == u"struct" || kind == u"typeParameter")
    {
        return colorFormat(kTypeRgb);
    }
    if (kind == u"parameter") {
        return colorFormat(kVariableRgb, false, true);
    }
    if (kind == u"variable") {
        return colorFormat(kVariableRgb);
    }
    if (kind == u"property" || kind == u"enumMember" || kind == u"event") {
        return colorFormat(kPropertyRgb);
    }
    if (kind == u"macro" || kind == u"decorator") {
        return colorFormat(kMetaRgb);
    }
    if (kind == u"keyword" || kind == u"modifier") {
        return colorFormat(kKeywordRgb, true);
    }
    if (kind == u"comment") {
        return colorFormat(kCommentRgb, false, true);
    }
    if (kind == u"string" || kind == u"regexp") {
        return colorFormat(kStringRgb);
    }
    if (kind == u"number") {
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

} // namespace kernwerk
