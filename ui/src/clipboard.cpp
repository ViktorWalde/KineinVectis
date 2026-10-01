#include "clipboard.h"

#include <QClipboard>
#include <QFileInfo>
#include <QGuiApplication>
#include <QMimeData>
#include <QUrl>

namespace kinein {

Clipboard::Clipboard(QObject* parent) : QObject(parent)
{
    if (QClipboard* clip = QGuiApplication::clipboard()) {
        connect(clip, &QClipboard::dataChanged, this, &Clipboard::filesChanged);
    }
}

// Métodos de instância (não static) porque o QML os invoca como `Q_INVOKABLE`
// no singleton; o clang-tidy sugere static por não usarem `this`, o que
// quebraria a invocação do QML — suprimido de propósito.

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
void Clipboard::setText(const QString& text)
{
    if (QClipboard* clip = QGuiApplication::clipboard()) {
        clip->setText(text);
    }
}

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
QString Clipboard::text() const
{
    const QClipboard* clip = QGuiApplication::clipboard();
    return clip != nullptr ? clip->text() : QString();
}

// QML calls this instance method; QClipboard takes ownership of the MIME data.
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
void Clipboard::setFiles(const QStringList& paths, bool cut)
{
    QClipboard* clip = QGuiApplication::clipboard();
    if (clip == nullptr || paths.isEmpty()) {
        return;
    }
    // NOLINTNEXTLINE(cppcoreguidelines-owning-memory)
    auto* data = new QMimeData();
    QList<QUrl> urls;
    QByteArray gnome = cut ? QByteArrayLiteral("cut") : QByteArrayLiteral("copy");
    for (const QString& path : paths) {
        const QUrl url = QUrl::fromLocalFile(path);
        urls.append(url);
        gnome.append('\n');
        gnome.append(url.toEncoded());
    }
    data->setUrls(urls);
    data->setData(QStringLiteral("x-special/gnome-copied-files"), gnome);
    if (cut) {
        data->setData(QStringLiteral("application/x-kinein-cut"), QByteArrayLiteral("1"));
    }
    clip->setMimeData(data);
}

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
QStringList Clipboard::filePaths() const
{
    QStringList paths;
    const QClipboard* clip = QGuiApplication::clipboard();
    const QMimeData* data = clip != nullptr ? clip->mimeData() : nullptr;
    if (data == nullptr) {
        return paths;
    }
    for (const QUrl& url : data->urls()) {
        if (!url.isLocalFile()) {
            return {};
        }
        paths.append(url.toLocalFile());
    }
    return paths;
}

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
QStringList Clipboard::localFilePathsFromUrls(const QVariantList& urls) const
{
    if (urls.isEmpty() || urls.size() > 128) {
        return {};
    }
    QStringList paths;
    for (const QVariant& value : urls) {
        if (!value.canConvert<QUrl>()) {
            return {};
        }
        const QUrl url = value.toUrl();
        if (!url.isValid() || !url.isLocalFile() || !url.host().isEmpty() || url.hasQuery() ||
            url.hasFragment())
        {
            return {};
        }
        const QString path = url.toLocalFile();
        if (!path.startsWith(QLatin1Char('/'))) {
            return {};
        }
        paths.append(path);
    }
    return paths;
}

// Reusa a validação de URLs acima; o acolhimento só anuncia drop de pasta.
// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
QString Clipboard::localDirectoryPathFromUrls(const QVariantList& urls) const
{
    const QStringList paths = localFilePathsFromUrls(urls);
    if (paths.size() != 1 || !QFileInfo(paths.front()).isDir()) {
        return {};
    }
    return paths.front();
}

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
QUrl Clipboard::localFileUrl(const QString& path) const
{
    return path.startsWith(QLatin1Char('/')) ? QUrl::fromLocalFile(path) : QUrl{};
}

bool Clipboard::hasFiles() const
{
    return !filePaths().isEmpty();
}

// NOLINTNEXTLINE(readability-convert-member-functions-to-static)
bool Clipboard::filesCut() const
{
    const QClipboard* clip = QGuiApplication::clipboard();
    const QMimeData* data = clip != nullptr ? clip->mimeData() : nullptr;
    return data != nullptr &&
           (data->data(QStringLiteral("application/x-kinein-cut")) == QByteArrayLiteral("1") ||
            data->data(QStringLiteral("x-special/gnome-copied-files"))
                .startsWith(QByteArrayLiteral("cut\n")));
}

void Clipboard::clearCutFileIfMatches(const QString& path)
{
    removeCutFilesIfMatches(QStringList{path}, QStringList{path});
}

void Clipboard::removeCutFilesIfMatches(const QStringList& expectedPaths,
                                        const QStringList& movedPaths)
{
    QClipboard* clip = QGuiApplication::clipboard();
    if (clip == nullptr || expectedPaths.isEmpty() || !filesCut() || filePaths() != expectedPaths) {
        return;
    }
    QStringList remaining;
    for (const QString& path : expectedPaths) {
        if (!movedPaths.contains(path)) {
            remaining.append(path);
        }
    }
    if (remaining.isEmpty()) {
        clip->clear();
    }
    else {
        setFiles(remaining, true);
    }
}

} // namespace kinein
