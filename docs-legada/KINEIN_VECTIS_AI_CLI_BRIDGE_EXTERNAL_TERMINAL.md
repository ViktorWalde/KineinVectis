# Kinein Vectis — Parte 7.1: AI CLI Bridge e Terminal IA Externo

> ## ⛔ FORA DE ESCOPO — decisão do autor, 2026-07-17
>
> **A linha inteira de assistência com IA na IDE foi cancelada.** Não há painel
> de IA, não há chat, não há seletor de agente, não há aba dedicada e não há
> atalho no rail. Nada disto está implementado e nada será implementado a partir
> deste documento.
>
> **O motivo, e ele é de produto:**
>
> ```text
> o usuario roda claude/codex/qualquer agente no TERMINAL, naturalmente.
> a IDE ja tem terminal. o atalho visual so poupava digitar uma palavra —
> e cobrava por isso um seletor, um rotulo, uma numeracao, uma aba e um
> icone. nao se paga.
> ```
>
> Foi implementado em 2026-07-17 (seletor Claude/Codex + aba própria + ícone no
> rail) e **removido no mesmo dia**, depois de rodar. A decisão veio de usar, não
> de teorizar: `git log` entre `4782d82` e a remoção tem a fatia inteira.
>
> **O que sobreviveu, e por quê:** o core detecta `claude` e `codex` no
> `KNOWN_TOOLS`, como detecta `cargo` ou `clangd` — mesmo probe, sem nenhum ramo
> por programa, coberto por `ai_clis_are_detected_exactly_like_any_other_tool`.
> Isso não é "assistente": é o painel Ferramentas dizendo se o binário está no
> PATH, e é *mais* útil para quem vai usar o terminal direto.
>
> **O que este documento ainda vale:** registro do raciocínio. Contexto
> determinístico, preview antes de aplicar, evidência e sanitização continuam
> boas ideias se um dia a linha for reaberta. Não implementar nada daqui sem
> reabrir a decisão acima, explicitamente.


> ## Estado em 2026-07-16 — o bridge foi REMOVIDO (protocolo `0.59.0`)
>
> **O domínio `aiBridge.*` não existe mais**, e com ele saíram a superfície do
> Assistente, os perfis allowlisted e as settings `aiCliProfile` /
> `aiCliFlatTranscript`. Uma CLI de IA passa a ser usada como em qualquer IDE
> profissional: abrir o terminal integrado e rodar `claude` ou `codex`.
>
> **Motivo — o bridge era a interferência.** Ele tratava um agente como um
> programa especial: injetava argumentos pelo core (`--no-alt-screen`,
> `--ax-screen-reader`) e filtrava `CSI 3 J` emitido pela própria aplicação.
> Isso é a IDE se metendo entre o programa e o terminal — exatamente o que
> nenhuma IDE profissional faz e o que fazia a ferramenta se comportar de um
> jeito dentro da Kinein e de outro fora dela. O sintoma aparecia atribuído ao
> painel; a causa era o caminho especial somado a um emulador raso (ver
> [ADR-0004](../docs/adr/ADR-0004-alacritty-terminal-emulator.md)).
>
> **O modelo correto, agora implementado:** uma CLI de IA é **um programa como
> outro qualquer**. Mesmo `terminal.open`, mesmo PTY, mesmo emulador, mesmo
> contrato `terminal.input/resize/scroll/close` de um `ls` ou de um `vim`. Sem
> allowlist, sem argumento imposto, sem filtro, sem preferência persistida.
>
> **Retorno do Assistente.** Pode voltar como **UI pura** — um atalho visual
> que abre uma sessão de terminal comum para desacoplar visualmente do uso
> padrão do terminal, sem regra de negócio própria no core. A UI representa o
> backend; ela não o define. Volta quando o terminal estiver consolidado, com a
> integração do Zed como referência.
>
> **O que continua valendo neste documento:** o modelo de IA (externa, por CLI
> do usuário, sem chat embutido e sem chamada de rede pela IDE) permanece a
> fonte de verdade. O que caiu foi o **mecanismo**, não o princípio.

