#include "debug_flags.h"

namespace kinein {

DebugFlags::DebugFlags(QObject* parent) : QObject(parent) {}

bool DebugFlags::terminalGeometry()
{
    // Lida a cada chamada, mas a propriedade e CONSTANT: o Qt consulta uma vez
    // por binding. Nao ha estado para sincronizar nem custo em regime.
    return qEnvironmentVariableIsSet("KINEIN_TERMINAL_DEBUG_GEOMETRY");
}

} // namespace kinein
