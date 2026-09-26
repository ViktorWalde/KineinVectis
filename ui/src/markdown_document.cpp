#include "markdown_document.h"

#include "markdown_policy.h"

#include <QDir>
#include <QFileInfo>
#include <QImageReader>
#include <QList>
#include <QRegularExpression>
#include <QTextBlock>
#include <QTextCursor>
#include <QTextDocument>

#include <ranges>

namespace kinein {

namespace {

// Um `.md` nao decide quanta memoria a IDE gasta. A §7 pede limite de
// dimensoes e de memoria para imagem; estes sao os tetos do primeiro corte.
constexpr int kMaxImageSide = 4096;
constexpr qint64 kMaxImageBytes = 16LL * 1024 * 1024;
constexpr int kMaxBlockedRemembered = 32;

QString kindName(markdown::LinkKind kind)
{
    switch (kind) {
    case markdown::LinkKind::Anchor:
        return QStringLiteral("anchor");
    case markdown::LinkKind::LocalFile:
        return QStringLiteral("localFile");
    case markdown::LinkKind::Web:
        return QStringLiteral("web");
    case markdown::LinkKind::Refused:
        return QStringLiteral("refused");
    }
    return QStringLiteral("refused");
}

} // namespace

MarkdownDocument::MarkdownDocument(QObject* parent) : QObject(parent) {}

QQuickTextDocument* MarkdownDocument::target() const
{
    return m_target;
}

void MarkdownDocument::setTarget(QQuickTextDocument* target)
{
    if (m_target == target) {
        return;
    }
    m_target = target;
    installProvider();
    applyToDocument();
    emit targetChanged();
}

QString MarkdownDocument::markdown() const
{
    return m_markdown;
}

void MarkdownDocument::setMarkdown(const QString& markdown)
{
    if (m_markdown == markdown) {
        return;
    }
    m_markdown = markdown;
    applyToDocument();
    emit markdownChanged();
}

QString MarkdownDocument::documentPath() const
{
    return m_documentPath;
}

void MarkdownDocument::setDocumentPath(const QString& path)
{
    if (m_documentPath == path) {
        return;
    }
    m_documentPath = path;
    applyToDocument();
    emit documentPathChanged();
}

QString MarkdownDocument::workspaceRoot() const
{
    return m_workspaceRoot;
}

void MarkdownDocument::setWorkspaceRoot(const QString& root)
{
    if (m_workspaceRoot == root) {
        return;
    }
    m_workspaceRoot = root;
    emit workspaceRootChanged();
}

bool MarkdownDocument::remoteImagesAllowed() const
{
    return m_remoteImagesAllowed;
}

void MarkdownDocument::setRemoteImagesAllowed(bool allowed)
{
    if (m_remoteImagesAllowed == allowed) {
        return;
    }
    m_remoteImagesAllowed = allowed;
    emit remoteImagesAllowedChanged();
}

QColor MarkdownDocument::linkColor() const
{
    return m_linkColor;
}

void MarkdownDocument::setLinkColor(const QColor& color)
{
    if (m_linkColor == color) {
        return;
    }
    m_linkColor = color;
    applyToDocument();
    emit paletteChanged();
}

QColor MarkdownDocument::quoteColor() const
{
    return m_quoteColor;
}

void MarkdownDocument::setQuoteColor(const QColor& color)
{
    if (m_quoteColor == color) {
        return;
    }
    m_quoteColor = color;
    applyToDocument();
    emit paletteChanged();
}

QColor MarkdownDocument::codeBackground() const
{
    return m_codeBackground;
}

void MarkdownDocument::setCodeBackground(const QColor& color)
{
    if (m_codeBackground == color) {
        return;
    }
    m_codeBackground = color;
    applyToDocument();
    emit paletteChanged();
}

QStringList MarkdownDocument::blockedResources() const
{
    return m_blockedResources;
}

QVariantMap MarkdownDocument::decideLink(const QString& href) const
{
    const markdown::LinkDecision decision =
        markdown::decideLink(href, m_documentPath, m_workspaceRoot);
    QVariantMap result;
    result.insert(QStringLiteral("kind"), kindName(decision.kind));
    result.insert(QStringLiteral("target"), decision.target);
    result.insert(QStringLiteral("reason"), decision.reason);
    return result;
}

void MarkdownDocument::installProvider()
{
    if (m_target == nullptr || m_target->textDocument() == nullptr) {
        return;
    }
    // TODA leitura de recurso passa por aqui. Sem provider, o QTextDocument
    // resolve `file:` sozinho contra a baseUrl — e ai' o documento manda.
    m_target->textDocument()->setResourceProvider(
        [this](const QUrl& url) -> QVariant { return provideResource(url); });
}

void MarkdownDocument::applyToDocument()
{
    if (m_target == nullptr) {
        return;
    }
    QTextDocument* document = m_target->textDocument();
    if (document == nullptr) {
        return;
    }
    if (!m_blockedResources.isEmpty()) {
        m_blockedResources.clear();
        emit blockedResourcesChanged();
    }
    // A baseUrl e' a PASTA do `.md`: e' contra ela que caminho relativo
    // resolve. Ela nao autoriza nada sozinha — quem autoriza e' o provider.
    if (!m_documentPath.isEmpty()) {
        document->setBaseUrl(
            QUrl::fromLocalFile(QFileInfo(m_documentPath).absolutePath() + QLatin1Char('/')));
    }
    // MarkdownNoHTML e' a trava da §5: HTML embutido nao passa. Sem ele, um
    // `.md` traz `<script>`, `<iframe>` e pixel remoto para dentro da IDE.
    // O `|` de dois valores do enum vira `int`, e o Qt nao declara o operador
    // de flags para este; montar o QFlags e' o caminho que compila sob -Werror.
    QTextDocument::MarkdownFeatures features = QTextDocument::MarkdownDialectGitHub;
    features |= QTextDocument::MarkdownNoHTML;
    document->setMarkdown(sanitizeSource(m_markdown), features);
    sanitizeImages(document);
    applyPalette(document);
}

// O PORTAO DE VERDADE: a URL proibida nao chega ao renderer.
//
// A costura no texto mora no `markdown_policy`, que e' puro e tem teste; aqui
// fica so' a decisao, que precisa do disco.
QString MarkdownDocument::sanitizeSource(const QString& markdown)
{
    return markdown::rewriteRefusedImages(
        markdown, [this](const QString& name) { return imageRefusal(name); });
}

// O SEGUNDO PORTAO, para o que o primeiro nao alcanca: imagem por referencia
// (`![alt][ref]`), que o texto acima nao reescreve. Aqui ela ja' foi lida, mas
// pelo menos nao aparece — e o motivo e' dito.
//
// Por que o resource provider NAO serve de portao
// (medido em 2026-09-25, com a foto na mao).
//
// `QTextDocument::setResourceProvider` parecia o lugar certo: toda leitura de
// recurso passaria por ele. NAO PASSA. O `TextEdit` do Qt Quick usa um
// documento proprio (`QQuickTextDocumentWithImageResources`) que carrega a
// imagem SOZINHO quando consegue, e so' cai no `loadResource` — e portanto no
// provider — quando o carregamento dele falha.
//
// Medido: com tres imagens no documento (uma ausente, uma dentro do projeto e
// uma FORA), o provider foi chamado tres vezes, todas para a AUSENTE. A de
// fora do projeto apareceu na tela. O provider e' fallback, nao portao.
//
// O portao de verdade e' este: a URL proibida nao pode EXISTIR no documento.
// Depois do render, cada fragmento de imagem passa pela politica e, quando
// recusado, vira TEXTO com o motivo — que e' o alt text que a §4.2 pede.
void MarkdownDocument::sanitizeImages(QTextDocument* document)
{
    struct Refusal
    {
        int position;
        int length;
        QString text;
    };
    QList<Refusal> refusals;

    for (QTextBlock block = document->begin(); block.isValid(); block = block.next()) {
        for (QTextBlock::iterator it = block.begin(); !it.atEnd(); ++it) {
            const QTextFragment fragment = it.fragment();
            if (!fragment.isValid() || !fragment.charFormat().isImageFormat()) {
                continue;
            }
            const QString name = fragment.charFormat().toImageFormat().name();
            const QString refusal = imageRefusal(name);
            if (refusal.isEmpty()) {
                continue;
            }
            refusals.append(
                {.position = fragment.position(), .length = fragment.length(), .text = refusal});
        }
    }

    // DE TRAS PARA A FRENTE: cada substituicao muda o tamanho do documento, e
    // as posicoes colhidas antes valeriam para o texto de antes.
    for (const Refusal& refusal : std::views::reverse(refusals)) {
        QTextCursor cursor(document);
        cursor.setPosition(refusal.position);
        cursor.setPosition(refusal.position + refusal.length, QTextCursor::KeepAnchor);
        QTextCharFormat plain;
        if (m_quoteColor.isValid()) {
            plain.setForeground(m_quoteColor);
        }
        cursor.insertText(refusal.text, plain);
    }
}

// Vazio quando a imagem pode ser carregada; o motivo, quando nao pode.
QString MarkdownDocument::imageRefusal(const QString& name)
{
    const QUrl url(name);
    const QString href = url.isLocalFile() ? url.toLocalFile() : name;
    const markdown::ResourceDecision decision =
        markdown::decideResource(href, m_documentPath, m_workspaceRoot, m_remoteImagesAllowed);
    if (!decision.allowed) {
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), decision.reason));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), decision.reason);
    }
    if (decision.path.startsWith(QLatin1String("http"))) {
        const QString motivo = QStringLiteral("imagem remota ainda não é buscada");
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), motivo));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), motivo);
    }

    const QFileInfo info(decision.path);
    if (!info.exists()) {
        const QString motivo = QStringLiteral("arquivo não encontrado");
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), motivo));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), motivo);
    }
    // A trava do SYMLINK: normalizar caminho nao o resolve, e um link dentro do
    // projeto apontando para fora passaria pela politica pura.
    const QString canonical = info.canonicalFilePath();
    if (canonical.isEmpty() || !markdown::insideRoot(canonical, m_workspaceRoot)) {
        const QString motivo = QStringLiteral("o caminho real sai do projeto");
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), motivo));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), motivo);
    }
    if (info.size() > kMaxImageBytes) {
        const QString motivo = QStringLiteral("imagem acima do limite de memória");
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), motivo));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), motivo);
    }
    QImageReader reader(canonical);
    const QSize size = reader.size();
    if (size.width() > kMaxImageSide || size.height() > kMaxImageSide) {
        const QString motivo = QStringLiteral("imagem acima de %1 px").arg(kMaxImageSide);
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), motivo));
        return QStringLiteral("[%1 — %2]").arg(shortName(href), motivo);
    }
    return {};
}

