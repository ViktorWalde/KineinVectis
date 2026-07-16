# Kinein Vectis — Parte 7: KV Context, IA, Documentação e Assistência Técnica

> ## ⚠ HISTÓRICO — SUPERADO
>
> **Esta parte não é fonte de verdade.** Ela foi substituída conceitualmente
> pela Parte 7.1,
> [`KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md`](KINEIN_VECTIS_AI_CLI_BRIDGE_EXTERNAL_TERMINAL.md),
> que é a fonte de verdade para IA.
>
> **O que mudou:** este documento descrevia o KV Context como um painel de
> assistência com IA embutida e chat lateral. A decisão vigente é o oposto:
>
> ```text
> não existe IA embutida  ·  não existe chat lateral na IDE
> IA = atalho para uma CLI externa que o usuário instalou e escolheu
> KV Context = terminal dedicado sobre o mesmo TerminalManager
> ```
>
> **O que continua válido aqui:** as ideias de contexto determinístico,
> preview antes de aplicar, evidência e sanitização. Elas foram absorvidas pela
> Parte 7.1; leia-as aqui apenas como origem do raciocínio.
>
> Mantido por valor histórico. Não implementar nada a partir deste documento
> sem confrontar com a Parte 7.1.

> **Status:** especificação de produto e arquitetura visual.  
> **Escopo:** painel KV Context, assistência local/externa, explicação de erros, CMake/toolchain, documentação, privacidade e integração com o core.  
> **Fora de escopo nesta parte:** simulação OpenGL pesada, marketplace de plugins, telemetria remota obrigatória e auto-modificação irrestrita de código.

---

## 1. Objetivo da Parte 7

O **KV Context** é o painel de assistência da Kinein Vectis. Ele não deve ser tratado como “chat genérico dentro da IDE”. Ele deve ser um **assistente técnico contextual**, focado em ajudar o usuário a resolver problemas reais de C, C++, Rust, CMake, toolchains, build, debug, embarcados e documentação.

A ideia central é:

```text
O usuário não deveria parar de programar para decifrar toolchain, CMake, linker, clangd,
rust-analyzer, targets, sysroot, QEMU, flash, serial ou logs de build.
A IDE deve transformar contexto técnico bruto em ações claras, seguras e rastreáveis.
```

O KV Context deve ser útil desde o MVP, mas desenhado para escalar no longo prazo.

---

## 2. Princípios do KV Context

### 2.1 Assistente quieto, não invasivo

O painel não deve dominar a IDE. Ele deve ficar disponível quando útil, mas nunca competir com o editor.

**Regra de UX:**

```text
Editor = área principal.
KV Context = apoio lateral.
Build/Debug/Terminal = evidência operacional.
Ações automáticas = sempre revisáveis.
```

O usuário deve sentir:

```text
“Tem um engenheiro auxiliar do lado, mas eu ainda controlo tudo.”
```

---

### 2.2 Contexto antes de conversa

A IA não deve começar perguntando genericamente “como posso ajudar?”. Ela deve entender o estado atual da IDE:

- arquivo aberto;
- seleção atual;
- símbolo sob o cursor;
- projeto ativo;
- linguagem detectada;
- target ativo;
- perfil de build;
- último erro de build;
- último erro de linker;
- toolchain ativa;
- CMake preset ativo;
- clangd/rust-analyzer status;
- serial monitor ativo;
- debug session ativa;
- documentos locais relevantes.

O painel deve mostrar esse contexto de forma transparente para o usuário.

---

### 2.3 Local-first e privacidade por padrão

A Kinein deve ser pensada como IDE **local-first**. O KV Context deve funcionar com IA local quando possível e com IA externa apenas se o usuário configurar explicitamente.

**Regras rígidas:**

```text
- Nenhum arquivo do workspace deve ser enviado para IA externa sem consentimento explícito.
- Nenhuma chave, token, .env, credencial SSH ou segredo deve ser incluído em prompts.
- O usuário deve poder ver o contexto que será enviado.
- O usuário deve poder escolher entre modo local, externo ou desativado.
- A IDE deve funcionar sem IA.
```

