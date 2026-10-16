# `format.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Formatação de buffer (`format.text` / `format.capabilities`)

Implementado no protocolo `0.21.0` (fatia M1.1). Requer workspace aberto. Formata o conteúdo
do editor com a ferramenta do projeto via stdin/stdout, sem tocar o disco —
salvar continua sendo decisão do usuário. O formatter roda com cwd na raiz do
workspace, então `rustfmt.toml`/`.clang-format` do projeto valem.

- `format.text { path, text }` →
  `{ path, text, changed, formatter }`.
- `path` precisa existir e estar confinado ao workspace (`INVALID_PARAMS` se
  escapar); é ecoado canonicalizado na resposta para a UI descartar respostas
  de uma aba que já mudou.
- Formatter por extensão: `.rs` → `rustfmt` (`--emit stdout`; acrescenta
  `--edition 2021` apenas quando o root não tem `rustfmt.toml`/
  `.rustfmt.toml`); `.c/.cc/.cpp/.cxx/.h/.hh/.hpp/.hxx` → `clang-format`
  (`--assume-filename=<path> --style=file --fallback-style=LLVM`);
  `.py/.pyi` → `ruff` (`0.99.0`, 2026-09-13: `ruff format --stdin-filename
  <path>` — o nome do arquivo decide qual `ruff.toml`/`pyproject [tool.ruff]`
  vale; com `--stdin-filename` e sem caminhos o ruff lê stdin, medido no
  0.16.4). Extensão sem formatter → `INVALID_PARAMS`.
- `changed: false` quando a saída é idêntica ao texto enviado.
- **O binário é o DETECTADO** (`0.99.0`): o handler pergunta ao detector de
  ferramentas — que procura no `PATH` e em `~/.local/bin`, onde pipx/uv põem o
  ruff e o `PATH` do processo da IDE pode não alcançar — e só cai no nome nu
  quando o detector não acha. Binário ausente → `TOOL_NOT_FOUND` (com
  `data.tool`); formatter com exit ≠ 0 → `INTERNAL_ERROR` com o stderr na
  mensagem.
- `format.capabilities {}` → `{ formatters: [ { id, extensions[] } ] }`
  (`0.61.0`). **Não** requer workspace: é o mapa estático de extensões, derivado
  da mesma constante que `formatter_for_path` usa para decidir — o que a UI
  recebe é, por construção, o que o `format.text` vai aceitar.

  **Invariante de camada.** Quem decide o que é formatável é o core; a UI
  consome. Até `0.60.0` o `EditorController.qml` mantinha duas listas escritas à
  mão (`formattableLanguage` por linguagem, `formattablePath` por extensão) que
  não concordavam entre si nem com o core, e adicionar linguagem exigia editar
  QML. Duas fontes para a mesma verdade divergem por construção.

  Capacidade e disponibilidade são perguntas **distintas**: `format.capabilities`
  responde "existe formatter registrado para esta extensão"; se o binário está no
  `PATH` só se descobre ao rodar `format.text` (`TOOL_NOT_FOUND`). O catálogo não
  muda com o workspace, então a UI pode pedi-lo uma vez por conexão.
- Operação síncrona por ser curta (um buffer); "formatar workspace inteiro"
  viraria job, e fica fora deste contrato.
