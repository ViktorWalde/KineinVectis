// 0.3.9 (roadmap 53 §13.1): o TEMPO DE QUADRO das animacoes e da interacao.
//
// O autor pediu fluidez de 60 Hz, 120 Hz onde der (2026-10-01); o criterio e'
// quadro <= 16,7 ms (e <= 8,3 ms onde a tela permitir). Sem medir, "fluido" e'
// opiniao. Ligado so' por KINEIN_PERF_FRAMES=<segundos>: registra o intervalo
// entre quadros apresentados e, no fim, imprime mediana, p95, pior caso e
// quantos passaram de 16,7 ms e de 8,3 ms. O Qt so' desenha quando algo muda:
// intervalos acima de 100 ms sao pausa ociosa, nao quadro lento, e ficam fora.
// Sem a env, zero efeito. Nao e' configuracao nem telemetria: so' stderr.

#pragma once

class QGuiApplication;
class QQmlApplicationEngine;

namespace kinein {

void installFramePacingProbe(QGuiApplication& app, QQmlApplicationEngine& engine);

} // namespace kinein