---

## 3. Modos do KV Context

O KV Context deve ter modos claros. Esses modos podem virar abas ou chips no topo do painel.

### 3.1 Contexto

Mostra uma leitura técnica do estado atual:

```text
Arquivo atual: src/main.cpp
Linguagem: C++
Build system: CMake
Preset: Debug
Compiler: clang++ 18
Target: x86_64-linux
LSP: clangd ativo
Último build: falhou
Erro principal: undefined reference
```

Esse modo é passivo, sem “chat”.

---

### 3.2 Explicar

Explica:

- trecho de código selecionado;
- erro de compilação;
- erro de linker;
- warning;
- diagnóstico do clangd/rust-analyzer;
- trecho de CMake;
- comando gerado pela IDE;
- configuração de target;
- log serial;
- falha de flash;
- falha de debug.

O foco é **explicação curta, técnica e acionável**.

---

### 3.3 Corrigir

Sugere correções, mas com segurança.

Correções possíveis:

- editar CMakeLists.txt;
- ajustar CMakePresets.json;
- adicionar include directory;
- corrigir target_link_libraries;
- sugerir pacote faltante;
- trocar compiler path;
- corrigir cargo feature;
- ajustar launch/debug config;
- explicar flags incompatíveis;
- sugerir geração de compile_commands.json.

Nunca aplicar automaticamente sem revisão.

---

### 3.4 Toolchain

Focado em ambiente:

- compilador detectado;
- versões;
- PATH;
- sysroot;
- SDK;
- cross compiler;
- CMake toolchain file;
- Ninja/Make;
- clangd;
- rust-analyzer;
- cargo;
- rustup;
- gdb/lldb;
- OpenOCD;
- probe-rs;
- QEMU.

Esse modo deve explicar o que está faltando e como corrigir.

---

### 3.5 Docs

Mostra documentação relevante:

- documentação local do projeto;
- README;
- docs internos;
- CMake docs offline/cacheadas;
- man pages;
- exemplos do próprio projeto;
- documentação oficial configurada pelo usuário;
- snippets explicativos.

O painel deve priorizar documentação local antes de sugerir internet.

---

### 3.6 Ask

Chat técnico contextual, mas restrito ao domínio do projeto.

Perguntas esperadas:

```text
Por que meu CMake não achou a biblioteca?
Como configurar clangd para este projeto?
O que esse erro de linker significa?
Como criar uma configuração Debug para esse target?
Qual arquivo provavelmente preciso alterar?
Como transformar esse CMake em preset?
Por que o rust-analyzer não está indexando?
Como configurar cross compile para ARM?
```

---

## 4. Layout visual do painel KV Context

### 4.1 Posição

O KV Context deve ficar preferencialmente no lado direito da IDE.

```text
┌──────────────────────────────────────────────┬───────────────────┐
│ Editor                                       │ KV Context        │
│                                              │ Contexto          │
│ Código C/C++/Rust                            │ Explicar          │
│                                              │ Corrigir          │
│                                              │ Toolchain         │
│                                              │ Docs              │
└──────────────────────────────────────────────┴───────────────────┘
```

O painel deve ser redimensionável e recolhível.

---

### 4.2 Larguras recomendadas

```text
Mínima: 280 px
Confortável: 360–420 px
Expandida: 520–640 px
Modo tela cheia opcional: command palette / modal
```

Em telas pequenas, o painel deve virar aba inferior ou overlay temporário.

---

### 4.3 Hierarquia visual interna

Estrutura recomendada:

```text
KV Context
├── Header
│   ├── título
│   ├── modelo/modo atual
│   ├── status local/external/offline
│   └── botão de configurações
├── Context chips
│   ├── C++
│   ├── CMake
│   ├── Debug
│   ├── x86_64-linux
│   └── clangd
├── Mode tabs
│   ├── Contexto
│   ├── Explicar
│   ├── Corrigir
│   ├── Toolchain
│   └── Docs
├── Main content
│   ├── cards de resumo
│   ├── cards de erro
│   ├── ações sugeridas
│   └── referências
└── Input/action area
    ├── campo de pergunta
    ├── anexar contexto
    └── executar ação
```

