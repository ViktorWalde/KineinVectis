#include "editor_highlighter.h"

#include <algorithm>

#include <QColor>
#include <QFileInfo>
#include <QRegularExpressionMatch>
#include <QRegularExpressionMatchIterator>
#include <QStringList>
#include <QTextBlock>
#include <QTextDocument>

namespace kinein {

namespace {

// Paleta de docs/05-design-system.md, em QRgb para inicializacao constexpr.
constexpr QRgb kKeywordRgb = 0xffffbb00; // accent
constexpr QRgb kStringRgb = 0xff7fbf7f;  // success soft
constexpr QRgb kCommentRgb = 0xff8f8a7c; // text muted
constexpr QRgb kNumberRgb = 0xff7aa2d8;  // info soft
constexpr QRgb kMetaRgb = 0xffd16d6d;    // error soft (macros, atributos, secoes)

// Find/Replace (D1b): fundo de TODAS as ocorrencias e da ocorrencia ATUAL.
// Ambar da paleta em duas intensidades — a atual "acende" sob o cursor.
constexpr QRgb kSearchMatchRgb = 0xff4a3a12;   // accent bem rebaixado
constexpr QRgb kSearchCurrentRgb = 0xff8a6a1a; // accent dim, ocorrencia atual

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
    // Realce de biblioteca padrao (fallback pre-LSP; semantic tokens
    // refinam depois). Reusa os tokens de docs/05: tipos=teal, callables
    // da stdlib=gold — mesmas cores dos semantic tokens de tipo/funcao.
    const QTextCharFormat typeFormat = colorFormat(kTypeRgb);
    const QTextCharFormat stdlibFormat = colorFormat(kFunctionRgb);

