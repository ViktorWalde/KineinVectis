#pragma once

#include <QColor>
#include <QImage>
#include <QObject>
#include <QQuickTextDocument>
#include <QString>
#include <QStringList>
#include <QTextDocument>
#include <QUrl>
#include <QVariantMap>
#include <QtQml/qqmlregistration.h>

// A PONTE DO PREVIEW DE MARKDOWN (fatia V5, 2026-09-25).
//
// Pequena de proposito. A §2 da especificacao mediu que o Qt 6.4 ja' renderiza
// Markdown dialeto GitHub (`QTextDocument::setMarkdown`), entao o primeiro
// corte NAO precisa de Chromium, WebEngine, servidor local nem parser novo — e
// o AppImage nao ganha dependencia nenhuma.
//
// O que existe aqui e nao existe no `TextEdit.MarkdownText` do QML sao as duas
// travas da §5: `MarkdownNoHTML`, que desliga HTML embutido (script, iframe,
// pixel remoto), e o RESOURCE PROVIDER, que decide arquivo por arquivo o que o
// documento pode ler. Sem provider, um `.md` de terceiros le' o que quiser do
// disco pelo caminho que escrever numa imagem.
//
// Nenhuma decisao mora aqui: elas estao no `markdown_policy`, que tem teste.
// Esta classe so' liga o veredito ao documento.
namespace kinein {

class MarkdownDocument : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickTextDocument* target READ target WRITE setTarget NOTIFY targetChanged)
    Q_PROPERTY(QString markdown READ markdown WRITE setMarkdown NOTIFY markdownChanged)
    Q_PROPERTY(
        QString documentPath READ documentPath WRITE setDocumentPath NOTIFY documentPathChanged)
    Q_PROPERTY(
        QString workspaceRoot READ workspaceRoot WRITE setWorkspaceRoot NOTIFY workspaceRootChanged)
    // §4.2: a preferencia futura existe, e na 0.3 e' sempre falsa.
    Q_PROPERTY(bool remoteImagesAllowed READ remoteImagesAllowed WRITE setRemoteImagesAllowed NOTIFY
                   remoteImagesAllowedChanged)
    // AS CORES DO TEMA, aplicadas como FORMATO e nao como CSS.
    //
    // A primeira versao passava uma folha de estilo por
    // `setDefaultStyleSheet`, e a foto mostrou que ela nao fazia NADA: o
    // `setMarkdown` do Qt monta blocos com propriedades proprias
    // (`BlockQuoteLevel`, `BlockCodeLanguage`), e nao elementos HTML — entao
    // seletor CSS nao encontra nada para casar. Uma propriedade que parece
    // funcionar e nao funciona e' pior que a ausencia dela.
    Q_PROPERTY(QColor linkColor READ linkColor WRITE setLinkColor NOTIFY paletteChanged)
    Q_PROPERTY(QColor quoteColor READ quoteColor WRITE setQuoteColor NOTIFY paletteChanged)
    Q_PROPERTY(
        QColor codeBackground READ codeBackground WRITE setCodeBackground NOTIFY paletteChanged)
    // O que foi RECUSADO, para a tela poder dizer em vez de sumir em silencio.
    Q_PROPERTY(QStringList blockedResources READ blockedResources NOTIFY blockedResourcesChanged)

public:
    explicit MarkdownDocument(QObject* parent = nullptr);

    [[nodiscard]] QQuickTextDocument* target() const;
    void setTarget(QQuickTextDocument* target);
    [[nodiscard]] QString markdown() const;
    void setMarkdown(const QString& markdown);
    [[nodiscard]] QString documentPath() const;
    void setDocumentPath(const QString& path);
    [[nodiscard]] QString workspaceRoot() const;
    void setWorkspaceRoot(const QString& root);
    [[nodiscard]] bool remoteImagesAllowed() const;
    void setRemoteImagesAllowed(bool allowed);
    [[nodiscard]] QColor linkColor() const;
    void setLinkColor(const QColor& color);
    [[nodiscard]] QColor quoteColor() const;
    void setQuoteColor(const QColor& color);
    [[nodiscard]] QColor codeBackground() const;
    void setCodeBackground(const QColor& color);
    [[nodiscard]] QStringList blockedResources() const;

    // O clique num link: o QML pergunta o que fazer, e a resposta vem da
    // politica pura. { kind: "anchor"|"localFile"|"web"|"refused",
    //                  target: "...", reason: "..." }
    [[nodiscard]] Q_INVOKABLE QVariantMap decideLink(const QString& href) const;

signals:
    void targetChanged();
    void markdownChanged();
    void documentPathChanged();
    void workspaceRootChanged();
    void remoteImagesAllowedChanged();
    void paletteChanged();
    void blockedResourcesChanged();

private:
    void applyToDocument();
    void applyPalette(QTextDocument* document);
    [[nodiscard]] QString sanitizeSource(const QString& markdown);
    void sanitizeImages(QTextDocument* document);
    [[nodiscard]] QString imageRefusal(const QString& name);
    void installProvider();
    [[nodiscard]] QVariant provideResource(const QUrl& url);
    [[nodiscard]] QString shortName(const QString& path) const;
    void noteBlocked(const QString& description);

    QQuickTextDocument* m_target = nullptr;
    QString m_markdown;
    QString m_documentPath;
    QString m_workspaceRoot;
    bool m_remoteImagesAllowed = false;
    QColor m_linkColor;
    QColor m_quoteColor;
    QColor m_codeBackground;
    QStringList m_blockedResources;
};

} // namespace kinein
