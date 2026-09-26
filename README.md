# Kinein Vectis

**Linux-first IDE for embedded, systems and IoT development.**

**C · C++ · Rust · Python**

Kinein Vectis é uma IDE open source voltada para desenvolvimento de **firmware, sistemas, Linux embarcado, IoT e software de apoio ao dispositivo**.

A proposta é simples:

> **A Vectis lê o projeto e o ambiente, mostra o que encontrou, explica o que será usado e deixa a decisão com o desenvolvedor.**

Compilador, toolchain, target, sysroot, SDK, preset, language server, interpretador, debugger, placa, container e serviços locais permanecem explícitos e sob controle do usuário.

A Vectis não tenta substituir CMake, Cargo, clangd, GDB ou os SDKs dos fabricantes.

**Ela integra e orquestra essas ferramentas.**

> **Versão pública atual: [`0.2.0 — Public Beta`](https://github.com/ViktorWalde/KineinVectis/releases/tag/v0.2.0) · Linux x86_64**

**[Site oficial](https://viktorwalde.github.io/KineinSite/) · [Download 0.2.0](https://github.com/ViktorWalde/KineinVectis/releases/tag/v0.2.0) · [Documentação](https://viktorwalde.github.io/KineinSite/documentacao/) · [Discord / Comunidade](https://discord.gg/cWRkUGUmQU)**

---

## Download

A **Kinein Vectis 0.2.0** está disponível publicamente pelo GitHub Releases:

### [Download Kinein Vectis 0.2.0](https://github.com/ViktorWalde/KineinVectis/releases/tag/v0.2.0)

Pacote de distribuição:

```text
Kinein_0.2.zip
```

A distribuição contém o AppImage e os arquivos necessários para instalação e execução.

Depois de extrair o pacote, confira o SHA-256 do AppImage:

```bash
sha256sum -c Kinein-Vectis-0.2.0-x86_64.AppImage.sha256
```

O resultado deve terminar em:

```text
OK
```

Execute:

```bash
chmod +x Kinein-Vectis-0.2.0-x86_64.AppImage
./Kinein-Vectis-0.2.0-x86_64.AppImage
```

Se houver problema com FUSE:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./Kinein-Vectis-0.2.0-x86_64.AppImage
```

A série `0.x` está em **beta público**.

Bugs, mudanças de interface, alterações de configuração e fluxos ainda incompletos são esperados até a `1.0`.

---

## O foco

A Vectis foi projetada para projetos que frequentemente atravessam diferentes camadas:

```text
Firmware
   ↓
Bare Metal / RTOS
   ↓
Linux Embedded / Edge
   ↓
Gateway / Serviços
   ↓
API / Banco / Observabilidade
```

As quatro linguagens de primeira classe refletem esse objetivo:

* **C** — firmware, drivers, RTOS e systems programming;
* **C++** — firmware, sistemas, aplicações nativas e robótica;
* **Rust** — systems programming, tooling e embedded;
* **Python** — automação, testes, ferramentas de engenharia, MicroPython, gateways, APIs e serviços IoT.

A Vectis não busca ser uma IDE universal para todas as linguagens.

O objetivo é aprofundar o ecossistema de **embedded, systems e IoT**.

---

# A ideia

Um projeto C/C++ ou embedded raramente é apenas:

```text
arquivo → compilador
```

Na prática existe uma cadeia maior:

```text
Projeto
  ↓
Build System
  ↓
Preset / Profile
  ↓
Toolchain
  ↓
Target / Sysroot
  ↓
Dependencies
  ↓
Language Server
  ↓
Debugger
  ↓
Device / Host remoto
```

A Vectis tenta tornar essa cadeia **visível, explicável e controlável**.

Sempre que possível, a IDE diferencia:

```text
Detected
   ↓
Selected
   ↓
Effective
```

Ou seja:

* o que foi encontrado;
* o que o desenvolvedor escolheu;
* o que o projeto realmente está utilizando;
* de onde cada informação foi obtida.

---

## Escolhas explícitas

Compilador, preset, kit, chip, target, sysroot, SVD, debugger, language server, formatador, interpretador e motor de containers continuam sendo escolhas do desenvolvedor.

A IDE pode sugerir configurações com base no projeto e na máquina, mas mudanças relevantes são apresentadas antes de serem aplicadas.

---

## Nenhuma instalação silenciosa

Se uma ferramenta estiver ausente, a Vectis apresenta o procedimento correspondente da distribuição ou do fabricante.

A IDE **não executa `sudo`**.

Toolchains gerenciadas pela própria Vectis ficam isoladas em sua área de dados e têm sua integridade verificada antes da instalação.

---

## Ferramentas reais

A Vectis não cria um compilador, build system, package manager ou debugger próprio.

Ela trabalha com ferramentas dos próprios ecossistemas, como:

```text
CMake
Cargo
clangd
rust-analyzer
basedpyright
clang-tidy
Ruff
LLDB
GDB
debugpy
Git
Docker
Podman
```

além dos SDKs e utilitários relacionados aos targets do projeto.

---

## Local-first

Não há telemetria.

Código e informações do projeto não são enviados para servidores da Kinein.

---

## Sem IA embutida

A Vectis não possui chat ou assistente de IA integrado.

Agentes e ferramentas CLI podem ser utilizados normalmente pelo terminal integrado, caso o desenvolvedor queira.

A IDE continua independente de qualquer fornecedor ou modelo de IA.

---

# Embedded & Firmware

Embedded é uma das áreas centrais da Vectis.

O fluxo é organizado em torno de:

```text
Placa
  ↓
Projeto
  ↓
Kit
  ↓
Build
  ↓
Flash
  ↓
Debug
```

### Ecossistemas reconhecidos

* ESP-IDF
* Zephyr
* Raspberry Pi Pico / pico-sdk
* PlatformIO
* STM32Cube
* Rust bare metal
* MicroPython
* Yocto
* Buildroot

### Hardware

A IDE possui suporte para:

* descoberta de portas seriais sem abri-las;
* identificação de dispositivos;
* monitor serial;
* seleção de chip e target;
* SDKs e sysroots;
* kits por alvo;
* debug probes;
* QEMU.

### Flash

Dependendo do projeto e do dispositivo:

```text
esptool
probe-rs
picotool
dfu-util
idf.py
west
pio
```

Permissões, dispositivo e ferramenta permanecem explícitos antes da execução.

### Debug

O fluxo atual contempla:

* GDB/DAP;
* LLDB;
* QEMU;
* memória;
* disassembly;
* periféricos via SVD;
* RTT / defmt quando disponíveis para o target.

---

# C e C++

A Vectis trata o **ambiente C/C++ como parte do projeto**, e não apenas como configuração do editor.

O suporte atual inclui:

* CMake;
* CMake Presets;
* detecção de compiladores;
* `compile_commands.json`;
* clangd;
* clang-format;
* clang-tidy;
* GDB / LLDB;
* gtest / Catch2;
* cobertura LCOV;
* kits;
* toolchains;
* sysroots;
* cross-compilation.

O ambiente está sendo desenvolvido para tornar cada vez mais explícitos:

```text
Compiler
Standard
Target
Preset
Dependencies
Compile Context
Link Context
Debugger
```

sem substituir CMake ou criar formatos próprios desnecessários.

---

# Rust

Projetos Rust possuem integração com:

* Cargo;
* rust-analyzer;
* rustfmt;
* Clippy;
* testes;
* cobertura;
* debug;
* targets externos;
* Rust bare metal;
* `probe-rs`.

A Vectis utiliza a própria estrutura do ecossistema Rust:

```text
rustup
Cargo.toml
Cargo.lock
Workspaces
Features
Profiles
Targets
```

em vez de criar uma configuração paralela.

---

# Python

Python é uma linguagem de primeira classe porque faz parte do fluxo completo de muitos projetos embedded e IoT.

O suporte atual inclui:

* resolução do interpretador;
* `.venv`;
* criação de ambiente com `uv`;
* basedpyright;
* Ruff;
* pytest;
* debugpy;
* execução de módulos e scripts;
* MicroPython via `mpremote`.

Python não é limitado a scripts.

A Vectis também pode ser utilizada para:

```text
Automação
Ferramentas de engenharia
Testes de hardware
Gateways
APIs
FastAPI
Serviços locais
Backends IoT
```

---

# Linux Embedded & Edge

A Vectis também trabalha com projetos cujo target é um sistema Linux.

O ecossistema contempla:

* Yocto;
* Buildroot;
* SDKs externos;
* sysroots;
* cross-toolchains;
* SSH;
* deploy;
* debug remoto;
* QEMU;
* containers.

A intenção é permitir um fluxo contínuo entre:

```text
Host de desenvolvimento
        ↓
Cross Toolchain
        ↓
Linux Target
        ↓
Deploy
        ↓
Run / Debug
```

---

# IoT, Edge e software de apoio

Nem todo projeto termina no firmware.

Um produto pode envolver:

```text
MCU
 ↓
Gateway
 ↓
Backend
 ↓
Database
 ↓
Monitoring
```

Por isso algumas integrações da Vectis também cobrem o software ao redor do dispositivo.

## Containers

Docker e Podman:

* descoberta do motor;
* ciclo de vida;
* jobs;
* logs;
* shell;
* Compose associado ao projeto.

## Bancos de dados

Suporte atual para:

* PostgreSQL;
* TimescaleDB;
* SQLite;
* MongoDB.

É possível descobrir serviços locais, executar consultas e visualizar esquemas.

Credenciais não são persistidas em texto aberto pela IDE.

## Observabilidade

A Vectis possui integração com Grafana através da API HTTP para consultar informações relacionadas ao ambiente observado pelo projeto.

Essa área continua em desenvolvimento.

---

# Desenvolvimento diário

## Editor

O editor possui:

* múltiplas abas;
* recuperação de rascunhos;
* Tree-sitter incremental;
* syntax highlighting;
* folding;
* estrutura do arquivo;
* completion;
* diagnósticos;
* navegação;
* rename;
* quick fixes com preview;
* símbolos do arquivo e do projeto.

A inteligência semântica é fornecida principalmente pelos language servers:

```text
C / C++  → clangd
Rust     → rust-analyzer
Python   → basedpyright
```

---

## Build, testes e qualidade

Dependendo do projeto:

```text
Build
Test
Lint
Static Analysis
Coverage
Run
Debug
```

Operações demoradas são executadas como **jobs assíncronos e canceláveis**, evitando bloquear a interface.

---

## Terminal

O terminal integrado utiliza PTY real e suporta múltiplas sessões.

Shells, CLIs, SDKs, ferramentas de build e agentes podem ser utilizados normalmente.

---

## Git

O fluxo Git integrado cobre operações comuns:

* status;
* diff;
* stage;
* commit;
* amend;
* commit + push;
* branches;
* pull;
* push;
* stash;
* histórico;
* grafo de refs;
* blame.

Diffs e commits podem ser abertos no próprio editor.

---

# Ambiente do projeto

Uma das áreas centrais da Vectis é construir uma visão coerente do ambiente.

```text
Projeto
├── Linguagens
├── Targets
├── Toolchains
├── Compiladores
├── Build System
├── Presets / Profiles
├── Dependências
├── Language Servers
├── Debuggers
├── Devices
└── Serviços locais
```

O objetivo é reduzir a quantidade de configuração que o desenvolvedor precisa reconstruir mentalmente toda vez que abre um projeto.

---

# Biblioteca

A **Biblioteca** organiza capacidades, ferramentas e integrações do ambiente por assunto e necessidade.

Exemplos:

```text
Build
Debug
Testing
Quality
Coverage
Profiling
Dependencies
Toolchains
Cross Compilation
Embedded
```

As capacidades também podem ser encontradas por ecossistema:

```text
C / C++
Rust
Python
Embedded
IoT / Edge
```

A intenção é permitir tanto uma busca por necessidade:

> “Quero configurar cobertura.”

quanto uma busca por ferramenta:

> “Quero usar clang-tidy.”

e chegar às mesmas capacidades relacionadas.

A Biblioteca é pensada para evoluir futuramente para comportar também:

```text
Integrations
Recipes
Extensions
Plugins
```

sem transformar toda a experiência da IDE em um marketplace.

---

# Arquitetura

A interface e a lógica de negócio são separadas:

```text
┌───────────────────────────┐
│       Qt / QML UI         │
└─────────────┬─────────────┘
              │
         JSON-RPC local
              │
┌─────────────▼─────────────┐
│         Rust Core         │
└─────────────┬─────────────┘
              │
     ┌────────┼─────────┐
     ▼        ▼         ▼
   CMake    Cargo     Python
   clangd   rust-     basedpyright
            analyzer
     │        │         │
     └────────┼─────────┘
              ▼
   Toolchains · Debuggers
      Git · Containers
       Devices · SDKs
```

A UI apresenta estado e recebe ações.

O core mantém o modelo do projeto, valida operações e orquestra ferramentas externas.

Operações longas são jobs canceláveis.

O protocolo entre frontend e core está documentado em:

```text
DocsPublic/arquitetura/03-ipc-protocol.md
```

As decisões e regras arquiteturais estão em:

```text
DocsPublic/arquitetura/ARCHITECTURE.md
```

---

# Documentação

### Instalação, atualização e distribuição

[`DocsPublic/tutorial.md`](DocsPublic/tutorial.md)

### Manual da IDE

[`DocsPublic/manual.md`](DocsPublic/manual.md)

### Compilar pelo código-fonte

[`DocsPublic/build/como-executar.md`](DocsPublic/build/como-executar.md)

### Índice da documentação

[`DocsPublic/README.md`](DocsPublic/README.md)

---

# Comunidade

A Vectis está em **beta público** e feedback de projetos reais é especialmente importante nesta fase.

### Discord

**[Entrar no servidor da Kinein Vectis](https://discord.gg/cWRkUGUmQU)**

Convite direto:

```text
https://discord.gg/cWRkUGUmQU
```

O servidor pode ser usado para:

* relatar bugs;
* discutir funcionalidades;
* sugerir melhorias;
* compartilhar experiências com C/C++, Rust, Python e embedded;
* comparar fluxos com outras IDEs;
* acompanhar o desenvolvimento das próximas versões.

Ao relatar um problema, se possível inclua:

```text
Distribuição Linux e versão
Wayland ou X11
Passos para reproduzir
Comportamento esperado
Comportamento observado
Screenshot ou log
```

Logs da interface podem ser encontrados em:

```text
~/.cache/kinein-vectis/logs/kinein-ui-erros.txt
```

---

# Contribuindo

Contribuições são bem-vindas.

Áreas de interesse incluem:

* C/C++;
* Rust;
* Python;
* embedded;
* Linux;
* Qt/QML;
* UX;
* toolchains;
* debugging;
* documentação;
* testes automatizados.

O ponto de entrada é:

[`DocsPublic/contribuindo/README.md`](DocsPublic/contribuindo/README.md)

O projeto procura manter um fluxo previsível para mudanças:

```text
Entender
   ↓
Desenhar
   ↓
Definir contrato
   ↓
Implementar core
   ↓
Implementar UI
   ↓
Testar
   ↓
Documentar
```

O uso de agentes de IA é opcional e não faz parte da arquitetura da IDE.

---

# Plataforma

A versão `0.2.0` suporta atualmente:

```text
Linux x86_64
glibc 2.36+
Wayland ou X11
```

A distribuição oficial utiliza AppImage.

Atualmente:

```text
Windows        não suportado
Linux ARM64    ainda não distribuído
```

---

# Estado do projeto

Kinein Vectis está em desenvolvimento ativo.

```text
0.x → Public Beta
1.0 → primeira versão considerada estável
```

Até a `1.0`, podem ocorrer:

* mudanças de interface;
* alterações em configurações;
* mudanças de fluxo;
* recursos incompletos;
* bugs.

Durante o beta, a prioridade é aprofundar e estabilizar os fluxos existentes antes de ampliar significativamente o escopo.

As principais áreas de desenvolvimento são:

```text
Editor & ergonomia
C/C++ environment intelligence
Rust
Python
Embedded
Linux Embedded
Cross-compilation
Debugging
Toolchains
Biblioteca
UX
Performance
Stability
Testing
```

---

# Filosofia

A Vectis não tenta esconder o ambiente de desenvolvimento atrás de configuração implícita.

A ideia é:

```text
Detectar
   ↓
Entender
   ↓
Explicar
   ↓
Escolher
   ↓
Executar
```

**O ambiente continua sendo seu.**

---

# Licença

Kinein Vectis é software open source disponibilizado sob licença dupla:

* MIT;
* Apache-2.0.

Consulte:

[`LICENSE-MIT.txt`](LICENSE-MIT.txt)

[`LICENSE-APACHE-2.0.txt`](LICENSE-APACHE-2.0.txt)

