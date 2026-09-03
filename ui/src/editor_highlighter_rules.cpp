// Regras de realce por REGEX — o fallback quando nao ha Tree-sitter nem LSP.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-03, etapa 16). `rebuildRules` sozinho
// tinha 274 linhas: e' a TABELA de padroes por linguagem, nao logica de
// composicao. Tabela cresce quando uma linguagem nova entra, e por isso ela
// tem que crescer LONGE do compositor — senao adicionar uma linguagem empurra
// o arquivo do realce para a catraca de novo.
#include "editor_highlighter.h"
#include "editor_highlighter_palette.h"

#include <QFileInfo>

namespace kinein {

using namespace kinein::highlight;

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

} // namespace kinein
