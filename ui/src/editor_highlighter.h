// Syntax highlighting local do editor.
//
// Colore keywords, strings, comentarios, numeros e metadados por regex,
// usando a paleta de DocsPublic/05-design-system.md. Cores semanticas mais ricas
// virao do LSP (Fase 5); este highlighter continua como base offline.

#pragma once

#include <QHash>
#include <QList>
#include <QQuickTextDocument>
#include <QRegularExpression>
#include <QSet>
#include <QString>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class EditorHighlighter : public QSyntaxHighlighter
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickTextDocument* document READ quickDocument WRITE setQuickDocument NOTIFY
                   documentChanged)
    Q_PROPERTY(QString filePath READ filePath WRITE setFilePath NOTIFY filePathChanged)
    Q_PROPERTY(QString language READ language NOTIFY filePathChanged)

public:
    explicit EditorHighlighter(QObject* parent = nullptr);

    [[nodiscard]] QQuickTextDocument* quickDocument() const;
    void setQuickDocument(QQuickTextDocument* document);
    [[nodiscard]] QString filePath() const;
    void setFilePath(const QString& filePath);
    [[nodiscard]] QString language() const;

    // Tokens semanticos vindos do LSP: [{line (1-based), start, length
    // (UTF-16), kind}]. Substituem o realce por regex onde existirem.
    Q_INVOKABLE void setSemanticTokens(const QVariantList& tokens);
    Q_INVOKABLE void clearSemanticTokens();

    // Captures estruturais Tree-sitter: mesma geometria UTF-16 dos tokens
    // semanticos, com `scope` no lugar de `kind`. Formam a camada entre o
    // fallback regex e os semantic tokens do LSP.
    Q_INVOKABLE void setSyntaxTokens(const QVariantList& tokens);
    Q_INVOKABLE void clearSyntaxTokens();

    // Folding estrutural (linhas 1-based). Os blocos internos sao ocultados
    // no QTextDocument, sem alterar o buffer.
    Q_INVOKABLE void setFoldingRanges(const QVariantList& ranges);
    Q_INVOKABLE bool toggleFoldAtLine(int line);
    Q_INVOKABLE bool isFoldableLine(int line) const;
    Q_INVOKABLE bool isFoldedLine(int line) const;
    Q_INVOKABLE QVariantList visibleLineNumbers() const;

    // Diagnosticos do arquivo ativo (T6): [{startLine, startChar,
    // endLine, endChar (0-based UTF-16), severity}]. Desenham sublinhado
    // ondulado por severidade sobre o range, por cima do realce.
    Q_INVOKABLE void setDiagnostics(const QVariantList& diagnostics);
    Q_INVOKABLE void clearDiagnostics();

    // Ocorrencias do Find/Replace (D1b): [{start, end}] em offsets ABSOLUTOS
    // (UTF-16) do documento, mais o indice da ocorrencia ATUAL. Todas ganham
    // fundo; a atual, um fundo mais forte. Offsets absolutos (em vez de
    // linha/coluna) porque e o que o scanner da UI ja produz.
    Q_INVOKABLE void setSearchMatches(const QVariantList& matches, int current);

signals:
    void documentChanged();
    void filePathChanged();
    void foldingChanged();

protected:
    void highlightBlock(const QString& text) override;

private:
    struct Rule
    {
        QRegularExpression pattern;
        QTextCharFormat format;
    };

    struct SemanticSpan
    {
        int start = 0;
        int length = 0;
        QTextCharFormat format;
    };

    // Um diagnostico com range absoluto em linhas/caracteres 0-based
    // UTF-16 (o span por bloco e derivado em applyDiagnosticSpans).
    struct DiagnosticSpan
    {
        int startLine = 0;
        int startChar = 0;
        int endLine = 0;
        int endChar = 0;
        QString severity;
    };

    struct FoldingRange
    {
        int startLine = 0;
        int endLine = 0;
    };

    // Ocorrencia do Find em offsets absolutos do documento.
    struct SearchSpan
    {
        int start = 0;
        int end = 0;
    };

    void rebuildRules();
    void applyBlockSpans(const QString& text);
    void applySyntaxSpans();
    void applySemanticSpans();
    void applyStdlibOverride(const QString& text);
    void applyDiagnosticSpans(const QString& text);
    void applySearchSpans(const QString& text);
    [[nodiscard]] static QString languageForPath(const QString& filePath);
    [[nodiscard]] static QTextCharFormat formatForSemanticKind(const QString& kind);
    [[nodiscard]] static QTextCharFormat formatForSyntaxScope(const QString& scope);
    [[nodiscard]] static QTextCharFormat underlineFormatForSeverity(const QString& severity);
    void applyFoldVisibility();
    void showAllBlocks();
    // Esquece a dobra do documento anterior. Dono unico do "limpa tudo de
    // dobra": o compositor nao mexe em m_foldingRanges/m_foldedStartLines.
    void resetFolding();

    QQuickTextDocument* m_quickDocument = nullptr;
    QString m_filePath;
    QString m_language;
    QList<Rule> m_rules;
    QRegularExpression m_blockStart;
    QRegularExpression m_blockEnd;
    QTextCharFormat m_blockFormat;
    bool m_hasBlockSpans = false;
    QHash<int, QList<SemanticSpan>> m_syntaxSpansByLine;
    QHash<int, QList<SemanticSpan>> m_semanticSpansByLine;
    QList<FoldingRange> m_foldingRanges;
    QSet<int> m_foldedStartLines;
    QList<DiagnosticSpan> m_diagnostics;
    QList<SearchSpan> m_searchMatches;
    int m_currentSearchMatch = -1;
};

} // namespace kinein