// O tema entra DEPOIS do conteudo, percorrendo os blocos que o `setMarkdown`
// montou. E' o unico caminho: as propriedades de bloco do Qt
// (`BlockQuoteLevel`, `BlockCodeLanguage`) nao sao elementos HTML, entao folha
// de estilo nao as alcanca — medido, com a foto na mao.
void MarkdownDocument::applyPalette(QTextDocument* document)
{
    QTextCursor cursor(document);
    cursor.beginEditBlock();
    for (QTextBlock block = document->begin(); block.isValid(); block = block.next()) {
        const QTextBlockFormat blockFormat = block.blockFormat();
        const bool isQuote = blockFormat.hasProperty(QTextFormat::BlockQuoteLevel);
        const bool isCode = blockFormat.hasProperty(QTextFormat::BlockCodeLanguage) ||
                            blockFormat.nonBreakableLines();

        if (isCode && m_codeBackground.isValid()) {
            QTextBlockFormat updated = blockFormat;
            updated.setBackground(m_codeBackground);
            // A margem existe para o fundo nao colar no texto: um bloco de
            // codigo sem respiro fica pior do que sem fundo nenhum.
            updated.setLeftMargin(8);
            updated.setRightMargin(8);
            cursor.setPosition(block.position());
            cursor.setBlockFormat(updated);
        }

        // A CITACAO e' o caso que a foto reprovou: sem cor, ela ficava
        // indistinguivel de um paragrafo indentado.
        if (isQuote && m_quoteColor.isValid()) {
            QTextCursor blockCursor(block);
            blockCursor.select(QTextCursor::BlockUnderCursor);
            QTextCharFormat charFormat;
            charFormat.setForeground(m_quoteColor);
            blockCursor.mergeCharFormat(charFormat);
        }

        if (!m_linkColor.isValid()) {
            continue;
        }
        // O LINK vinha no azul padrao do Qt, que nao e' o tema de ninguem.
        for (QTextBlock::iterator it = block.begin(); !it.atEnd(); ++it) {
            const QTextFragment fragment = it.fragment();
            if (!fragment.isValid() || !fragment.charFormat().isAnchor()) {
                continue;
            }
            QTextCursor linkCursor(document);
            linkCursor.setPosition(fragment.position());
            linkCursor.setPosition(fragment.position() + fragment.length(),
                                   QTextCursor::KeepAnchor);
            QTextCharFormat charFormat;
            charFormat.setForeground(m_linkColor);
            linkCursor.mergeCharFormat(charFormat);
        }
    }
    cursor.endEditBlock();
}