> **Tipo:** correção arquitetural da Parte 7.  
> **Status:** decisão de produto e arquitetura; superfície do painel suspensa
> (ver bloco acima).  
> **Decisão principal:** a Kinein Vectis **não terá IA embutida como chat interno da IDE**.  
> **Modelo correto:** a IDE continua determinística; a IA é uma ferramenta externa acionada por atalho, abrindo um terminal separado configurado pelo usuário.

---

## 1. Decisão final

A Kinein Vectis deve seguir este modelo:

```text
Kinein Vectis = IDE determinística, auditável e local-first.
IA = ferramenta externa opcional, chamada por atalho ou botão.
```

A IDE não deve ter:

```text
- chat de IA embutido;
- painel lateral de conversa com IA;
- provider de IA obrigatório;
- API key obrigatória;
- respostas automáticas dentro do fluxo principal;
- sugestões invasivas aparecendo sem pedido;
- agente editando arquivos por conta própria.
```

A IDE pode ter:

```text
- um terminal especial para IA CLI;
- perfis configuráveis de IA CLI;
- um atalho para abrir a IA externa;
- geração segura de contexto em markdown;
- cópia de contexto para clipboard;
- envio opcional de contexto para o terminal IA;
- separação visual entre terminal comum e terminal IA.
```

---

## 2. Por que esta correção melhora o projeto

A decisão de remover IA embutida melhora a Kinein em vários pontos.

### 2.1 Menos complexidade no core

A IDE não precisa gerenciar:

```text
- provedores de IA;
- tokens de API;
- billing;
- streaming de resposta;
- histórico de chat;
- memória;
- autenticação;
- fallback entre modelos;
- prompts complexos dentro da UI;
- risco de privacidade por design.
```

Isso reduz escopo e aumenta a chance de um MVP sólido.

---

### 2.2 UX menos invasiva

A IA deixa de competir com o editor.

O fluxo vira:

```text
Usuário quer ajuda
  ↓
aciona atalho
  ↓
terminal IA abre
  ↓
usuário usa a ferramenta externa
```

A IDE não fica “opinando” o tempo todo.

---

### 2.3 Mais compatível com o jeito real de muitos desenvolvedores

Muitos programadores já usam IA CLI separada:

```text
Claude CLI
Codex CLI
Aider
OpenCode
Gemini CLI
Continue CLI
ferramenta própria
script local
```

A Kinein não precisa escolher uma.  
Ela só precisa abrir a ferramenta que o usuário já usa.

---

### 2.4 Preserva privacidade e controle

A IDE pode gerar contexto, mas o usuário decide:

```text
- se quer passar contexto;
- qual contexto passar;
- para qual ferramenta;
- se vai copiar, anexar ou abrir;
- se vai rodar comando;
- se vai enviar para nuvem ou local.
```

Isso evita a sensação de IDE espionando ou “mandando código para IA”.

---

## 3. Novo conceito: AI CLI Bridge

O nome conceitual desta camada pode ser:

```text
AI CLI Bridge
```

Ou, em linguagem visual da Kinein:

```text
KV AI Bridge
KV Terminal Bridge
AI Terminal Bridge
```

A função dela é simples:

```text
criar ponte entre contexto determinístico da IDE e uma IA CLI externa.
```

Ela não interpreta resposta da IA.  
Ela não injeta resposta no editor.  
Ela não aplica patch automaticamente.  
Ela só prepara e abre o caminho.

---

## 4. Terminal comum vs Terminal IA

A Kinein deve diferenciar claramente dois tipos de terminal.

### 4.1 Terminal comum

Uso:

```text
shell normal
git
cmake
cargo
ninja
scripts
build manual
debug manual
comandos do usuário
```

Visual:

```text
nome: Terminal
ícone: terminal padrão
acento: neutro
contexto: shell comum
```

---

### 4.2 Terminal IA

Uso:

```text
Claude CLI
Codex CLI
Aider
OpenCode
outra IA CLI
script configurado pelo usuário
```

Visual:

```text
nome: AI Terminal
ícone: terminal + marcador KV/AI discreto
acento: âmbar ou roxo discreto
badge: external tool
aviso: command configured by user
```

O usuário deve saber que está em um terminal especial.

### 4.3 Fidelidade de terminal

