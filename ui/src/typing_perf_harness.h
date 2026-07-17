// A3.3 item 1: harness de digitacao tecla->frame do editor real.
//
// Ligado so por KINEIN_PERF_TYPING. Sem a env, zero efeito no uso normal —
// mesma disciplina do KINEIN_PERF_MARKER (main.cpp) e do
// KINEIN_TERMINAL_DEBUG_GEOMETRY (debug_flags.h).
//
// Nao e configuracao: nao entra em settings, nao tem schema e nao aparece na
// UI. E instrumentacao de medicao, ligada por quem esta medindo. Nao loga
// texto do documento — so tempos.

#pragma once

class QGuiApplication;
class QQmlApplicationEngine;

namespace kinein {

// Instala o harness se KINEIN_PERF_TYPING estiver setado; caso contrario
// retorna sem tocar em nada.
void installTypingPerfHarness(QGuiApplication& app, QQmlApplicationEngine& engine);

} // namespace kinein
