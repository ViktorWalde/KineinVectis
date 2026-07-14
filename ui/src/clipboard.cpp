#include "clipboard.h"

#include <QClipboard>
#include <QGuiApplication>

namespace kinein {

Clipboard::Clipboard(QObject* parent) : QObject(parent) {}

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

} // namespace kinein
