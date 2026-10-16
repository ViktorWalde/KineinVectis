// O que a aba Web precisa saber desta abertura (16.6, DocsPublic/roadmaps/59
// §7.1.6): se o contexto OpenGL compartilhado foi ligado antes do
// QGuiApplication. Sem ele o QtWebEngine nao desenha; ligada a opcao com a IDE
// aberta, a aba diz que vale na proxima abertura.

#pragma once

#include <QObject>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class WebSupport : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    Q_PROPERTY(bool contextShared READ contextShared CONSTANT)

public:
    explicit WebSupport(QObject* parent = nullptr);

    [[nodiscard]] static bool contextShared();
};

} // namespace kinein
