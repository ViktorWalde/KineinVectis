#pragma once

// O LOG DE DIAGNOSTICO e a REDE DE SEGURANCA das mensagens do Qt (roadmap 53
// §0.1 e §5.1, 2026-10-01).
//
// Principio do autor: mensagem interna do Qt nao se esconde, ela NAO deve
// existir — cada uma e' rastreada ate' a causa e corrigida, e o gate
// (`scripts/avisos-qml.txt`) reprova a que aparecer. Isto aqui NAO e' a
// correcao: e' o que guarda, para relatorio de defeito, o que ainda escapar.
// Por isso grava SO' no arquivo de diagnostico — nunca na interface (a aba IDE
// e' do usuario) — e repassa a mensagem ao comportamento anterior, para o gate
// e o `--verbose` continuarem vendo o stderr.

#include <QString>

namespace kinein {

/// `<cache>/kinein-vectis/logs/kinein-ui-erros.txt` — o mesmo arquivo que o
/// `CoreClient::errorLogFile()` sempre usou; agora com um dono so'.
[[nodiscard]] QString diagnosticLogPath();

/// Instala o handler. Chamar uma vez, antes do QGuiApplication.
void installQtMessageLog();

} // namespace kinein
