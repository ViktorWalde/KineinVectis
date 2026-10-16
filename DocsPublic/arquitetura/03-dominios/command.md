# `command.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `command.*` — o catálogo de comandos que a UI mostra

Um método, e ele é a fonte única de **tudo que a IDE oferece por nome**: paleta,
menus, botões e atalhos leem daqui.

```text
command.list {}   ->  { commands: [CommandDescriptor] }
```

```json
{
  "id": "editor.save",
  "title": "Salvar arquivo",
  "category": "Editor",
  "description": "Grava o buffer atual no disco",
  "defaultShortcut": "Ctrl+S",
  "requiresWorkspace": true
}
```

Um descritor de encanamento leva também `"internal": true` (desde a
0.148.0). A paleta não o lista, mas os atalhos e os menus continuam a lê-lo.

**Não exige workspace aberto** — a paleta existe antes de haver projeto, e é o
`requiresWorkspace` de cada descritor que diz o que fica desabilitado.

**São 76 descritores, medidos em 2026-09-06**, em cinco grupos que são cinco
arquivos em `crates/kinein-core/src/commands/`:

```text
ide.rs      24    editor.rs   19    build.rs    16    git.rs       9    run.rs    8
```

**Por que este domínio existe em vez de a UI ter a lista:** porque o atalho que a
paleta **anuncia** tem de ser o que a IDE **obedece**, e isso é gate desde
2026-09-03 (`scripts/verificar-atalhos.sh`). Com a lista no core, o gate compara
uma fonte com o host; com a lista na UI, ele compararia a UI consigo mesma.