---

## 5. Componentes específicos do KV Context

### 5.1 Context Card

Card compacto com estado técnico.

Exemplo:

```text
Build failed
Target: motor_firmware
Compiler: arm-none-eabi-g++
Main issue: undefined reference to MotorDriver::init()
```

Estados:

```text
neutral
success
warning
error
running
stale
```

---

### 5.2 Evidence Block

Bloco que mostra evidência bruta usada pela IA.

Exemplo:

```text
Fonte:
build/Debug/compile.log: lines 120–147
```

Regras:

```text
- Sempre mostrar origem quando possível.
- Nunca esconder que a resposta foi baseada em logs.
- Permitir copiar evidência.
- Permitir abrir arquivo/linha.
```

---

### 5.3 Suggested Action

Ação sugerida com botão explícito.

Exemplo:

```text
Adicionar target_link_libraries(app PRIVATE motor_driver)

[Ver diff] [Aplicar] [Ignorar]
```

A ação deve ter:

```text
id
title
risk_level
affected_files
preview_diff
rollback_hint
requires_user_confirmation
```

---

### 5.4 Diff Preview

Qualquer correção de arquivo deve passar por preview.

```diff
 target_link_libraries(app PRIVATE
+    motor_driver
 )
```

O usuário deve poder aceitar/rejeitar trechos.

---

### 5.5 Doc Reference Card

Card de referência documental:

```text
CMake target_link_libraries
Fonte: documentação CMake cacheada
Relevância: alta
```

Não deve despejar documentação longa. Deve resumir e linkar/abrir.

---

### 5.6 Prompt Transparency Drawer

Um drawer opcional para usuário avançado ver o prompt/contexto.

Deve mostrar:

```text
- arquivos incluídos;
- trechos incluídos;
- logs incluídos;
- diagnósticos incluídos;
- secrets removidos;
- modelo usado;
- destino: local ou externo.
```

---

## 6. Fontes de contexto

O KV Context deve montar contexto a partir de camadas, nunca apenas enviar o workspace inteiro.

### 6.1 Contexto imediato

```text
arquivo aberto
seleção atual
posição do cursor
símbolo atual
diagnóstico sob cursor
aba ativa
```

### 6.2 Contexto de projeto

```text
CMakeLists.txt
CMakePresets.json
Cargo.toml
compile_commands.json
.kinein/workspace.json
.kinein/toolchains.json
.kinein/targets.json
```

### 6.3 Contexto operacional

```text
último build
último configure
último run
última sessão debug
logs do terminal integrado
logs de serial monitor
logs de flash
```

### 6.4 Contexto semântico

```text
clangd symbols
rust-analyzer symbols
Tree-sitter parse tree
outline local
call hierarchy
include graph
dependency graph
```

### 6.5 Contexto documental

```text
README.md
docs/
comentários de código
schemas internos
documentação cacheada
man pages
```

---

## 7. Pipeline de contexto

O core deve montar um pacote de contexto rastreável.

```text
UI action
  ↓
Context request
  ↓
Context collector
  ↓
Context sanitizer
  ↓
Context packer
  ↓
AI provider
  ↓
Response parser
  ↓
Action planner
  ↓
UI preview
  ↓
User confirmation
  ↓
Apply operation
```

---

## 8. Sanitização e proteção contra vazamento

### 8.1 Arquivos nunca enviados por padrão

```text
.env
.env.*
*.pem
*.key
id_rsa
id_ed25519
*.p12
*.pfx
secrets.*
credentials.*
token.*
.aws/
.ssh/
.git/
```

### 8.2 Padrões de segredo a mascarar

```text
API_KEY=
TOKEN=
SECRET=
PASSWORD=
PRIVATE_KEY
BEGIN RSA PRIVATE KEY
BEGIN OPENSSH PRIVATE KEY
```

