# 16 — Modos de Compilador e Loja de Funções Pós-V1

## Objetivo

Depois da V1.0, o Kernwerk Studio deve evoluir para oferecer uma camada visual
de configuração de compilador, linter, formatador e qualidade para C/C++ e
Rust.

A ideia central é permitir que o usuário ative "modos de compilador" e funções
de qualidade sem decorar flags, editar CMake manualmente ou pesquisar regras em
vários lugares.

Isso é pós-V1. Até a V1.0, a prioridade continua sendo entregar uma IDE
funcional para C/C++ e Rust, com build, run, testes, quality, debug, Git,
terminal e LSP em estado usável.

## O que é um modo de compilador

Um modo de compilador é um perfil nomeado que aplica um conjunto coerente de
opções ao projeto.

Exemplos futuros:

```text
Relaxed
Balanced
Strict
Strict ISO
Debug Sanitized
Release Hardened
Embedded Strict
Legacy Compatibility
```

Cada modo deve gerar configuração real e auditável:

- CMake/CMakePresets;
- flags de compilador;
- flags de linker;
- clang-tidy;
- clang-format;
- cargo/rustc;
- clippy;
- rustfmt;
- sanitizers;
- testes/quality gates.

A IDE não deve esconder o que está fazendo. Ao ativar um modo, o usuário deve
conseguir ver quais opções serão aplicadas, por que elas existem e qual fonte
as justifica.

## Experiência visual esperada

Essa funcionalidade deve ser visual, guiada e confortável. A intenção não é
criar mais uma tela onde o usuário edita flags manualmente; é criar uma janela
de opções de ambiente de projeto com a sensação de facilidade das IDEs
JetBrains.

A experiência deve funcionar para dois perfis:

- iniciante que não sabe quais flags ativar;
- usuário avançado que quer controle e auditoria.

Direção de UI:

- janela "Ambiente do Projeto" ou "Quality Center";
- lista de modos prontos no topo;
- catálogo de funções pesquisável;
- filtros por linguagem, categoria, fonte, risco e ferramenta;
- cada função com nome humano, descrição curta e impacto técnico;
- painel lateral com detalhes, comandos equivalentes e arquivos alterados;
- prévia das mudanças antes de aplicar;
- confirmação explícita para alterações destrutivas ou relaxamentos;
- botão para reverter uma função ou perfil aplicado;
- indicação clara quando uma ferramenta necessária não está instalada.

Um item visual da loja deve parecer com isto em conceito:

```text
[ ] Strict ISO C++23
    Usa C++23 sem extensões do compilador e ativa diagnóstico pedântico.
    Fonte: ISO / Standard
    Impacto: altera CMakePresets/CMakeLists
    Risco: pode quebrar código que depende de extensão GNU/Clang

[ ] AddressSanitizer
    Detecta uso de memória inválido em builds Debug.
    Fonte: toolchain oficial
    Impacto: altera preset Debug
    Risco: aumenta tempo de execução e consumo de memória
```

O usuário deve conseguir entender a configuração sem conhecer de antemão:

- `-Wall`;
- `-Wpedantic`;
- `CMAKE_CXX_EXTENSIONS`;
- sanitizers;
- `cargo clippy`;
- `rustfmt`;
- detalhes de CMakePresets.

A IDE pode mostrar esses detalhes, mas como explicação e auditoria, não como a
primeira coisa que o iniciante precisa dominar.

## O que é a loja de funções

A "loja de funções" não é uma loja comercial e não deve baixar código
arbitrário. Ela é um catálogo visual de capabilities, regras e opções que o
Kernwerk Studio sabe aplicar.

Ela pode conter, por exemplo:

- ativar C++23 estrito;
- desativar extensões não padronizadas;
- ativar warnings como erro;
- ativar sanitizers;
- ativar clang-tidy;
- ativar clang-format;
- ativar LTO/hardening;
- ativar `cargo clippy`;
- ativar `rustfmt`;
- ativar lints Rust mais rígidos;
- ativar perfil embedded;
- gerar preset CMake específico;
- adicionar quality gate de build/test.

Cada item deve declarar:

```text
id
nome
linguagem
categoria
origem/fonte
descrição curta
explicação para iniciantes
nível de confiança
risco de quebra
modo recomendado
ferramentas necessárias
arquivos que altera
comando equivalente
como reverter
```

Campos importantes para a UI:

- nome: deve ser legível e direto, por exemplo "Desativar extensões do
  compilador", não apenas `CMAKE_CXX_EXTENSIONS OFF`;
- descrição curta: uma frase sobre o benefício;
- explicação para iniciantes: texto simples, sem jargão desnecessário;
- impacto: quais arquivos e comandos mudam;
- risco: o que pode quebrar;
- fonte: por que a regra é confiável;
- reversão: como voltar atrás.

## Ordem de prioridade das funções

A loja deve apresentar e priorizar funções por confiança técnica. A regra é
começar pelo que é mais normativo, estável e seguro, e só depois expor opções
mais específicas ou opinativas.

Ordem desejada para C/C++:

```text
1. ISO / Standard
2. Diagnósticos oficiais do compilador
3. Recursos oficiais do toolchain
4. Ferramentas abertas maduras
5. Perfis do Kernwerk Studio
6. Regras de projeto/time
7. Opções experimentais ou específicas de fornecedor
```

Exemplos:

- ISO / Standard:
  - C++23;
  - `CMAKE_CXX_EXTENSIONS OFF`;
  - evitar extensões GNU quando o modo for ISO.
- Diagnósticos oficiais do compilador:
  - `-Wall`;
  - `-Wextra`;
  - `-Wpedantic`;
  - `-Wconversion`;
  - `-Werror`, quando o modo permitir.