Depois que a CLI é iniciada, a superfície deve ser **terminal-first** e usar o
mesmo renderer/PTY do Terminal comum:

```text
- grid VT e cursor reais;
- cores/atributos xterm-256color;
- teclado caractere a caractere e modos application-cursor;
- bracketed paste quando solicitado pela aplicação;
- seleção, copiar/colar e clique do meio;
- roda, barra arrastável e scrollback visível;
- resize coalescido e fluido.
```

O seletor de perfis existe apenas antes da sessão. Cards, explicações ou UI de
chat não devem dividir o espaço com o prompt depois que Claude/Codex abriu.

Para Codex, a integração pode usar a opção oficial `--no-alt-screen` como
argumento fixo do perfil. Ela preserva o TUI real em modo inline e torna a
conversa navegável pelo scrollback da IDE; não é parser, wrapper visual nem
comando livre criado pela UI.

Se a CLI emitir `CSI 3 J` mesmo no modo inline, o bridge pode suprimir somente
essa sequência para cumprir a promessa de transcript navegável. A política é
exclusiva das sessões de IA: o Terminal comum continua honrando comandos de
limpeza integralmente, e nenhum outro escape ou conteúdo da TUI é alterado.

---

## 5. Fluxo principal

### 5.1 Abrir IA sem contexto

```text
Usuário pressiona atalho
  ↓
Kinein abre AI Terminal
  ↓
executa comando configurado
```

Exemplo:

```bash
claude
```

Ou:

```bash
codex
```

Ou:

```bash
aider
```

---

### 5.2 Abrir IA com contexto do arquivo atual

```text
Usuário está em src/main.cpp
  ↓
pressiona "Open current file in AI Terminal"
  ↓
Kinein cria arquivo temporário de contexto
  ↓
abre AI Terminal
  ↓
executa comando configurado com o contexto
```

Exemplo genérico:

```bash
claude < /tmp/kinein/context/current-file.md
```

Ou:

```bash
codex --context /tmp/kinein/context/current-file.md
```

---

### 5.3 Abrir IA com erro de build

```text
Build falhou
  ↓
usuário clica "Open in AI Terminal"
  ↓
Kinein coleta:
    - comando de build
    - trecho relevante do log
    - CMakePresets/CMakeLists se permitido
    - target ativo
    - compiler ativo
  ↓
sanitiza
  ↓
gera context.md
  ↓
abre AI Terminal
```

---

### 5.4 Abrir IA com seleção de código

```text
Usuário seleciona trecho
  ↓
atalho "Send selection to AI Terminal"
  ↓
Kinein cria context.md com:
    - arquivo
    - linguagem
    - trecho selecionado
    - linha inicial/final
  ↓
abre terminal IA
```

---

## 6. O que a IDE pode enviar como contexto

A IDE deve gerar contexto em markdown simples, legível e auditável.

### 6.1 Contexto mínimo

```markdown
# Kinein Context

## Workspace
Name: motor-control
Path: /home/vitor/Projects/motor-control

## Active File
Path: src/main.cpp
Language: C++

## Selection
Lines: 12-28

```cpp
// trecho selecionado
```
```

---

### 6.2 Contexto de erro de build

```markdown
# Kinein Build Failure Context

## Build Command
cmake --build --preset debug

## Toolchain
Compiler: clang++ 18
Generator: Ninja
Preset: debug
Target: local-linux

## Main Error
undefined reference to `MotorDriver::init()`

## Relevant Log
```text
/usr/bin/ld: main.cpp.o: undefined reference to `MotorDriver::init()`
collect2: error: ld returned 1 exit status
```

## Relevant Files
- CMakeLists.txt
- src/main.cpp
- include/motor_driver.hpp
```

---

### 6.3 Contexto de toolchain

```markdown
# Kinein Toolchain Context

## Active Toolchain
Name: Clang Local
C Compiler: /usr/bin/clang
C++ Compiler: /usr/bin/clang++
Debugger: /usr/bin/lldb
CMake: /usr/bin/cmake
Ninja: /usr/bin/ninja

## Health Check
clang++: OK
cmake: OK
ninja: OK
clangd: OK
lldb: Missing
```