### 8.3 Política

```text
Quando em dúvida, remover.
Quando remover, avisar discretamente.
Quando o usuário insistir, pedir confirmação explícita.
```

---

## 9. Provedores de IA

### 9.1 Modo desativado

A IDE continua funcional.

```text
KV Context mostra:
- diagnóstico local;
- documentação local;
- ações determinísticas;
- logs e evidências.
```

### 9.2 Modo local

Integrações futuras:

```text
Ollama
llama.cpp server
LM Studio local server
OpenAI-compatible local endpoint
```

Regras:

```text
- preferido para privacidade;
- pode ter respostas menores;
- ideal para logs, CMake, explicação local;
- deve ser configurável por URL/modelo.
```

### 9.3 Modo externo

Provedores possíveis:

```text
OpenAI API
Anthropic API
outro endpoint OpenAI-compatible
```

Regras:

```text
- opt-in explícito;
- mostrar aviso de envio;
- permitir escolher escopo;
- nunca enviar secrets;
- permitir histórico off.
```

---

## 10. Arquitetura interna sugerida

### 10.1 Crates/módulos

```text
crates/
  kinein-context/
    context_collector.rs
    context_sanitizer.rs
    context_packer.rs
    context_types.rs

  kinein-ai/
    provider.rs
    local_provider.rs
    external_provider.rs
    prompt_templates.rs
    response_parser.rs

  kinein-actions/
    action_plan.rs
    diff_preview.rs
    apply_operation.rs
    rollback.rs

  kinein-docs/
    doc_index.rs
    doc_search.rs
    doc_cache.rs
```

### 10.2 UI/QML

```text
ui/
  components/context/
    KvContextPanel.qml
    ContextHeader.qml
    ContextChips.qml
    ContextModeTabs.qml
    ContextCard.qml
    EvidenceBlock.qml
    SuggestedActionCard.qml
    DiffPreview.qml
    PromptTransparencyDrawer.qml
```

---

## 11. Protocolo JSON-RPC sugerido

### 11.1 Status

```json
{
  "method": "context.status",
  "params": {}
}
```

Resposta:

```json
{
  "ai_enabled": true,
  "mode": "local",
  "provider": "ollama",
  "model": "qwen2.5-coder",
  "workspace_index_ready": true,
  "last_context_pack_id": "ctx_001"
}
```

---

### 11.2 Coletar contexto

```json
{
  "method": "context.collect",
  "params": {
    "scope": "current_file",
    "include_build_logs": true,
    "include_diagnostics": true,
    "include_docs": true
  }
}
```

---

### 11.3 Explicar diagnóstico

```json
{
  "method": "context.explainDiagnostic",
  "params": {
    "diagnostic_id": "diag_123",
    "mode": "short"
  }
}
```

---

### 11.4 Explicar erro de build

```json
{
  "method": "context.explainBuildFailure",
  "params": {
    "job_id": "job_build_42"
  }
}
```

---

### 11.5 Sugerir correção

```json
{
  "method": "context.suggestFix",
  "params": {
    "source": "build_failure",
    "id": "job_build_42",
    "allow_file_edits": true
  }
}
```

---

### 11.6 Aplicar ação

```json
{
  "method": "actions.apply",
  "params": {
    "action_id": "act_001",
    "confirmation_token": "user_confirmed"
  }
}
```

---

## 12. Prompt templates

### 12.1 Explicar erro de CMake

```text
Você é o KV Context da IDE Kinein Vectis.
Explique o erro de CMake abaixo de forma técnica, curta e acionável.
Não invente arquivos. Use apenas as evidências fornecidas.
Se uma correção exigir editar arquivo, proponha diff mínimo.
```

### 12.2 Explicar erro de linker

```text
Explique o erro de linker abaixo.
Identifique:
1. símbolo faltando;
2. provável arquivo/biblioteca ausente;
3. mudança mínima no CMake;
4. como validar.
```

