#include "markdown_policy.h"

#include <QDir>
#include <QFileInfo>
#include <QRegularExpression>
#include <QStringList>
#include <QUrl>

namespace kinein::markdown {

namespace {

// Os unicos esquemas que o preview reconhece. Qualquer outro (`javascript:`,
// `data:`, `vscode:`, `smb:`) e' recusado por nome, e nao por omissao: lista de
// permitidos, nunca de proibidos.
bool isWebScheme(const QString& scheme)
{
    return scheme == QLatin1String("http") || scheme == QLatin1String("https");
}

QString normalize(const QString& path)
{
    return QDir::cleanPath(path);
}

} // namespace

QString rewriteRefusedImages(const QString& markdown,
                             const std::function<QString(const QString&)>& refusal)
{
    // `![alt](alvo "titulo")` — a forma inline, que e' a que o texto carrega.
    // A forma por REFERENCIA (`![alt][ref]`) nao passa por aqui, e por isso
    // quem chama mantem um segundo portao depois do render.
    static const QRegularExpression image(
        QStringLiteral(R"(!\[([^\]]*)\]\(\s*([^)\s]+)(?:\s+"[^"]*")?\s*\))"));
    // Cerca de codigo: um exemplo de Markdown dentro de ``` e' CONTEUDO, e
    // reescreve-lo mudaria o codigo que o autor quis mostrar.
    static const QRegularExpression fence(QStringLiteral("^\\s*(```|~~~)"));

    const QStringList lines = markdown.split(QLatin1Char('\n'));
    QStringList out;
    out.reserve(lines.size());
    bool insideFence = false;
    for (const QString& line : lines) {
        if (fence.match(line).hasMatch()) {
            insideFence = !insideFence;
            out.append(line);
            continue;
        }
        if (insideFence) {
            out.append(line);
            continue;
        }
        QString rewritten;
        qsizetype last = 0;
        QRegularExpressionMatchIterator it = image.globalMatch(line);
        while (it.hasNext()) {
            const QRegularExpressionMatch match = it.next();
            const QString replacement = refusal(match.captured(2));
            if (replacement.isEmpty()) {
                continue;
            }
            rewritten += line.mid(last, match.capturedStart() - last);
            rewritten += replacement;
            last = match.capturedEnd();
        }
        out.append(last == 0 ? line : rewritten + line.mid(last));
    }
    return out.join(QLatin1Char('\n'));
}

bool insideRoot(const QString& absolutePath, const QString& root)
{
    if (root.isEmpty() || absolutePath.isEmpty()) {
        return false;
    }
    const QString cleanRoot = normalize(root);
    const QString target = normalize(absolutePath);
    if (target == cleanRoot) {
        return true;
    }
    // A barra no fim importa: `/casa/projeto` NAO pode autorizar
    // `/casa/projeto-de-outro`.
    const QString prefix =
        cleanRoot.endsWith(QLatin1Char('/')) ? cleanRoot : cleanRoot + QLatin1Char('/');
    return target.startsWith(prefix);
}

namespace {

// Resolve `href` contra a pasta do documento e devolve vazio se ele escapar da
// raiz. Serve ao link e ao recurso, que tem a mesma regra de escopo.
QString resolveInsideRoot(const QString& href, const QString& documentPath,
                          const QString& workspaceRoot)
{
    if (workspaceRoot.isEmpty()) {
        return {};
    }
    const QString base = QFileInfo(documentPath).absolutePath();
    const QString raw = QDir::isAbsolutePath(href) ? href : base + QLatin1Char('/') + href;
    const QString absolute = normalize(raw);
    return insideRoot(absolute, workspaceRoot) ? absolute : QString();
}

} // namespace

LinkDecision decideLink(const QString& href, const QString& documentPath,
                        const QString& workspaceRoot)
{
    const QString text = href.trimmed();
    if (text.isEmpty()) {
        return {.kind = LinkKind::Refused, .target = {}, .reason = QStringLiteral("link vazio")};
    }
    if (text.startsWith(QLatin1Char('#'))) {
        return {.kind = LinkKind::Anchor, .target = text.mid(1), .reason = {}};
    }

    const QUrl url(text);
    const QString scheme = url.scheme().toLower();
    if (isWebScheme(scheme)) {
        return {.kind = LinkKind::Web, .target = text, .reason = {}};
    }
    if (!scheme.isEmpty() && scheme != QLatin1String("file")) {
        // Esquema desconhecido nao e' "talvez": e' nao.
        return {.kind = LinkKind::Refused,
                .target = {},
                .reason = QStringLiteral("esquema não suportado no preview: %1").arg(scheme)};
    }

    const QString target = scheme == QLatin1String("file") ? url.toLocalFile() : text;
    // A ancora de um link para arquivo (`outro.md#secao`) nao faz parte do
    // caminho; quem abre o arquivo e' que a usa depois.
    const qsizetype hash = target.indexOf(QLatin1Char('#'));
    const QString withoutAnchor = hash >= 0 ? target.left(hash) : target;
    if (withoutAnchor.isEmpty()) {
        return {
            .kind = LinkKind::Refused, .target = {}, .reason = QStringLiteral("link sem arquivo")};
    }

    const QString absolute = resolveInsideRoot(withoutAnchor, documentPath, workspaceRoot);
    if (absolute.isEmpty()) {
        return {.kind = LinkKind::Refused,
                .target = {},
                .reason =
                    workspaceRoot.isEmpty()
                        ? QStringLiteral("sem projeto aberto, o preview não abre arquivo local")
                        : QStringLiteral("o caminho sai do projeto aberto")};
    }
    return {.kind = LinkKind::LocalFile, .target = absolute, .reason = {}};
}

ResourceDecision decideResource(const QString& href, const QString& documentPath,
                                const QString& workspaceRoot, bool remoteAllowed)
{
    const QString text = href.trimmed();
    if (text.isEmpty()) {
        return {.allowed = false, .path = {}, .reason = QStringLiteral("imagem sem caminho")};
    }

    const QUrl url(text);
    const QString scheme = url.scheme().toLower();
    if (isWebScheme(scheme)) {
        if (!remoteAllowed) {
            // §4.2: abrir documentacao nao busca recurso remoto. O alt text
            // aparece com o motivo, e o documento continua legivel.
            return {
                .allowed = false, .path = {}, .reason = QStringLiteral("imagem remota bloqueada")};
        }
        return {.allowed = true, .path = text, .reason = {}};
    }
    if (!scheme.isEmpty() && scheme != QLatin1String("file")) {
        return {.allowed = false,
                .path = {},
                .reason = QStringLiteral("esquema não suportado: %1").arg(scheme)};
    }

    const QString target = scheme == QLatin1String("file") ? url.toLocalFile() : text;
    const QString absolute = resolveInsideRoot(target, documentPath, workspaceRoot);
    if (absolute.isEmpty()) {
        return {.allowed = false,
                .path = {},
                .reason = workspaceRoot.isEmpty()
                              ? QStringLiteral("sem projeto aberto, imagens locais não são lidas")
                              : QStringLiteral("a imagem está fora do projeto aberto")};
    }
    return {.allowed = true, .path = absolute, .reason = {}};
}

} // namespace kinein::markdown
