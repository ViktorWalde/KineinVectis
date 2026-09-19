# 03 — O ritual de uma fatia

Uma **fatia** é a menor mudança que dá para desenhar, medir, provar e
commitar sozinha. O ritual abaixo é o mesmo para um bug de uma linha e
para uma funcionalidade nova — o que muda é quanto de cada passo é
necessário. A ordem **não** muda.

```text
0. LER       o estado (40 §cabeçalho e §7 recentes) e o roadmap da etapa
1. DESENHAR  no roadmap, ANTES do código: contrato, arquivos, medidas, o que fica fora
2. MEDIR     o antes (foto, número, teste que falha)
3. CONTRATO  kinein-protocol + PROTOCOL_VERSION + arquitetura/03 (se toca o IPC)
4. CORE      serviço puro → handler fino → testes de despacho
5. PONTE     ui/src/core_client_<dominio>.cpp: método + dispatch + sinal
6. ROUTER    ui/qml/ipc/<Dominio>{Request,Event}Router.qml
7. CONTROLLER e a view burra; harness em scripts/qml-harness
8. PROVAR    gates verdes; foto do depois; o número do depois
9. DOCUMENTAR arquitetura/03, manual, 40 §7.N, o roadmap §feito, o registro privado
10. COMMIT   um por fatia, com o que provou e o que NÃO fez
```

## 0. Ler o estado

`DocsPublic/roadmaps/40-estado-e-continuidade.md`: o cabeçalho tem os
números atuais (protocolo, métodos, eventos, testes, harnesses); a §4.2 é
o panorama; a §7 é o diário — leia as últimas entradas. O roadmap da
etapa em curso está em `roadmaps/README.md`. Se você vai mexer numa área,
`grep -n "<área>" DocsPublic/roadmaps/40-*.md` mostra o que já foi
tentado e por que foi feito assim.

## 1. Desenhar antes do código

No roadmap da etapa (ou numa seção `§N.1 desenho fino` dele), escreva:
**o que muda, arquivo a arquivo**; o contrato (método, params, resultado,
evento — com os nomes finais); **o que fica fora e por quê**; as medidas
que vão provar (foto, harness, número). Isso existe para que o contexto
sobreviva a um corte — de sessão, de agente, de pessoa. Um bom teste: se
você desaparecesse agora, outra pessoa conseguiria implementar só com o
que está escrito?

## 2. Medir o antes

Foto (`KINEIN_SCREENSHOT`), o número (tempo de primeiro frame, latência da
tecla, contagem), ou um teste que falha. "Estava ruim" não é medida.
Guarde em `DocsPrivate/Codex/evidencias-<data>-<tema>/`.

## 3. O contrato

Se a fatia toca o IPC:

1. o tipo em `crates/kinein-protocol/src/<dominio>.rs` (serde,
   `deny_unknown_fields` quando não há `flatten`), reexportado em
   `lib.rs`;
2. `PROTOCOL_VERSION` sobe (`0.129.0` → `0.130.0` para método/evento novo);
3. `DocsPublic/arquitetura/03-ipc-protocol.md`: a entrada no changelog do
   topo **e** a seção do domínio (a assinatura na lista de métodos e o
   texto);
4. o gate de fiação vai exigir que o método tenha handler no core, que a
   ponte o despache, que o sinal tenha um consumidor QML — **todas** as
   pontas, ou a fatia não fecha.

## 4. O core

- A lógica em `crates/kinein-core/src/<dominio>/` como função **pura**
  quando possível (recebe dados, devolve um plano/resultado — testável sem
  processo); o efeito (rodar o programa, escrever o arquivo) separado.
- O handler em `handlers/<dominio>.rs` é **fino**: valida params, chama o
  serviço, formata a resposta. Handler que decide é handler errado.
- O que demora **não segura o laço**: `defer_work` (thread + resposta),
  `defer_then` (espera fora, termina dentro com `&mut Core`), ou um job
  (`jobs.spawn`, com `JobRisk`, cancelável, com `event.<dominio>.<x>` no
  fim). A regra e as três formas: `arquitetura/04-boot-e-comunicacao.md` §6.
- Testes em `crates/kinein-core/src/tests/<dominio>.rs`: o despacho
  completo (`handle_request` com JSON), ferramentas **falsas** em
  `scripts/fake_*.py` quando a real não é determinística, e o helper
  `tests/mod.rs` (`terminal_run_until_closed`, `render_lines`…).