### 12.3 Explicar erro Rust

```text
Explique o erro Rust abaixo.
Priorize:
1. causa real;
2. regra da linguagem envolvida;
3. correção mínima;
4. impacto no código.
```

### 12.4 Corrigir CMake

```text
Sugira uma correção mínima para o CMake.
Responda com:
- explicação curta;
- arquivos afetados;
- patch proposto;
- risco da alteração.
```

---

## 13. Níveis de risco das ações

### 13.1 Low

```text
abrir documentação
explicar erro
copiar comando
abrir arquivo/linha
gerar sugestão sem editar
```

### 13.2 Medium

```text
editar CMakeLists.txt
editar CMakePresets.json
criar launch config
criar target config
alterar include path local
```

### 13.3 High

```text
executar comando shell
instalar pacote
alterar PATH global
alterar arquivo fora do workspace
enviar contexto para IA externa
apagar build directory
```

Ações high exigem confirmação explícita.

---

## 14. Integração com documentação

### 14.1 Índice local

A IDE deve indexar:

```text
README.md
docs/
*.md
*.rst
CMakePresets.json
schemas/
comentários especiais
```

### 14.2 Busca documental

O KV Context deve poder responder com base na documentação do projeto antes de IA externa.

Exemplo:

```text
“Como configuro target ARM neste projeto?”
```

Primeira tentativa:

```text
docs/targets.md
.kinein/targets.json
CMakePresets.json
```

### 14.3 Documentação oficial cacheada

No futuro:

```text
CMake
GCC
Clang
Rust
Cargo
GDB
LLDB
OpenOCD
QEMU
```

A IDE deve permitir cache offline.

---

## 15. KV Context sem IA

Mesmo sem IA, o painel deve ser útil.

Recursos determinísticos:

```text
- listar erros do build;
- agrupar diagnósticos;
- abrir arquivo/linha;
- mostrar toolchain faltante;
- sugerir comandos conhecidos;
- apontar CMakePresets ausente;
- indicar compile_commands.json ausente;
- mostrar status do clangd/rust-analyzer;
- exibir docs locais.
```

Isso garante que a Kinein não dependa de IA para ser uma IDE sólida.

---

## 16. Estados visuais do KV Context

### 16.1 Idle

```text
Sem problema ativo.
Mostra resumo do arquivo/projeto.
```

### 16.2 Observing

```text
Build rodando.
Debug ativo.
Serial monitor aberto.
```

### 16.3 Needs Attention

```text
Erro detectado.
Mostra ação primária: Explicar.
```

### 16.4 Suggesting

```text
IA ou motor local gerando sugestão.
Mostrar progresso discreto.
```

### 16.5 Awaiting Confirmation

```text
Há patch ou comando pronto.
Usuário precisa revisar.
```

### 16.6 Applied

```text
Ação aplicada.
Mostrar rollback/abrir diff.
```

---

## 17. Integração com Build, CMake e Debug

O KV Context deve ser profundamente conectado com a Parte 4.

### 17.1 Build failure

Fluxo:

```text
Build falhou
  ↓
Parser identifica erro principal
  ↓
KV Context cria card
  ↓
Usuário clica Explicar
  ↓
Contexto inclui log + CMake + target + compiler
  ↓
Resposta sugere causa e próxima ação
```

### 17.2 CMake configure failure

```text
CMake falhou
  ↓
Detectar pacote faltante / compiler inválido / generator problem
  ↓
Mostrar comando executado
  ↓
Sugerir correção mínima
```

### 17.3 Debug failure

```text
Debug não iniciou
  ↓
Verificar binário
  ↓
Verificar símbolos
  ↓
Verificar gdb/lldb
  ↓
Verificar target remoto
  ↓
Sugerir correção
```

---

## 18. Integração com sistemas embarcados

O KV Context deve ajudar em:

```text
flash falhou
OpenOCD não conectou
porta serial não abriu
permissão de /dev/ttyUSB*
target remoto offline
sysroot ausente
cross compiler não encontrado
QEMU não iniciou
```

