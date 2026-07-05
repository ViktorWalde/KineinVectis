# 04 — Sistema de Comandos

## Ideia central

Tudo na IDE deve ser um comando.

Menus, atalhos, command palette, botões, ações de IA e plugins devem chamar o mesmo sistema de comandos.

## Exemplos de comandos

```text
core.ping
workspace.open
workspace.createFolder
workspace.createProject
workspace.reload
project.create
project.index
editor.openFile
editor.saveFile
editor.formatFile
editor.quickFix
fs.findFiles
symbol.rename
build.configure
build.run
build.clean
lsp.didChange
lsp.definition
lsp.hover
lsp.restart
debug.start
debug.stop
git.status
git.commit
git.stageFile
ai.explainSelection
ai.reviewDiff
tools.detect
settings.open
```

## Estrutura de comando

```json
{
  "id": "lsp.definition",
  "title": "Go to Definition",
  "category": "LSP",
  "description": "Resolve a definicao do simbolo na posicao atual do editor",
  "defaultShortcut": "Ctrl+B",
  "requiresWorkspace": true
}
```

## Benefícios

- Menus ficam simples.
- Atalhos ficam consistentes.
- Command palette fica poderosa.
- Plugins podem registrar comandos.
- IA pode sugerir comandos com segurança.
- Logs podem registrar ações de forma rastreável.

## Command Palette

Atalhos-alvo:

```text
Shift Shift       Search Everywhere
Ctrl+Shift+A     Find Action
Ctrl+Shift+P     Command Palette alternativa
```

Estado MVP atual:

```text
command.list expõe o registro de comandos do core;
Search Everywhere consome command.list e mistura comandos com arquivos;
a execução ainda usa mapeamentos locais existentes na UI, sem command.execute genérico.
```

## Presets de atalhos

Começar com:

```text
JetBrains Compatible
```

Futuro:

```text
Kernwerk Default
VS Code Compatible
Vim Mode
```

## Regras

1. Não criar botão que não chame comando.
2. Não criar atalho direto para função interna.
3. Comandos devem validar contexto.
4. Comandos devem retornar resultado estruturado.
5. Comandos longos devem emitir eventos de progresso.
