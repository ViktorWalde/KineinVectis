// Syntax highlighting local do editor.
//
// Colore keywords, strings, comentarios, numeros e metadados por regex,
// usando a paleta de docs/05-design-system.md. Cores semanticas mais ricas
// virao do LSP (Fase 5); este highlighter continua como base offline.

#pragma once

#include <QHash>
#include <QList>
#include <QQuickTextDocument>
#include <QRegularExpression>
#include <QString>
#include <QSyntaxHighlighter>
#include <QTextCharFormat>
#include <QVariantList>
#include <QtQml/qqmlregistration.h>

namespace kernwerk {

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

signals:
    void documentChanged();
    void filePathChanged();

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

    void rebuildRules();
    void applyBlockSpans(const QString& text);
    void applySemanticSpans();
    [[nodiscard]] static QString languageForPath(const QString& filePath);
    [[nodiscard]] static QTextCharFormat formatForSemanticKind(const QString& kind);

    QQuickTextDocument* m_quickDocument = nullptr;
    QString m_filePath;
    QString m_language;
    QList<Rule> m_rules;
    QRegularExpression m_blockStart;
    QRegularExpression m_blockEnd;
    QTextCharFormat m_blockFormat;
    bool m_hasBlockSpans = false;
    QHash<int, QList<SemanticSpan>> m_semanticSpansByLine;
};

} // namespace kernwerk
