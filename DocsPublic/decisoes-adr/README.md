# decisoes-adr/ — decisões de arquitetura datadas

Cada ADR registra **uma** decisão, com contexto, alternativas e consequências.
Um ADR não se reescreve: ganha nota datada quando algo muda.

```text
ADR-0001-notify-filesystem-watcher.md     o watcher `notify` e a barreira compare-before-save
ADR-0002-tree-sitter-syntax-foundation.md  Tree-sitter como fundação sintática (C, C++, Rust;
                                           Python em 2026-09-12)
ADR-0003-linuxdeploy-appimage-packaging.md linuxdeploy/AppImage como empacotamento
ADR-0004-alacritty-terminal-emulator.md    o emulador de terminal do alacritty
ADR-0005-tres-arvores-de-documentacao.md   as árvores de documentação (duas desde 2026-09-12)
ADR-0007-odbc-com-consentimento.md         DSN local, driver nativo e consentimento de sessão
ADR-0008-previa-postgresql-no-worker.md   streaming, transação e decisão única no worker
ADR-0009-banco-e-linguagem-por-provedores.md acesso ao banco e LSP independentes,
                                           ferramentas externas e expansão modular
ADR-0010-drivers-em-processos-versionados.md atualização de drivers sem recompilar a IDE,
                                            processo adaptador e contrato negociado
```

O ADR-0006 (`exmex`) saiu do repositório com a simulação em 2026-09-12
(`roadmaps/40` §5). O número não se reaproveita.