    const auto addRule = [this](const QRegularExpression& pattern, const QTextCharFormat& format) {
        m_rules.append(Rule{.pattern = pattern, .format = format});
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
        // Tipos primitivos e da std (fallback antes do rust-analyzer).
        addRule(
            keywordPattern(
                {QStringLiteral("i8"),      QStringLiteral("i16"),      QStringLiteral("i32"),
                 QStringLiteral("i64"),     QStringLiteral("i128"),     QStringLiteral("isize"),
                 QStringLiteral("u8"),      QStringLiteral("u16"),      QStringLiteral("u32"),
                 QStringLiteral("u64"),     QStringLiteral("u128"),     QStringLiteral("usize"),
                 QStringLiteral("f32"),     QStringLiteral("f64"),      QStringLiteral("bool"),
                 QStringLiteral("char"),    QStringLiteral("str"),      QStringLiteral("String"),
                 QStringLiteral("Vec"),     QStringLiteral("Option"),   QStringLiteral("Result"),
                 QStringLiteral("Box"),     QStringLiteral("Rc"),       QStringLiteral("Arc"),
                 QStringLiteral("Cell"),    QStringLiteral("RefCell"),  QStringLiteral("HashMap"),
                 QStringLiteral("HashSet"), QStringLiteral("BTreeMap"), QStringLiteral("VecDeque"),
                 QStringLiteral("Cow"),     QStringLiteral("Path"),     QStringLiteral("PathBuf"),
                 QStringLiteral("Some"),    QStringLiteral("None"),     QStringLiteral("Ok"),
                 QStringLiteral("Err")}),
            typeFormat);
        // Invocacao de macro (println!/vec!/format!/write!...): o "!"
        // depois de um identificador e macro (nunca negacao, que e prefixa).
        addRule(QRegularExpression(QStringLiteral("\\b[A-Za-z_]\\w*!")), metaFormat);
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
            // Tipos/containers da std (raramente nome de variavel do
            // usuario; semantic token de "class" concorda).
            addRule(
                keywordPattern({QStringLiteral("string"),        QStringLiteral("wstring"),
                                QStringLiteral("string_view"),   QStringLiteral("vector"),
                                QStringLiteral("array"),         QStringLiteral("deque"),
                                QStringLiteral("list"),          QStringLiteral("forward_list"),
                                QStringLiteral("map"),           QStringLiteral("multimap"),
                                QStringLiteral("unordered_map"), QStringLiteral("unordered_set"),
                                QStringLiteral("set"),           QStringLiteral("multiset"),
                                QStringLiteral("pair"),          QStringLiteral("tuple"),
                                QStringLiteral("optional"),      QStringLiteral("variant"),
                                QStringLiteral("span"),          QStringLiteral("queue"),
                                QStringLiteral("stack"),         QStringLiteral("shared_ptr"),
                                QStringLiteral("unique_ptr"),    QStringLiteral("weak_ptr"),
                                QStringLiteral("function"),      QStringLiteral("initializer_list"),
                                QStringLiteral("ostream"),       QStringLiteral("istream"),
                                QStringLiteral("stringstream"),  QStringLiteral("ifstream"),
                                QStringLiteral("ofstream"),      QStringLiteral("fstream"),
                                QStringLiteral("size_t"),        QStringLiteral("ssize_t"),
                                QStringLiteral("ptrdiff_t"),     QStringLiteral("intptr_t"),
                                QStringLiteral("uintptr_t"),     QStringLiteral("int8_t"),
                                QStringLiteral("int16_t"),       QStringLiteral("int32_t"),
                                QStringLiteral("int64_t"),       QStringLiteral("uint8_t"),
                                QStringLiteral("uint16_t"),      QStringLiteral("uint32_t"),
                                QStringLiteral("uint64_t"),      QStringLiteral("wchar_t"),
                                QStringLiteral("char8_t"),       QStringLiteral("char16_t"),
                                QStringLiteral("char32_t")}),
                typeFormat);
            // Streams e funcoes iconicas da stdlib (I/O, memoria, string).
            addRule(keywordPattern({QStringLiteral("cout"),        QStringLiteral("cin"),
                                    QStringLiteral("cerr"),        QStringLiteral("clog"),
                                    QStringLiteral("wcout"),       QStringLiteral("wcin"),
                                    QStringLiteral("wcerr"),       QStringLiteral("wclog"),
                                    QStringLiteral("endl"),        QStringLiteral("flush"),
                                    QStringLiteral("printf"),      QStringLiteral("fprintf"),
                                    QStringLiteral("sprintf"),     QStringLiteral("snprintf"),
                                    QStringLiteral("scanf"),       QStringLiteral("sscanf"),
                                    QStringLiteral("fscanf"),      QStringLiteral("puts"),
                                    QStringLiteral("fputs"),       QStringLiteral("putchar"),
                                    QStringLiteral("getchar"),     QStringLiteral("fgets"),
                                    QStringLiteral("fopen"),       QStringLiteral("fclose"),
                                    QStringLiteral("fread"),       QStringLiteral("fwrite"),
                                    QStringLiteral("malloc"),      QStringLiteral("calloc"),
                                    QStringLiteral("realloc"),     QStringLiteral("free"),
                                    QStringLiteral("memcpy"),      QStringLiteral("memmove"),
                                    QStringLiteral("memset"),      QStringLiteral("memcmp"),
                                    QStringLiteral("strlen"),      QStringLiteral("strcmp"),
                                    QStringLiteral("strncmp"),     QStringLiteral("strcpy"),
                                    QStringLiteral("strncpy"),     QStringLiteral("strcat"),
                                    QStringLiteral("strstr"),      QStringLiteral("strchr"),
                                    QStringLiteral("atoi"),        QStringLiteral("atol"),
                                    QStringLiteral("atof"),        QStringLiteral("exit"),
                                    QStringLiteral("abort"),       QStringLiteral("make_shared"),
                                    QStringLiteral("make_unique"), QStringLiteral("make_pair"),
                                    QStringLiteral("make_tuple")}),
                    stdlibFormat);
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
    // stdlib. Único override PÓS-semantic de propósito (docs-privada/diario/18, CR1).
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
