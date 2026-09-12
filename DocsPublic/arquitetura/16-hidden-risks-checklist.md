# 16 — Riscos Ocultos e Checklist de Prontidão

## Objetivo

Lista enxuta de riscos que não são "features bonitas", mas que fazem uma IDE
parecer profissional — ou que quebram a confiança do usuário se ignorados.
Extraído e condensado da proposta de governança documental revisada em
2026-07-05 (ver `DocsPublic/arquitetura/15-engineering-debt-and-refactor.md` para o contexto da
limpeza de docs). Não é um plano de implementação por si só: é a lista que
qualquer plano de feature grande deve checar antes de ser considerado pronto.

## 1. Perda de trabalho do usuário

**Prioridade:** P0.

Riscos: crash, fechamento inesperado, queda de energia, arquivo alterado fora
da IDE, salvar por cima sem perceber, buffer não salvo perdido.

O que precisa existir: dirty state confiável, recovery buffer, sessão
anterior, detecção de alteração externa, salvamento atômico quando possível,
confirmação antes de sobrescrever conflito.

Regra: uma IDE pode faltar feature; ela não pode perder trabalho do usuário.

## 2. Migração de schemas e configs

**Prioridade:** P0/P1.

Arquivos afetados: `.kinein/workspace.json`, futuros `.kinein/toolchains.json`,
`.kinein/run-configs.json`, `.kinein/quality-profile.json`,
`.kinein/ui-state.json`, `CMakePresets.json`, `CMakeUserPresets.json`.

Cada config própria precisa de: `schemaVersion`, validação, migração, backup,
erro humano legível, reset seguro. Sem isso, versões futuras quebram projetos
antigos.

## 3. Arquivos gerados/alterados pela IDE

**Prioridade:** P0/P1.

Regra: a IDE não é dona do projeto do usuário. Toda alteração não explícita no
editor precisa de preview, diff, explicação, lista de arquivos afetados,
risco e confirmação — com rollback quando possível. Vale especialmente para
`CMakeLists.txt`, `CMakePresets.json`, `Cargo.toml`, `.clang-format`,
`.clang-tidy`, `README.md`, scripts e configs em `.kinein/`.

## 4. Segurança de comandos

**Prioridade:** P0/P1.

Todo comando/ação deve ter nível de risco: `safe`, `modifies_files`,
`runs_external_process`, `network`, `dangerous`, `forbidden_by_default`.
Exemplos: `lsp.hover`/`workspace.status` são `safe`; `fs.write`/
`configActions.apply`/`lsp.workspaceEdit` são `modifies_files`; `git reset
--hard`, `rm -rf`, flash de firmware, reboot de alvo remoto ou comando remoto
com sudo são `dangerous`. Comando perigoso nunca roda sem confirmação
explícita (ver `JobRisk` em `kinein-protocol`, hoje só classificado, ainda sem
enforcement geral — ver `DocsPublic/roadmaps/backend-para-ui-ux.md` P1 "Risk engine").

## 5. Segredos e logs

**Prioridade:** P0.

Nunca salvar segredo em `.kinein/*.json`, logs, histórico, crash reports,
prompt de IA ou stdout/stderr persistente sem máscara. Mascarar padrões como
`*_TOKEN`, `*_KEY`, `*_SECRET`, `PASSWORD`, `PASS`, `AUTH`, `COOKIE`,
`PRIVATE_KEY`. Log útil não pode virar vazamento de credencial.

## 6. Testes de regressão

**Prioridade:** P0/P1.

Não testar só "compila". Cobrir comportamento: IPC, workspace, filesystem
confinado, rename/delete, diagnostics, request/response de LSP, templates,
schemas, arquivos gerados, parsing de saída de ferramenta, migração de
config, segredos mascarados, comandos perigosos bloqueados. Refatoração só é
segura se o comportamento estiver coberto por teste antes de mover código.

## 7. Acessibilidade e telas pequenas

**Prioridade:** P2.

Não esquecer: resolução 1366x768, fonte configurável, contraste suficiente,
não depender só de cor, navegação por teclado, tooltips, redução de
animações, painel direito fechado por padrão em telas pequenas.

## 8. Observabilidade local

**Prioridade:** P1/P2.

A IDE deve explicar localmente, sem telemetria externa: core conectado, IPC
latency, LSP rodando, jobs ativos, memória aproximada, último erro, logs
locais, ferramenta ausente, processo travado, task cancelável, background
services.

## 9. Packaging, launcher e caminhos Linux

**Prioridade:** P1/P2.

Não esquecer: desktop entry, ícone, script launcher, config dir, cache dir,
data dir, logs dir, AppImage/PKGBUILD futuro, seleção release/debug, não
depender de caminho absoluto da máquina do autor.

## 10. Cópia cega, referência obsoleta e transplante arquitetural

**Prioridade:** P0/P1.

Estudar IDEs profissionais reduz erro de projeto somente quando a fonte é
oficial, mantida e pertinente. Copiar uma função pronta, traduzi-la
mecanicamente ou carregar o runtime/serviços internos da IDE de origem cria
incompatibilidade arquitetural, risco de licença, bugs de ciclo de vida e uma
manutenção que a Kinein não controla. Código antigo não vira boa referência só
porque resolve um caso parecido.

Aplicar a seção 2.1 de
`DocsPublic/roadmaps/adaptacao-de-plugins-abertos.md`: registrar revisão e licença,
ler também testes/falhas, extrair invariantes e implementar código novo nas
camadas nativas. Code OSS, IntelliJ IDEA Community, Zed, Lapce e Apache
NetBeans são referências obrigatórias conforme pertinência, nunca fornecedores
automáticos de código.

## Checklist rápido

```text
[ ] Recovery/autosave definido antes de dogfooding pesado.
[ ] Schema version em configs próprias.
[ ] Preview/diff obrigatório em alterações automáticas.
[ ] Comandos classificados por risco.
[ ] Secrets mascarados em logs.
[ ] Support bundle futuro não vaza credenciais.
[ ] Testes cobrem comportamento, não só compilação.
[ ] UI considera tela pequena e teclado.
[ ] Background Services observáveis.
[ ] Launcher/paths Linux documentados.
[ ] Feature de IDE registra fonte/revisão atual, invariantes e adaptação própria.
[ ] Nenhuma função pronta, tradução mecânica ou runtime de outra IDE foi copiado.
```