---

## 7. O que a IDE nunca deve enviar automaticamente

A Kinein deve bloquear por padrão:

```text
.env
.env.*
*.pem
*.key
id_rsa
id_ed25519
*.p12
*.pfx
credentials.*
secrets.*
token.*
.ssh/
.aws/
.git/
```

Também deve mascarar padrões:

```text
API_KEY=
TOKEN=
SECRET=
PASSWORD=
PRIVATE_KEY
BEGIN OPENSSH PRIVATE KEY
BEGIN RSA PRIVATE KEY
```

Regra:

```text
Na dúvida, não incluir.
```

---

## 8. Context Preview obrigatório

Antes de abrir o terminal IA com contexto, a IDE deve oferecer preview, especialmente no primeiro uso.

### 8.1 Modos

```text
Always preview
Preview first time only
Never preview for low-risk context
```

Padrão recomendado:

```text
Always preview para contexto com arquivos.
Preview first time only para seleção curta.
```

### 8.2 Preview deve mostrar

```text
- arquivos incluídos;
- linhas incluídas;
- logs incluídos;
- secrets removidos;
- comando que será executado;
- pasta de trabalho;
- perfil de IA usado.
```

---

## 9. Perfis de AI CLI

A Kinein deve permitir perfis configuráveis.

### 9.1 Exemplo de perfil

```json
{
  "id": "claude-default",
  "name": "Claude CLI",
  "command": "claude",
  "args": [],
  "context_mode": "stdin",
  "working_directory": "${workspaceRoot}",
  "terminal_kind": "ai",
  "enabled": true
}
```

### 9.2 Perfil com arquivo de contexto

```json
{
  "id": "codex-context-file",
  "name": "Codex CLI",
  "command": "codex",
  "args": ["--context", "${contextFile}"],
  "context_mode": "file_arg",
  "working_directory": "${workspaceRoot}",
  "terminal_kind": "ai",
  "enabled": true
}
```

### 9.3 Perfil com Aider

```json
{
  "id": "aider-current-file",
  "name": "Aider",
  "command": "aider",
  "args": ["${activeFile}"],
  "context_mode": "active_file_arg",
  "working_directory": "${workspaceRoot}",
  "terminal_kind": "ai",
  "enabled": true
}
```

---

## 10. Variáveis de template

A Kinein pode suportar variáveis:

```text
${workspaceRoot}
${activeFile}
${selectedTextFile}
${contextFile}
${buildLogFile}
${diagnosticFile}
${projectName}
${targetName}
${buildPreset}
${toolchainName}
```

Essas variáveis são substituídas antes de abrir o terminal.

---

## 11. Modos de passagem de contexto

### 11.1 Nenhum contexto

```text
context_mode: none
```

Só abre o CLI.

---

### 11.2 stdin

```text
context_mode: stdin
```

A IDE envia o markdown para entrada padrão do processo.

Vantagem:

```text
não deixa arquivo temporário persistente.
```

Desvantagem:

```text
nem toda IA CLI trabalha bem com stdin.
```

---

### 11.3 arquivo temporário

```text
context_mode: file_arg
```

A IDE cria um arquivo:

```text
/tmp/kinein/context/ctx_2026_07_04_181200.md
```

E passa como argumento.

Vantagem:

```text
compatível com mais ferramentas.
```

Desvantagem:

```text
precisa limpar depois.
```

---

### 11.4 clipboard

```text
context_mode: clipboard
```

A IDE copia o contexto para clipboard e abre o terminal.

Vantagem:

```text
usuário controla manualmente o paste.
```

Desvantagem:

```text
menos automatizado.
```

---

## 12. Comandos e atalhos

### 12.1 Command Palette

Comandos sugeridos:

```text
AI Terminal: Open
AI Terminal: Open with Current File
AI Terminal: Open with Selection
AI Terminal: Open with Build Error
AI Terminal: Open with Diagnostic
AI Terminal: Copy Context
AI Terminal: Preview Context
AI Terminal: Configure Profiles
```

### 12.2 Atalhos sugeridos

```text
Ctrl+Alt+A      Open AI Terminal
Ctrl+Alt+E      Open Build Error in AI Terminal
Ctrl+Alt+S      Send Selection to AI Terminal
```

