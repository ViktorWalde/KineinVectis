// Flags de diagnostico da UI, ligadas por variavel de ambiente (R0,
// docs/roadmaps/26).
//
// Existe porque investigar geometria de terminal exige VER a grade, e a unica
// forma honesta de fazer isso e desenhar por cima do render real — nao numa
// maquete. Sem a env, todas as flags sao false, o Loader do overlay nunca
// instancia e o uso normal nao paga nada.
//
// Isto NAO e configuracao: nao entra em settings, nao tem schema e nao aparece
// na UI. E instrumentacao temporaria de diagnostico, ligada por quem esta
// depurando. Nao loga texto do terminal, prompt, clipboard nem bytes da sessao
// — so metricas e estado do cursor.

#pragma once

#include <QObject>
#include <QtQml/qqmlregistration.h>

namespace kinein {

class DebugFlags : public QObject
{
    Q_OBJECT
    QML_ELEMENT
    QML_SINGLETON
    // KINEIN_TERMINAL_DEBUG_GEOMETRY=1 — overlay de celula/baseline/cursor.
    Q_PROPERTY(bool terminalGeometry READ terminalGeometry CONSTANT)

public:
    explicit DebugFlags(QObject* parent = nullptr);

    [[nodiscard]] static bool terminalGeometry();
};

} // namespace kinein