// O que a tarja mostra. Caminho absoluto de duzentos caracteres nao cabe, e nao
// e' o que o autor escreveu no documento: ele escreveu `img/logo.png`.
QString MarkdownDocument::shortName(const QString& path) const
{
    if (m_workspaceRoot.isEmpty() || !markdown::insideRoot(path, m_workspaceRoot)) {
        return path;
    }
    return QDir(m_workspaceRoot).relativeFilePath(path);
}

void MarkdownDocument::noteBlocked(const QString& description)
{
    if (m_blockedResources.contains(description)) {
        return;
    }
    if (m_blockedResources.size() >= kMaxBlockedRemembered) {
        return;
    }
    m_blockedResources.append(description);
    emit blockedResourcesChanged();
}

QVariant MarkdownDocument::provideResource(const QUrl& url)
{
    const QString href = url.isLocalFile() ? url.toLocalFile() : url.toString();
    const markdown::ResourceDecision decision =
        markdown::decideResource(href, m_documentPath, m_workspaceRoot, m_remoteImagesAllowed);
    if (!decision.allowed) {
        noteBlocked(QStringLiteral("%1 — %2").arg(shortName(href), decision.reason));
        return {};
    }
    if (decision.path.startsWith(QLatin1String("http"))) {
        // Remoto permitido pela preferencia ainda assim NAO e' buscado aqui:
        // rede tem dono proprio no core, e este provider e' sincrono, no
        // thread da UI. Ate' esse dono existir, a resposta honesta e' nao.
        noteBlocked(QStringLiteral("%1 — imagem remota ainda não é buscada").arg(shortName(href)));
        return {};
    }

    const QFileInfo info(decision.path);
    // ARQUIVO QUE NAO EXISTE NAO E' ARQUIVO PROIBIDO, e a primeira versao disto
    // dizia que era: `canonicalFilePath()` volta VAZIO para caminho
    // inexistente, e a tarja acusava "o caminho real sai do projeto" para uma
    // imagem que estava dentro dele, so' que ausente. Motivo errado e' pior que
    // motivo nenhum: manda procurar o problema no lugar errado.
    if (!info.exists()) {
        noteBlocked(QStringLiteral("%1 — arquivo não encontrado").arg(shortName(href)));
        return {};
    }
    // A SEGUNDA TRAVA, e ela existe porque a primeira nao ve' symlink: o
    // caminho CANONICO tem que continuar dentro da raiz. Um link dentro do
    // projeto apontando para `/etc` passa pela normalizacao e morre aqui.
    const QString canonical = info.canonicalFilePath();
    if (canonical.isEmpty() || !markdown::insideRoot(canonical, m_workspaceRoot)) {
        noteBlocked(QStringLiteral("%1 — o caminho real sai do projeto").arg(shortName(href)));
        return {};
    }
    if (info.size() > kMaxImageBytes) {
        noteBlocked(QStringLiteral("%1 — imagem acima do limite de memória").arg(shortName(href)));
        return {};
    }

    QImageReader reader(canonical);
    reader.setAutoTransform(true);
    const QSize size = reader.size();
    if (size.width() > kMaxImageSide || size.height() > kMaxImageSide) {
        noteBlocked(
            QStringLiteral("%1 — imagem acima de %2 px").arg(shortName(href)).arg(kMaxImageSide));
        return {};
    }
    QImage image = reader.read();
    if (image.isNull()) {
        noteBlocked(QStringLiteral("%1 — não é uma imagem legível").arg(shortName(href)));
        return {};
    }
    return image;
}

} // namespace kinein