- Limites: 500 linhas fora de `#[cfg(test)]`; clippy pedante
  (`--all-targets`, incluindo doc-comments com crase em nomes).

## 5. A ponte C++

`ui/src/core_client_<dominio>.cpp`: o `Q_INVOKABLE` que monta o pedido, a
entrada no `dispatch<Dominio>Result` que reconhece o método e emite o
sinal tipado, e — para eventos — `core_client_notifications.cpp`. A
assinatura do sinal vai em `core_client.h`. O gate de fiação confere que
o `dispatch*` é chamado na cadeia e que o sinal tem quem ouça. Limite:
500 linhas por arquivo.

## 6. Os roteadores QML

`ui/qml/ipc/<Dominio>RequestRouter.qml` liga sinais do controller aos
métodos da ponte; `<Dominio>EventRouter.qml` liga sinais da ponte aos
`handle*` do controller. Só fiação — nenhuma lógica. Quando o dono é um
**filho** do controller (`controller.identity`, `controller.symbols`), o
`target` do `Connections` é o filho, não o pai (o erro do `40` §7.72).

## 7. Controller e view

- **Controller** (`ui/qml/<dominio>/<X>Controller.qml`): estado e
  derivações; pede por `signal`, recebe por `function handle*`. Sem
  Theme, sem geometria — para o harness carregar sem tela. ≤ 400 linhas;
  se não cabe, um filho com responsabilidade própria (não um "Part2").
- **View** (`<X>Panel.qml`, `<X>List.qml`…): lê propriedades, emite
  gestos. Usa as peças comuns (`KvPanelFrame`, `KvPanelHeader`,
  `KvVerdict`, `KvDataGrid`, `KvToggleChip`, `KvButton`). ≤ 300 linhas.
  Toda âncora com margem tem a âncora; um binding não pode ficar mais
  indentado que a propriedade (o gate de propriedades acusa os dois).
- **Host** (`ui/qml/shell/*Host.qml`, `app/AppDomains.qml`): só
  composição. `AppDomains` está em 399/400 — não entra linha ali sem sair
  outra.
- **Harness** `scripts/qml-harness/tst_<x>.qml`: carrega o controller
  real, afirma com bitmask, `Qt.exit(failures === 0 ? 0 : 1)` (o bitmask
  vai para o `console.error` porque o exit code tem 8 bits).
- Registrar componentes novos em `ui/CMakeLists.txt` (as duas listas).
- Se a fatia tem atalho: o comando no catálogo do core
  (`commands/*.rs`, `default_shortcut`) **e** o `Shortcut` anotado
  `// comando: <id>` em `GlobalShortcuts.qml`; o item de menu em
  `AppMenuItems.qml` **e** o `case` no `ShellHeaderHost`.

## 8. Provar

`bash scripts/verificar.sh --rapido` verde; os gates QML e o
`verificar-binario-abre.sh --preset linux-clang-debug-strict` (sem aviso
QML no stderr); a **foto do depois** com o mesmo enquadramento do antes; o
número do depois. Se algo só uma pessoa na frente da IDE consegue medir
(o clique, a placa, um Grafana real), **diga isso** — não afirme.

## 9. Documentar

- `arquitetura/03` (se contrato); `manual.md` (se o usuário vê);
- `roadmaps/40` §7.N: o que, por quê, como provou, o que **não** fez e
  por quê; os números do cabeçalho se mudaram;
- o roadmap da etapa: a fatia no "feito";
- o registro privado `DocsPrivate/Codex/<data>-<tema>.md` (tabela
  fatia × commit × público; provas; notas de método; o que ficou) e o
  `PROMPT-proxima-sessao.md` apontando o próximo passo;
- `bash scripts/verificar-docs.sh` e `verificar-links-docs.sh` verdes.

## 10. Commit

Um por fatia. A mensagem diz o que mudou **e o que provou** e **o que não
fez**. Nada de push, release ou AppImage sem o mantenedor pedir. Se a
árvore tem duas fatias misturadas, separe o índice (`git add` por arquivo;
para arquivo compartilhado, `git hash-object -w` + `git update-index
--cacheinfo` de uma versão parcial) — nunca `git checkout` do arquivo.

## A fatia mínima (um bug de uma linha)

0 (ler o `40` da área) → 2 (o teste ou harness que falha) → 4/7 (a
correção) → 8 (gates) → 9 (uma linha no `40` §7.N e no registro) → 10.
Mesmo o bug de uma linha ganha um teste: foi assim que os gates
cresceram, e é assim que o bug não volta.
