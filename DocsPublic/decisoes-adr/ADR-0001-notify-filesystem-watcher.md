# ADR-0001 — `notify` para mudanças externas do workspace

- Status: aceito
- Data: 2026-07-14
- Escopo: T2 / protocolo 0.45.0

## Contexto

Buffers podem envelhecer quando `git checkout`, formatters ou outro editor
alteram o disco. Sem detecção, um save da IDE podia sobrescrever a versão
externa. As specs exigem aviso antes de overwrite, e o documento do motor
semântico exige versões de documento e workspace edits conservadores.

Implementar inotify/polling internamente violaria a regra de orquestrar
componentes maduros. O roadmap de adoção exige auditoria, pin e registro.

## Decisão

Adotar `notify` `=8.2.0` em Modo A, somente no `kinein-core`. É a versão estável
compatível com o MSRV 1.85 do workspace; a linha 9.x ainda estava em release
candidate durante a decisão. No Linux, `RecommendedWatcher` usa inotify. Se a
inicialização nativa falhar, o core usa `PollWatcher` com intervalo de dois
segundos e comparação de conteúdo.

O watcher não percorre todo o workspace recursivamente. A raiz é registrada ao
abrir; `fs.list` adiciona diretórios expandidos e `fs.read` adiciona o pai de
arquivos abertos, sempre `NonRecursive`. Eventos são filtrados, deduplicados e
agrupados em 180 ms antes de `event.fs.changed`.

> **Nota datada (2026-09-12):** o índice do projeto inteiro (`roadmaps/42`
> P0) passou a registrar no watcher **todas as pastas que caminhou**, uma a
> uma e `NonRecursive`, com a mesma lista de pastas ignoradas — a decisão
> deste ADR (nada recursivo na raiz) fica de pé; o que muda é *quem* pede o
> registro. Medido: 148 watches neste repositório. O registro para no
> primeiro erro e o relata uma vez (`event.fs.watchError`).

O watcher é apenas aviso antecipado. A garantia contra perda de dados é
`fs.write { expectedContent }`: o core compara o snapshot com o disco e retorna
`FILE_CHANGED` sem escrever quando divergem. A UI só atualiza a base esperada
após escolha explícita de manter o buffer local.

## Auditoria

- upstream oficial: `notify-rs/notify`;
- licença do crate: SPDX `CC0-1.0`;
- MSRV publicado: Rust 1.77, abaixo do MSRV 1.85 do projeto;
- runtime: sem rede, telemetria, shell, segredos ou upload de conteúdo;
- transitivos Linux inspecionados pelo `cargo tree`: `inotify`, `mio`, `libc`,
  `notify-types`, `walkdir`, `same-file` e `log`;
- versão fixada no manifest e no lockfile;
- testes: backend real, coalescência/filtros, conflito de save e harness QML.

O inventário detalhado vive em
`DocsPublic/integracoes/registro-de-componentes-abertos.json`.

## Consequências

Positivas: usa o backend correto de cada SO, não duplica um watcher, mantém o
core desacoplado de Qt e protege buffers sem indexar árvores gigantes.

Custos: há uma thread de debounce e watchers por diretório alcançado; limites
do SO podem falhar, caso em que a UI recebe `event.fs.watchError`. Mesmo nesse
caso, compare-before-save continua impedindo sobrescrita silenciosa.

## Alternativas rejeitadas

- implementar inotify diretamente: duplicação, portabilidade e manutenção;
- polling próprio recursivo: custo de I/O e latência em workspaces grandes;
- watcher recursivo de toda a raiz: simples, porém contrário ao orçamento de
  recursos e sujeito a ruído de `target`, `build`, caches e VCS;
- confiar só no watcher: mantém uma corrida entre evento e save;
- confiar só no compare-before-save: seguro, mas sem auto-reload/feedback
  imediato e sem atualização incremental da árvore/Git.
