# `tools.detect`, `tools.status`, `environment.scan` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Resultado de `tools.detect` / `tools.status`

Implementado no protocolo `0.2.0`. `tools.detect` sempre executa a detecção e
atualiza o registro interno; `tools.status` responde com o último resultado
conhecido (detectando na primeira chamada).

Desde 2026-10-16 (16.1, F4 do `roadmaps/59` §7.1.1), o probe de versão
(`--version`) roda na raiz do projeto aberto, quando há um: os proxies do
rustup (`cargo`, `rustc`, `rust-analyzer`) escolhem a toolchain pelo
diretório, e a versão informada é a que o build daquele projeto usa. Sem
projeto, roda no diretório do processo. A procura do binário não muda, e o
contrato também não; a UI pede `tools.detect` de novo a cada troca de
projeto. O mesmo vale para `environment.scan`.

Inventário atual: cargo, rustc, rustup, rust-analyzer, cmake, ninja, git,
clangd, clang, clang++ (`id: clangxx`), gcc, g++ (`id: gxx`), gdb, lldb,
ripgrep (`rg`) e fd/fdfind.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "tools": [
      {
        "id": "cargo",
        "displayName": "Cargo",
        "status": "detected",
        "path": "/home/user/.cargo/bin/cargo",
        "version": "cargo 1.96.1"
      },
      {
        "id": "clangd",
        "displayName": "clangd",
        "status": "missing",
        "suggestedInstall": "sudo pacman -S clang",
        "message": "clangd nao foi encontrado no PATH."
      }
    ]
  }
}
```

Estados possíveis de ferramenta (o ciclo de vida inteiro é esta lista):
`notConfigured`, `missing`, `detected`, `ready`, `running`, `failed`,
`disabled`. A detecção usa `missing`, `detected` e `failed`; os demais são
reservados para o gerenciamento de processos.

### Scan de ambiente (`environment.scan` — job assíncrono)

Implementado no protocolo `0.20.0`. Roda a mesma detecção de ferramentas de
`tools.detect`, mas como job assíncrono para o fluxo de First Run / Toolchain
Settings. Responde na hora com `{ "jobId" }`, não requer workspace aberto e,
ao concluir, atualiza o mesmo registry consultado por `tools.status`.

```text
event.environment.started   { "jobId", "tools" }
event.environment.tool      { "jobId", "tool": ToolInfo }
event.environment.finished  { "jobId", "success", "total", "detected", "missing", "failed", "tools": [ToolInfo] }
```

Além destes, emite `event.job.created/progress/output/finished`. A UI deve usar
`event.environment.*` para preencher telas de ambiente/toolchain e
`event.job.*` para status bar/lista de jobs. O core nunca instala ferramentas;
`suggestedInstall` continua sendo apenas uma sugestão para ação explícita do
usuário.