No Linux, validar conflitos antes.

---

## 13. UI: Terminal IA

### 13.1 Bottom Tool Window

Abas:

```text
Terminal
AI Terminal
Build
CMake
Debug
Serial
Problems
```

A aba AI Terminal deve ser distinta, mas discreta.

Na implementação do shell atual, essa superfície pode ocupar o painel direito
**Assistente**, desde que continue separada do Terminal comum e obedeça às
mesmas capacidades de terminal. A largura 300–480px vale para o seletor e para
o estado compacto. Com uma CLI ativa:

```text
- largura inicial recomendada: cerca de 50% da área;
- splitter livre e largura persistida da sessão entre 300 e 720px, separada
  dos 300–480px do seletor compacto;
- preservar uma faixa útil do editor quando não maximizado;
- oferecer maximização/reversão imediata da área de trabalho;
- manter Project como escolha independente; ocultá-lo apenas na maximização
  explícita, sem apagar a preferência de largura;
- recalcular cols/rows do PTY sem rajadas de resize por pixel.
```

### 13.2 Visual

```text
AI Terminal
external CLI: Claude CLI
context: build failure
workspace: motor-control
```

Mostrar no topo do terminal:

```text
This is an external AI CLI configured by the user.
Kinein only prepared the context and launched the command.
```

Essa mensagem pode ser compacta.

Depois do primeiro uso, basta um header técnico compacto com perfil, comando
efetivo e ações de ampliar, trocar e encerrar. A mensagem explicativa não deve
reduzir permanentemente a área de digitação.

---

## 14. UI: Context Builder

Não é chat. É um preview técnico.

### 14.1 Layout

```text
AI Context Preview
├── Profile: Claude CLI
├── Command: claude < context.md
├── Working dir: ${workspaceRoot}
├── Included context
│   ├── build log excerpt
│   ├── active file
│   ├── CMake preset
│   └── toolchain summary
├── Removed sensitive data
│   └── 2 entries masked
└── [Open AI Terminal] [Copy Context] [Cancel]
```

---

## 15. Integração com Project Health

Quando a IDE detectar um problema determinístico, ela deve mostrar ações principais sem IA:

```text
[Fix Setup]
[Open Settings]
[Open Docs]
[Copy Command]
```

A ação de IA deve ser secundária:

```text
[Open in AI Terminal]
```

Isso reforça que IA é auxiliar, não base do produto.

---

## 16. Integração com Build Failure

Build falhou:

```text
Primary:
[Open error]
[Run configure]
[Open CMake]
[Copy log]

Secondary:
[Open in AI Terminal]
```

O botão de IA não deve ser o primeiro se houver reparo determinístico claro.

---

## 17. Integração com Assistente

A Parte 7 deve ser reinterpretada.

### 17.1 Antes

```text
Assistente = painel de IA dentro da IDE.
```

### 17.2 Agora

```text
Assistente = contexto determinístico da IDE.
AI CLI Bridge = ponte opcional para ferramenta externa.
```

O nome “Assistente” ainda pode existir, mas com outra função:

```text
- mostrar estado técnico;
- listar evidências;
- preparar contexto;
- abrir docs;
- abrir AI Terminal;
- nunca conversar dentro da IDE.
```

---

## 18. Arquitetura interna

### 18.1 Módulos Rust sugeridos

```text
crates/
  kinein-context/
    context_builder.rs
    context_types.rs
    context_sanitizer.rs
    context_preview.rs

  kinein-ai-bridge/
    ai_profile.rs
    ai_launcher.rs
    ai_context_mode.rs
    ai_terminal_session.rs
    command_template.rs

  kinein-terminal/
    terminal_session.rs
    terminal_kind.rs
    shell_profile.rs
    process_launcher.rs
```

### 18.2 UI/QML sugerida

```text
ui/components/ai_bridge/
  AiTerminalTab.qml
  AiContextPreview.qml
  AiProfileSettings.qml
  AiLaunchButton.qml
  AiContextBadge.qml
```

---

## 19. JSON-RPC sugerido

### 19.1 Listar perfis

