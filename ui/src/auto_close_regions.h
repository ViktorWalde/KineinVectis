// Rastreio de fechadores AUTO-INSERIDOS pelo auto-close de pares.
//
// POR QUE EXISTE (E6, 2026-07-17). O type-over correto so pula o fechador que a
// PROPRIA IDE inseriu; pular qualquer fechador no cursor engole o caractere que
// o usuario quis digitar (em `foo(bar)` escrito a mao, digitar `)` antes do `)`
// sumia com ele — divergencia medida contra o Code OSS, registrado em
// docsprivate/diario/18). A parte sutil e' que um int de posicao nao sobrevive a uma
// edicao: quem acompanha o documento e' o QTextCursor, que o Qt reposiciona
// sozinho em digitacao, colagem, remocao e undo. Este objeto guarda um cursor
// por fechador e valida por preguica: regiao cujo texto sumiu morre sozinha.

#pragma once

#include <QList>
#include <QObject>
#include <QQuickTextDocument>
#include <QTextCursor>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class AutoCloseRegions : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    Q_PROPERTY(QQuickTextDocument* document READ quickDocument WRITE setQuickDocument NOTIFY
                   documentChanged)

public:
    explicit AutoCloseRegions(QObject* parent = nullptr);

    [[nodiscard]] QQuickTextDocument* quickDocument() const;
    void setQuickDocument(QQuickTextDocument* document);

    // O rastreio funciona sobre QTextDocument puro; o QQuickTextDocument acima
    // e' so a casca de fiacao QML. E' esta porta que o teste C++ usa.
    void setDocument(QTextDocument* document);

    // Chamado pelo EditorAutoClosePairs logo apos inserir um par:
    // `closerPosition` e' a posicao do FECHADOR recem-inserido.
    Q_INVOKABLE void notePairInserted(int closerPosition);

    // true se ha um fechador auto-inserido exatamente em `position`.
    Q_INVOKABLE bool isAutoClosedAt(int position);

    // Remove o registro em `position` (apos o type-over consumi-lo).
    Q_INVOKABLE void consumeAt(int position);

    [[nodiscard]] Q_INVOKABLE int trackedCount();

signals:
    void documentChanged();

private:
    void prune();

    QQuickTextDocument* m_quickDocument = nullptr;
    QTextDocument* m_document = nullptr;
    QList<QTextCursor> m_closers;
};

} // namespace kinein
