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
// (52 §14.45) no `settings.json` global, o unico que o core usa para elas
// (INF-14). O arquivo e' lido com as regras do core (`settings/file.rs`):
// segue o link, tem de ser arquivo regular de ate' 1 MiB, JSON em UTF-8, um
// objeto com `schemaVersion` 1; o que nao passa conta como desligada. Duas
// diferencas, que custam so' o contexto a mais: o valor de tipo errado em
// outra chave nao e' conferido aqui, e o `schemaVersion` 1.0, que o core
// recusa, passa, porque o Qt o guarda como o inteiro 1.
[[nodiscard]] bool wantsWebView(const QString& globalFile);

} // namespace kinein