```json
{
  "method": "aiBridge.profiles.list",
  "params": {}
}
```

### 19.2 Criar contexto

```json
{
  "method": "aiBridge.context.create",
  "params": {
    "source": "build_failure",
    "include_active_file": true,
    "include_toolchain": true,
    "include_cmake": true
  }
}
```

### 19.3 Preview

```json
{
  "method": "aiBridge.context.preview",
  "params": {
    "context_id": "ctx_001"
  }
}
```

### 19.4 Abrir terminal IA

```json
{
  "method": "aiBridge.terminal.open",
  "params": {
    "profile_id": "claude-default",
    "context_id": "ctx_001",
    "mode": "file_arg"
  }
}
```

### 19.5 Copiar contexto

```json
{
  "method": "aiBridge.context.copyToClipboard",
  "params": {
    "context_id": "ctx_001"
  }
}
```

---

## 20. Configuração local

Arquivo sugerido:

```text
~/.config/kinein/ai-bridge.json
```

Exemplo:

```json
{
  "version": 1,
  "default_profile": "claude-default",
  "preview_policy": "always_for_files",
  "profiles": [
    {
      "id": "claude-default",
      "name": "Claude CLI",
      "command": "claude",
      "args": [],
      "context_mode": "stdin",
      "working_directory": "${workspaceRoot}",
      "enabled": true
    }
  ]
}
```

Esse arquivo é local, não deve ser versionado no projeto.

---

## 21. Segurança

### 21.1 Ações de baixo risco

```text
abrir AI Terminal sem contexto
copiar seleção curta
criar contexto local
preview de contexto
```

### 21.2 Ações de médio risco

```text
incluir arquivo atual completo
incluir log de build
incluir CMakeLists/CMakePresets
passar contexto para CLI externa
```

### 21.3 Ações de alto risco

```text
incluir múltiplos arquivos
incluir diretórios inteiros
incluir arquivos com possíveis secrets
executar comando customizado não revisado
```

Ações de alto risco devem exigir confirmação.

---

## 22. O que não implementar

Para manter a decisão limpa, não implementar:

```text
- chat lateral dentro da IDE;
- histórico de conversa dentro da IDE;
- resposta da IA renderizada como painel principal;
- aplicação automática de sugestões vindas da IA;
- leitura automática da saída da IA para editar código;
- provider cloud embutido;
- API key manager para IA;
- agente autônomo.
```

Se no futuro houver integração mais profunda, ela deve ser opcional e separada do MVP.

---

## 23. MVP desta parte

Implementação inicial recomendada:

```text
[ ] Terminal comum e AI Terminal separados.
[ ] Settings para configurar comando da IA CLI.
[ ] Command Palette: Open AI Terminal.
[ ] Command Palette: Open AI Terminal with Selection.
[ ] Command Palette: Open AI Terminal with Build Error.
[ ] Context Builder em markdown.
[ ] Sanitização básica de secrets.
[ ] Preview de contexto.
[ ] Suporte a stdin, arquivo temporário e clipboard.
[ ] Badge visual indicando external AI CLI.
```

---

## 24. Pós-MVP

```text
[ ] Múltiplos perfis de IA CLI.
[ ] Templates por ferramenta.
[ ] Context recipes.
[ ] Histórico local de contextos gerados.
[ ] Integração com Project Health.
[ ] Integração com Support Bundle sanitizado.
[ ] Integração com docs locais.
[ ] Limpeza automática de contextos temporários.
[ ] Comando kinein context export.
```

---

## 25. Resumo executivo

A Kinein Vectis não deve ter IA embutida.  
Ela deve ter uma ponte para IA CLI externa.

A arquitetura correta é:

```text
IDE determinística
  ↓
Context Builder seguro
  ↓
AI CLI Bridge
  ↓
Terminal IA externo
```

A IA é atalho, não centro do produto.

Isso preserva:

```text
- foco;
- privacidade;
- simplicidade;
- controle do usuário;
- arquitetura limpa;
- MVP viável;
- identidade profissional da IDE.
```

Frase-guia:

```text
A Kinein não conversa por você.
Ela organiza o contexto e abre a ferramenta que você escolheu.
```
