# Kernwerk Studio — First Run e Toolchain Setup Wizard

> Este documento define a experiência de primeira execução e configuração de toolchains. Uma IDE profissional deve orientar o usuário sem instalar coisas silenciosamente ou esconder dependências.

---

## 1. Objetivo

Na primeira execução, o Kernwerk deve detectar ferramentas e explicar o estado do ambiente.

Ele deve responder:

```text
O que está instalado?
O que está faltando?
O que é necessário para C/C++?
O que é necessário para Rust?
O que é opcional?
Como corrigir?
```

---

## 2. Princípios

```text
não instalar nada sem confirmação;
não exigir tudo de uma vez;
explicar impacto de ferramenta ausente;
permitir continuar com suporte limitado;
permitir seleção manual de binário;
salvar toolchain profile;
não salvar segredos em texto puro.
```

---

## 3. Tela inicial

```text
Bem-vindo ao Kernwerk Studio

Ambiente detectado:
✓ Git
✓ Cargo
✓ Rustc
✓ Rust Analyzer
✓ CMake
✗ Ninja
✗ clangd
✗ clang-format
✗ GDB

Perfis disponíveis:
[Configurar C/C++]
[Configurar Rust]
[Configurar Embedded/Remote]
[Continuar]
```

---

## 4. Detecção de ferramentas

Ferramentas C/C++:

```text
gcc
g++
clang
clang++
cmake
ninja
clangd
clang-format
clang-tidy
gdb
lldb
pkg-config
```

Ferramentas Rust:

```text
rustup
cargo
rustc
rustfmt
clippy
rust-analyzer
cargo-deny
cargo-audit
```

Ferramentas Embedded/Remote:

```text
ssh
scp
rsync
qemu-system-*
gdbserver
gdb-multiarch
openocd
pyocd
probe-rs
west
```

---

## 5. Status

Estados:

```text
Ready
Detected
Missing
Invalid
Disabled
Optional
```

Exemplo:

```text
clangd
Status: Missing
Impacto: C/C++ terá autocomplete e diagnostics limitados.
Ação: instalar pacote clang ou selecionar binário.
```

---

## 6. Toolchain Profile

Ao final, gerar:

```text
.kernwerk/toolchains.json
```

Exemplo:

```json
{
  "schemaVersion": 1,
  "active": "system-clang",
  "toolchains": [
    {
      "id": "system-clang",
      "name": "System Clang",
      "type": "cpp",
      "cCompiler": "/usr/bin/clang",
      "cppCompiler": "/usr/bin/clang++",
      "cmake": "/usr/bin/cmake",
      "ninja": "/usr/bin/ninja",
      "debugger": "/usr/bin/lldb",
      "languageServer": "/usr/bin/clangd"
    }
  ]
}
```

---

## 7. Instalação sugerida

A IDE pode mostrar comandos, mas não executar sem confirmação.

Exemplo Arch/CachyOS:

```bash
sudo pacman -S cmake ninja clang lldb gdb git rustup
```

Exemplo Debian/Ubuntu:

```bash
sudo apt install cmake ninja-build clang clangd clang-format clang-tidy gdb lldb git
```

Regra:

```text
Comandos de instalação devem ser sugestões visuais.
Execução exige confirmação.
```

---

## 8. Seleção manual

Usuário deve poder selecionar binários:

```text
C Compiler
C++ Compiler
CMake
Ninja
Debugger
Language Server
Formatter
Linter
```

A IDE deve validar:

```text
arquivo existe;
é executável;
--version funciona;
tipo parece correto.
```

---

## 9. Setup por perfil

Perfis:

```text
C/C++ Local
Rust Local
C/C++ Qt
C/C++ GTK
Remote Linux
Cross Linux
Bare Metal Futuro
```

Cada perfil mostra ferramentas necessárias e opcionais.

---

## 10. Não bloquear uso

Se uma ferramenta opcional faltar, permitir continuar.

Exemplo:

```text
clang-tidy ausente:
C++ ainda funciona, mas análise estática avançada ficará indisponível.
```

---

## 11. Critérios de aceite

Wizard MVP pronto quando:

```text
detecta ferramentas principais;
mostra missing/ready;
gera toolchains.json;
permite continuar;
não instala nada sozinho;
explica impacto;
funciona via CLI antes da UI.
```

---

## 12. Decisão final

A primeira execução deve reduzir ansiedade e confusão.

O usuário deve saber exatamente o que a IDE encontrou e o que ela precisa.