- Recursos oficiais do toolchain:
  - AddressSanitizer;
  - UndefinedBehaviorSanitizer;
  - LTO;
  - hardening flags suportadas.
- Ferramentas abertas maduras:
  - clang-format;
  - clang-tidy;
  - CTest;
  - cppcheck, se for adotado futuramente.
- Perfis Kernwerk:
  - Strict ISO;
  - Debug Sanitized;
  - Release Hardened;
  - Embedded Strict.
- Regras de projeto/time:
  - convenções locais;
  - padrões internos;
  - relaxamentos documentados para legado.
- Opções experimentais/fornecedor:
  - flags específicas de Clang/GCC/MSVC;
  - análises instáveis;
  - opções que podem variar muito entre versões.

Ordem desejada para Rust:

```text
1. Rust Reference / comportamento oficial da linguagem
2. rustc oficial
3. Cargo oficial
4. rustfmt oficial
5. clippy oficial
6. Perfis do Kernwerk Studio
7. Regras de projeto/time
8. Ferramentas externas maduras
9. Opções nightly/experimentais
```

Rust não tem ISO como C++, então a fonte normativa principal deve ser o
ecossistema oficial da linguagem: Rust Reference, rustc, Cargo, rustfmt e
clippy.

## Modos rígidos e menos rígidos

O usuário deve poder escolher o nível de rigidez. O padrão do Kernwerk Studio
continua sendo rígido, mas a IDE deve reconhecer que há projetos legados,
projetos embarcados e projetos experimentais.

Modelo inicial:

```text
Strict ISO       padrão recomendado para C/C++ novo
Strict           rígido, mas aceita algumas escolhas práticas
Balanced         bom para projetos reais em evolução
Relaxed          legado ou integração temporária
Embedded Strict  rígido com restrições próprias de embarcados
```

Regras:

- relaxar deve ser escolha explícita;
- cada relaxamento deve ter motivo visível;
- a IDE deve mostrar o impacto antes de aplicar;
- a IDE deve preferir configurações reversíveis;
- o core deve gerar arquivos, não depender de estado invisível da UI.

## Fluxo de uso esperado

Fluxo para projeto novo:

```text
Novo Projeto
→ Escolher template C++ ou Rust
→ Escolher modo inicial: Strict ISO / Strict / Balanced / Relaxed
→ Ver resumo visual do que será ativado
→ Criar projeto com arquivos reais já configurados
```

Fluxo para projeto existente:

```text
Configurações do Projeto
→ Ambiente / Quality Center
→ Detectar CMake/Cargo/toolchain atual
→ Mostrar modo atual ou "configuração customizada"
→ Sugerir funções aplicáveis
→ Pré-visualizar alterações
→ Aplicar somente com confirmação
```

Fluxo para iniciante:

```text
Escolher "Strict recomendado"
→ IDE mostra explicações simples
→ usuário aplica
→ IDE gera configuração
→ painel Problems/Quality mostra resultados
```

Fluxo para avançado:

```text
Abrir catálogo
→ Filtrar por ISO / Clang / Rust / Sanitizer / Performance
→ Ver flags e arquivos alterados
→ Aplicar parcialmente
→ Revisar diff
```

Esse desenho mantém o espírito JetBrains-like: opções profundas existem, mas
ficam organizadas e explicáveis em telas previsíveis.

## Relação com V1.0

Essa ideia não deve bloquear a V1.0.

Antes da V1.0, o projeto deve priorizar:

- Git básico;
- build/run/test/quality para C/C++ e Rust;
- debug básico com GDB/LLDB;
- LSP suficiente para uso real;
- terminal dedicado para IA via CLI;
- dogfooding do próprio Kernwerk Studio.

Depois da V1.0, a loja de funções e os modos de compilador entram como uma
camada de produto para transformar configurações complexas em escolhas visuais,
auditáveis e bem ordenadas.

## Relação com arquitetura

A UI pode apresentar a loja e os modos, mas não deve aplicar regras sozinha.

Responsabilidades:

- UI: mostrar catálogo, explicar impacto, pedir confirmação.
- Core: detectar contexto, validar, gerar plano, aplicar mudanças em arquivos
  do projeto.
- Protocol: representar plano, diff, confirmação e resultado.
- Tooling: usar compiladores, linters e formatadores reais.

Operações futuras devem seguir o padrão:

```text
quality.catalog.list
quality.profile.preview
quality.profile.apply
quality.rule.preview
quality.rule.apply
quality.rule.revert
quality.environment.detect
```

Os nomes exatos do protocolo podem mudar, mas a regra arquitetural não muda:
a UI não chama compilador, não edita configuração crítica sozinha e não
instala ferramenta sem confirmação.

## Segurança e confiança

Cada função deve ser classificada por confiança:

```text
Normativa       ISO, Rust oficial, documentação oficial do compilador
Oficial         ferramenta oficial do ecossistema
Madura          ferramenta aberta consolidada
Kernwerk        preset opinativo do projeto
Local           regra do projeto/time
Experimental    instável, nightly, vendor-specific ou de alto risco
```

Essa classificação deve guiar a ordem visual. Itens normativos aparecem antes;
itens experimentais devem aparecer mais abaixo e com aviso claro.

## Decisão

Modos de compilador e loja de funções são uma direção importante do
Kernwerk Studio pós-V1, especialmente para C/C++ e Rust.

Eles devem existir para reduzir configuração manual, aumentar rigor e tornar
visível o motivo de cada regra. Porém, não devem atrasar a V1.0 nem virar uma
camada mágica. Tudo precisa ser auditável, reversível e baseado em ferramentas
reais.
