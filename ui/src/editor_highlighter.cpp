// O COMPOSITOR do realce: ciclo de vida, documento, lingua e a ordem em que
// as camadas se sobrepoem num bloco.
//
// Este arquivo era 910/500, o maior do repositorio, e foi cortado por
// responsabilidade em 2026-09-03 (etapa 16 do roadmaps/34). O que sobrou aqui
// e' a composicao; cada camada tem dono proprio:
//
//   editor_highlighter_rules.cpp     regex por linguagem (o piso)
//   editor_highlighter_tokens.cpp    Tree-sitter e LSP (vem de fora)
//   editor_highlighter_overlays.cpp  busca e diagnosticos (vem da sessao)
//   editor_highlighter_folding.cpp   dobra — visibilidade, NAO realce
//   editor_highlighter_palette.h     a paleta, com dono unico
#include "editor_highlighter.h"
#include "editor_highlighter_palette.h"

#include <algorithm>

#include <QFileInfo>
#include <QTextBlock>
#include <QTextDocument>

namespace kinein {

using namespace kinein::highlight;

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
    resetFolding();
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
    resetFolding();
    m_filePath = filePath;
    m_syntaxSpansByLine.clear();
    m_semanticSpansByLine.clear();
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
