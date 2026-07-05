# Kernwerk Studio — Error UX e Diagnostics UX

> Este documento define como erros, falhas, warnings, diagnósticos e mensagens de ferramentas devem aparecer na IDE. Uma IDE profissional não apenas mostra erro: ela explica contexto, origem, impacto e ações possíveis.

---

## 1. Objetivo

O Kernwerk deve transformar erros brutos de ferramentas em mensagens úteis.

Exemplo ruim:

```text
Process exited with code 1
```

Exemplo bom:

```text
CMake configure falhou.

Comando:
cmake --preset debug-strict

Causa provável:
Preset "debug-strict" não existe em CMakePresets.json.

Ações:
[Gerar preset] [Abrir CMakePresets.json] [Ver log completo]
```

---

## 2. Tipos de diagnóstico

```text
Tool Diagnostic:
erro de ferramenta ausente ou falha externa.

Build Diagnostic:
erro/warning de compilação.

LSP Diagnostic:
erro vindo do clangd/rust-analyzer.

Quality Diagnostic:
falha em fmt, clippy, clang-tidy, tests.

Config Diagnostic:
erro em JSON/schema/config.

Remote Diagnostic:
falha SSH/deploy/debug remoto.

Editor Diagnostic:
arquivo alterado fora, encoding, conflito.

System Diagnostic:
permissão, falta de pacote, path inválido.
```

---

## 3. Modelo de diagnóstico

```json
{
  "id": "diag-001",
  "severity": "Error",
  "source": "cmake",
  "category": "Build",
  "message": "Preset debug-strict não encontrado",
  "file": "CMakePresets.json",
  "line": 12,
  "column": 5,
  "command": "cmake --preset debug-strict",
  "actions": [
    "open.file",
    "cmake.generatePreset",
    "logs.open"
  ]
}
```

---

## 4. Severidades

```text
Fatal:
impede operação principal.

Error:
falha que precisa correção.

Warning:
problema importante, mas não bloqueia tudo.

Info:
mensagem útil.

Hint:
sugestão leve.
```

---

## 5. Origem do erro

Sempre mostrar origem:

```text
clangd
rust-analyzer
cargo
cmake
ninja
gdb
ssh
rsync
qemu
OpenOCD
Kernwerk Core
Quality Center
```

Regra:

```text
Usuário deve saber se o erro é da IDE, da ferramenta ou do projeto.
```

---

## 6. Problems Panel

Deve mostrar:

```text
severidade;
mensagem;
arquivo;
linha;
origem;
ação rápida;
filtro.
```

Agrupamentos:

```text
por arquivo;
por ferramenta;
por severidade;
por target;
por quality check.
```

---

## 7. Build Output

Build output deve preservar log bruto, mas também extrair problemas estruturados.

Regra:

```text
Nunca esconder log bruto.
Sempre oferecer interpretação visual quando possível.
```

---

## 8. Mensagens úteis

Toda mensagem importante deve tentar responder:

```text
O que aconteceu?
Qual ferramenta falhou?
Qual comando foi executado?
Qual arquivo/linha?
Qual impacto?
Como corrigir?
Onde está o log completo?
```

---

## 9. Ações rápidas

Exemplos:

```text
[Instalar ferramenta]
[Selecionar binário]
[Gerar CMake configure]
[Abrir arquivo]
[Abrir settings]
[Reiniciar LSP]
[Rodar cargo fmt]
[Rodar cargo clippy]
[Abrir log]
[Explicar com IA]
```

IA é opcional e sob demanda.

---

## 10. Erro de ferramenta ausente

Exemplo:

```text
clangd não encontrado.

Impacto:
Autocomplete, diagnostics e refatorações C/C++ ficarão limitados.

Ações:
[Selecionar clangd manualmente]
[Continuar sem clangd]
[Abrir Toolchain Setup]
```

---

## 11. Erro de compile_commands.json

```text
compile_commands.json não encontrado.

Impacto:
clangd pode não entender includes, defines e flags do projeto.

Ações:
[Rodar CMake Configure]
[Selecionar compile_commands.json]
[Continuar com suporte limitado]
```

---

## 12. Erro remoto

```text
Falha ao conectar via SSH.

Host:
edge@192.168.0.50

Causa possível:
host offline, chave inválida ou known_hosts recusou conexão.

Ações:
[Testar novamente]
[Abrir terminal]
[Editar Target Profile]
[Ver log]
```

---

## 13. Logs

Todo erro deve ter link para log local.

```text
~/.cache/kernwerk-studio/logs/
```

Regra:

```text
Logs completos ficam locais.
Nada é enviado.
```

---

## 14. Critérios de aceite

Diagnostics UX MVP pronto quando:

```text
erros têm origem;
erros têm severidade;
build errors aparecem no Problems;
tool missing tem mensagem clara;
logs são acessíveis;
ações rápidas existem;
erro não vira panic.
```

---

## 15. Decisão final

Erro bom é erro que ensina o usuário a resolver.

Kernwerk deve ser rígido, mas não obscuro.
