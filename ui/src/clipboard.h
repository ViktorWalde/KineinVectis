// Ponte mínima para a área de transferência do sistema (fatia D2.2, DocsPublic/roadmaps/24).
//
// Utilitário de UI exposto ao QML como singleton `Clipboard`, para o terminal
// (e futuros consumidores) copiar/colar sem falar com o CoreClient — copiar/
// colar é UI, não IPC.

#pragma once

#include <QObject>
#include <QString>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class Clipboard : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON

public:
    explicit Clipboard(QObject* parent = nullptr);

    Q_INVOKABLE void setText(const QString& text);
    Q_INVOKABLE QString text() const;
};

} // namespace kinein