Exemplo de resposta ideal:

```text
A falha parece estar relacionada a permissão de acesso à porta serial /dev/ttyUSB0.
Você pode validar com:
ls -l /dev/ttyUSB0

A correção comum no Linux é adicionar seu usuário ao grupo dialout/uucp,
mas isso altera permissões do sistema, então a IDE deve pedir confirmação antes de sugerir execução.
```

---

## 19. Integração com Tree-sitter, LSP e indexação

O KV Context deve saber diferenciar fontes:

```text
Tree-sitter:
- escopo local
- estrutura rápida
- seleção
- folding
- símbolos aproximados

LSP:
- tipos
- referências reais
- diagnósticos
- rename seguro
- go to definition

Indexador Kinein:
- docs
- build artifacts
- project graph
- config graph
```

Ao responder, deve preferir LSP para semântica e Tree-sitter para estrutura local.

---

## 20. Histórico e memória local

### 20.1 Histórico de sessão

Manter:

```text
últimos erros explicados
ações aplicadas
patches rejeitados
comandos executados
context packs usados
```

### 20.2 Memória de projeto

Opcional, local:

```text
preferências de toolchain
target padrão
docs relevantes
correções recorrentes
```

Nunca sincronizar sem opt-in.

---

## 21. Critérios de aceite para MVP

O MVP do KV Context deve entregar:

```text
[ ] Painel lateral recolhível.
[ ] Mostrar resumo do workspace atual.
[ ] Mostrar linguagem/target/build profile.
[ ] Mostrar último erro de build.
[ ] Explicar erro de build com base em log.
[ ] Mostrar evidências usadas.
[ ] Não enviar secrets.
[ ] Suportar modo IA desativado.
[ ] Suportar provider local configurável.
[ ] Gerar sugestão sem aplicar automaticamente.
[ ] Preview de diff antes de editar arquivo.
[ ] Integração mínima com docs locais.
```

---

## 22. Critérios de aceite para longo prazo

```text
[ ] Explicação contextual de CMake.
[ ] Explicação contextual de linker.
[ ] Explicação contextual de Rust.
[ ] Sugestões seguras de CMakePresets.
[ ] Correções guiadas de toolchain.
[ ] Busca semântica em docs locais.
[ ] Cache de docs oficiais.
[ ] Prompt transparency drawer.
[ ] Provider externo com opt-in.
[ ] Histórico local revisável.
[ ] Integração com flash/serial/debug.
[ ] Integração com QEMU.
[ ] Ações com rollback.
[ ] Configuração visual completa.
```

---

## 23. O que não implementar agora

Para proteger qualidade, não implementar no começo:

```text
- agente que edita múltiplos arquivos sem revisão;
- auto-instalação de pacotes;
- envio automático para nuvem;
- chat genérico sem contexto;
- memória global entre projetos;
- execução arbitrária de shell pela IA;
- refatorações grandes feitas por IA;
- indexação sem sanitização;
- prompts invisíveis ao usuário.
```

---

## 24. Direção visual final

O KV Context deve parecer:

```text
técnico
calmo
útil
auditável
profissional
não invasivo
```

E não deve parecer:

```text
chatbot chamativo
widget de marketing
assistente que toma controle
painel de propaganda de IA
```

A personalidade visual deve ser:

```text
JetBrains-like na organização.
Kinein-like no foco em sistemas.
Industrial, escuro, confortável e confiável.
```

---

## 25. Resumo executivo

O **KV Context** é uma camada de assistência contextual para C, C++, Rust, CMake, toolchains e sistemas embarcados.

Ele deve:

```text
- entender contexto técnico;
- explicar erros;
- sugerir correções;
- proteger privacidade;
- mostrar evidências;
- exigir revisão para alterações;
- funcionar sem IA;
- escalar para IA local e externa;
- integrar docs, build, debug, targets e LSP.
```

A meta não é “colocar IA na IDE”.  
A meta é **remover atrito técnico sem tirar controle do programador**.
