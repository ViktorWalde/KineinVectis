# Quatro tarefas iniciais da trilha de Embarcados 0.4

> **Classe: PLANO.** Escopos para contribuições à trilha aberta do
> [52](../roadmaps/52-arquitetura-executavel-da-0.4.md). Nenhuma tarefa está
> concluída por constar nesta lista. Fronteira da base pública no
> [62](../roadmaps/62-base-publica-0.4.md).

Cada contribuição fecha core, ponte e UI da mesma jornada, com gate próprio.
Reusar os donos existentes; contrato IPC antes do produtor quando necessário.
As fixtures em `scripts/fixtures/projetos/` são sementes de detecção, não
projetos completos nem provas de build. Ampliá-las com origem/licença e
versões fixadas. Resultados distinguem implementado, integrado, validado em
software/emulador e validado em hardware. Publicar somente evidência técnica.

## BM1 — west nativo para ESP32

**Situação:** desenho no [52 §14.56](../roadmaps/52-arquitetura-executavel-da-0.4.md#1456-041h4b2b4p--west-nativo-runner-esp32-desenho-2026-10-27),
produtor nativo ausente na auditoria de 2026-11-02. O fluxo legado não
satisfaz o contrato nativo planejado.

- **Fixture:** ampliar `scripts/fixtures/projetos/zephyr/` com aplicação
  mínima ESP32, manifesto plano, build existente e imagens identificadas.
- **Dependências:** west 1.5.0, Zephyr 4.2.1, toolchain/SDK e esptool
  compatíveis; registrar versões e hashes de artefatos de teste.
- **Escopo:** prévia → intenção tipada → reserva de porta → execução/cancelamento
  pelo dono nativo. Recusar runner externo, sysbuild, efeito destrutivo não
  previsto, caminho fora do build e artefato alterado após prévia.
- **Aceite em software:** testes de parsing/identidade, CLI falsa para
  cancelamento e falhas; prévia com build real sem executar código do projeto;
  UI mostra runner, imagens, offsets, porta, versão e erro recuperável.
  Roteiro e comandos reproduzíveis acompanham o PR.
- **Aceite físico separado:** ESP32, porta serial explícita e confirmação
  da gravação; registrar modelo/chip e resultado sem identificadores únicos.
  CLI falsa ou emulador não validam gravação/reset físicos.
- **Fora do escopo:** todos os runners Zephyr, sysbuild e provisionamento de SDK.

## BM2 — Ciclo RP2040 com CMake/pico-sdk e picotool

**Situação:** consulta e UF2/ELF nativos implementados e provados em software
no [52 §14.55](../roadmaps/52-arquitetura-executavel-da-0.4.md#1455-041h4b2b4o2--uf2build-e-picotool-nativo-2026-10-27).
Conferir o título/escopo desse registro antes de reutilizar as provas.

- **Fixture:** ampliar `scripts/fixtures/projetos/pico-sdk/` com aplicação
  mínima que gere UF2/ELF, pico-sdk fixado e alvo CMake identificável.
- **Dependências:** CMake, Ninja, compilador ARM e picotool compatíveis,
  RP2040 com acesso BOOTSEL; SWD é uma capacidade distinta.
- **Escopo:** configurar/compilar → escolher artefato → prévia → gravar →
  recuperar desconexão/reconexão no painel existente. Corrigir lacunas
  encontradas sem substituir o produtor já implementado.
- **Aceite em software:** UF2/ELF do alvo correto; recusar arquivo/board
  incompatível e artefato obsoleto; cancelamento/erro liberam recursos;
  teste mutante que troca artefato ou identidade deve reprovar.
- **Aceite físico separado:** enumerar em BOOTSEL, gravar e verificar o
  comportamento da aplicação, desconectar durante operação e recuperar.
  Provar que outra placa não é selecionada silenciosamente.
- **Fora do escopo:** RP2350, promessa de USB serial automático e SWD implícito.

## LE1 — Kit Yocto e aplicação Linux por SSH

**Situação:** importação de SDK/kit, Remote e RunConfig têm donos; a jornada
integrada precisa de prova própria. Compilar para host não prova cross.

- **Fixture:** ampliar `scripts/fixtures/projetos/yocto/` com aplicação
  CMake C/C++, SDK instalado e configuração de alvo Linux ARM64 por SSH.
  Não usar credenciais reais na fixture versionada.
- **Dependências:** SDK Yocto identificado, CMake/Ninja, OpenSSH e alvo
  ARM64 ou VM QEMU acessível por SSH. Imagem/SDK prontos evitam build completo
  do Yocto para testar a IDE; registrar origem, licença e versão.
- **Escopo:** importar `environment-setup-*` → conferir sysroot/compilador
  efetivos → CDB/LSP → cross-build → envio/execução via Remote existente.
- **Aceite em software/VM:** ELF ARM64 comprovado, execução e exit code
  observados, interrupção e reconexão recuperáveis; SDK ausente/incompatível
  explicado. Trocar kit não pode conservar flags/sysroot do anterior.
  Provar edição/salvamento e navegação no projeto da fixture.
- **Aceite físico opcional e separado:** repetir em alvo ARM64 real, sem
  transformar o resultado de uma placa em suporte universal Yocto.
- **Fora do escopo:** gerar distribuição Yocto completa, provisionar rede e
  criar runtime novo para deployment.

## LE2 — Buildroot em QEMU e depuração Linux

**Situação:** QEMU, kits, RunConfig e DAP existentes são os pontos de partida;
não há aceite implícito da jornada completa.

- **Fixture:** adicionar aplicação C com falha controlada e configuração
  Buildroot mínima para QEMU ARM64; fixar release/configuração, origem da
  imagem e toolchain, hash e símbolos de debug. Reusar fixtures de processos
  para falhas; não incluir imagem binária no Git.
- **Dependências:** Buildroot/toolchain compatíveis, QEMU, GDB com DAP e
  gdbserver compatíveis. Build completo de imagem é uma carga própria;
  documentar tempo, RAM e disco, além do caminho com imagem pré-construída.
- **Escopo:** importar kit → compilar aplicação → iniciar alvo QEMU →
  conectar/depurar pelo DAP existente, com contexto e símbolos corretos.
- **Aceite em software/VM:** parar em breakpoint, inspecionar variável,
  continuar, terminar e repetir sem processo órfão; porta ocupada, alvo
  indisponível e incompatibilidade do debugger mostram recuperação.
  Mutações em sysroot/símbolos não podem produzir sucesso aparente.
- **Hardware:** não exigido por esta tarefa. QEMU valida a jornada Linux
  emulada, sem comprovar periféricos, boot ou comportamento de placa real.
- **Fora do escopo:** suporte genérico a todo board Buildroot e simulador próprio.

## Entrega de cada tarefa

PR com fixture reproduzível, limites de suporte, prova da jornada e das
falhas, versões das ferramentas, gates e documentação dos donos atualizada.
Evitar exigir prova física para aceitar uma fatia explicitamente de software;
manter o resultado físico pendente e identificado. As tarefas permanecem
neste checkout até sua publicação pelos canais do projeto.
