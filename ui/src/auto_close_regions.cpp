#include "auto_close_regions.h"

#include <algorithm>

#include <QTextDocument>

namespace kinein {

namespace {

// Ninguem navega 64 pares para tras antes de fechar os primeiros; o limite so
// impede a lista de crescer para sempre num arquivo editado por horas.
constexpr int kMaxTracked = 64;

// Um registro vale enquanto seleciona EXATAMENTE um caractere. Apagar o
// fechador colapsa a selecao (largura 0) e o registro morre — inclusive apos
// undo, que recoloca o texto mas nao reabre a selecao colapsada. Morrer no
// undo e' deliberado: e' o mesmo que o Code OSS faz ao invalidar a regiao na
// edicao, e um registro ressuscitado poderia apontar para um caractere que ja
// nao e' o que foi inserido.
bool isAlive(const QTextCursor& closer)
{
    return !closer.isNull() && closer.hasSelection() &&
           closer.selectionEnd() - closer.selectionStart() == 1;
}

} // namespace

AutoCloseRegions::AutoCloseRegions(QObject* parent) : QObject(parent) {}

QQuickTextDocument* AutoCloseRegions::quickDocument() const
{
    return m_quickDocument;
}

void AutoCloseRegions::setQuickDocument(QQuickTextDocument* document)
{
    if (m_quickDocument == document) {
        return;
    }
    m_quickDocument = document;
    setDocument(document != nullptr ? document->textDocument() : nullptr);
    emit documentChanged();
}

void AutoCloseRegions::setDocument(QTextDocument* document)
{
    if (m_document == document) {
        return;
    }
    // Cursores pertencem ao documento antigo; carrega-los adiante seria
    // apontar para texto de outro arquivo.
    m_closers.clear();
    if (m_document != nullptr) {
        disconnect(m_document, nullptr, this, nullptr);
    }
    m_document = document;
    if (m_document != nullptr) {
        // Documento destruido antes deste objeto: cursores orfaos nao podem
        // sobreviver a ele.
        connect(m_document, &QObject::destroyed, this, [this]() {
            m_closers.clear();
            m_document = nullptr;
        });
    }
}

void AutoCloseRegions::notePairInserted(int closerPosition)
{
    if (m_document == nullptr || closerPosition < 0 ||
        closerPosition >= m_document->characterCount())
    {
        return;
    }
    QTextCursor closer(m_document);
    closer.setPosition(closerPosition);
    closer.setPosition(closerPosition + 1, QTextCursor::KeepAnchor);
    m_closers.append(closer);
    while (m_closers.size() > kMaxTracked) {
        m_closers.removeFirst();
    }
}

bool AutoCloseRegions::isAutoClosedAt(int position)
{
    prune();
    return std::ranges::any_of(m_closers, [position](const QTextCursor& closer) {
        return closer.selectionStart() == position;
    });
}

void AutoCloseRegions::consumeAt(int position)
{
    m_closers.removeIf(
        [position](const QTextCursor& closer) { return closer.selectionStart() == position; });
}

int AutoCloseRegions::trackedCount()
{
    prune();
    return static_cast<int>(m_closers.size());
}

void AutoCloseRegions::prune()
{
    m_closers.removeIf([](const QTextCursor& closer) { return !isAlive(closer); });
}

} // namespace kinein
