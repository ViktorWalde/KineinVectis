// A aba Web pede o contexto OpenGL compartilhado ANTES do QGuiApplication
// (16.6, DocsPublic/roadmaps/59 §7.1.6). Medido em 2026-10-16: ligado, o
// atributo custa ~13 ms no primeiro quadro e ~9 MiB de RSS com GL de verdade;
// por isso ele so' e' ligado quando uma opcao de visualizacao web esta' ligada.
// Aqui fica a leitura dessa opcao, sem janela, para o ctest prova-la.

#pragma once

#include <QString>

namespace kinein {

// O `settings.json` global que o core grava: `$XDG_CONFIG_HOME` (ou
// `~/.config`) no Unix, `%APPDATA%` no Windows, mais `kinein-vectis/`.
[[nodiscard]] QString globalSettingsFile();

// Alguma visualizacao web pedida: `grafanaWebView` ou `platformioWebView`
// (52 §14.45), cada uma com a do projeto (quando ha' uma pasta na linha de
// comando) vencendo a global; arquivo ausente ou invalido conta como desligada.
[[nodiscard]] bool wantsWebView(const QString& globalFile, const QString& workspaceFile);

} // namespace kinein
