# PONTO ATUAL — fila viva do Kinein Vectis (2026-07-15)

> Este arquivo contém somente trabalho presente ou futuro, em ordem de
> execução. Trabalho concluído deve ser registrado no documento do domínio e
> em `ContextoIA.md`, e então removido daqui.
>
> Estado implementado: `ContextoIA.md` + docs numerados + código. Mapa de
> conhecimento e arquivos conectados: `GUIAIA.md`. Histórico de checkpoints:
> Git. Base remota atual: `928fbb5`, protocolo `0.55.0`.
>
> Não alterar a UI fora das specs. Commits locais de checkpoint após marco
> crítico/teste verde foram autorizados em 2026-07-15; push e publicação não
> foram. **Dogfooding/self-hosting ativo desde 2026-07-14:** o usuário já
> está na Kinein, e cada bloqueio ou saída para outra IDE passa a ordenar o
> backlog antes de funcionalidade nova.

## 0. Dogfooding ativo

O gatilho **“estou no Kinein”** já foi recebido. A primeira regressão concreta
é o KV Context não se comportar visualmente como um terminal profissional:
faltavam scrollback/barra perceptível no Codex, largura adequada e fluidez no
resize. Após o primeiro reinício, surgiu um segundo detalhe: a linha de
digitação inline ficava sem limite visual entre o aviso de usage e o status do
modelo. No segundo reinício foram reportados quatro sintomas adicionais: a
faixa não envolvia os glifos, o divisor livre desaparecia durante a sessão, a
árvore `Project` era fechada e o scroll se perdia em chats longos. A correção
0.52 tratou esses quatro sem criar input ou terminal paralelo. Após o aceite do
dimensionamento, o feedback restante ficou restrito à faixa da entrada
multilinha e à roda do mouse; a correção pós-0.52 de 2026-07-15 aguarda o gesto
real apenas nesses dois pontos, conforme `REENTRADA-KV`. O usuário pausou esse
gesto e autorizou expressamente a continuação do roadmap. A1 e A2 foram
entregues nos protocolos 0.53.0 e 0.55.0. A tentativa posterior de demarcar a
entrada foi removida: grade VT, spans e cursor voltaram a ser a única fonte
visual, no modelo terminal-first.

1. registrar cada problema observado pelo usuário ou por um testador com ação,
   esperado, resultado, reprodução, distro e log quando houver;
2. corrigir primeiro perda de dados, crash, corrupção, falha de abertura/build
   ou bloqueio que force a saída para outra IDE;
3. depois tratar regressões funcionais e atritos reproduzíveis de uso diário;
4. quando não houver feedback bloqueador e o usuário mandar prosseguir no
   roadmap, iniciar **A3 — responsividade medida**; A1 e A2 foram entregues;
5. continuar pelas etapas deste arquivo e pelos docs existentes, sem inventar
   outro remake, subsistema paralelo ou roadmap substituto.

O dogfooding não autoriza push, publicação, mudança de visibilidade, envio
externo ou implementação aleatória fora da fila. Commits locais são feitos
somente nos checkpoints verdes já autorizados. O repositório-fonte continua
privado.

### 0.1 Protocolo econômico de reentrada após reiniciar a própria Kinein

Reiniciar a IDE encerra a sessão de IA que está rodando dentro dela. Para não
gastar outra conversa reconstruindo contexto, o handoff deve ficar neste
arquivo antes de fechar. Na sessão nova, o usuário precisa escrever somente:

```text
Leia AGENTS.md e retome pelo marcador REENTRADA-KV em PONTO_ATUAL.md. Siga as
leituras obrigatórias silenciosamente, não resuma o histórico e não refaça o
que já está validado.
```

A sessão nova deve executar esta sequência:

1. ler `AGENTS.md` e a documentação obrigatória indicada nele, sem devolver
   uma recapitulação extensa ao usuário;
2. localizar primeiro o marcador `REENTRADA-KV` abaixo e confrontá-lo com
   `ContextoIA.md` + código apenas onde houver divergência;
3. rodar `git status --short` para preservar o worktree existente; arquivos
   presentes não autorizam limpeza/descarte. Commit local só depois do gate
   verde do marco; push continua sem autorização;
4. não repetir investigação, implementação, gate ou build já registrados como
   verdes, a menos que o código tenha mudado depois do marcador ou apareça
   evidência concreta de regressão;
5. retomar diretamente pela ação `PRÓXIMO GESTO`; responder inicialmente com
   uma frase curta de orientação, não com outro plano ou resumo;
6. depois do gesto real, registrar o resultado no marcador: se falhar,
   ação/esperado/observado/ambiente viram a prioridade do dogfooding; se passar,
   marcar o aceite e seguir a próxima fatia desta fila somente quando o usuário
   mandar prosseguir.

#### REENTRADA-KV — estado exato para a próxima reentrada

```text
ESTADO
- Protocolo atual 0.55.0. A1 (recentes) e A2 (capacidades Cargo+CMake) estão
  implementadas; `workspace.kind` é primário compatível e
  `workspace.capabilities.buildSystems` é a fonte das ações híbridas.
- Codex abre com argumento fixo --no-alt-screen; Claude permanece sem argumento.
- KV ativo reutiliza TerminalPanel/TerminalManager, tem barra persistente,
  roda/arrasto, teclado/paste VT, largura livre persistida 300–720px e
  ampliar/restaurar; Project permanece independente fora da maximização.
- O bridge preserva o transcript contra CSI 3 J ainda emitido por versões do
  Codex; o Terminal comum continua honrando clear.
- Não existe guia, faixa ou input paralelo: grade VT, spans ANSI e cursor são
  a única representação da entrada, seguindo comportamento terminal-first.
- A roda aceita os dois formatos do Qt (`angleDelta` e `pixelDelta`) e segue o
  mesmo `terminal.scroll` do Terminal comum.
- Gutter usa faixas independentes para folding/breakpoint, diagnóstico, blame,
  diff e números medidos por FontMetrics; marcador não invade o número.
- Semantic tokens carregam path+version e respostas obsoletas são descartadas;
  Tree-sitter permanece fallback estrutural instantâneo.
- Os cinco SVGs de árvore fornecidos foram integrados sem alterar seus bytes.
- Scripts shell reconhecidos têm ação de execução na árvore; o core confina o
  caminho e usa argv explícito, sem interpolação.
- Packaging corrigido para cache host/container isolado, mounts Podman/SELinux
  e invocação por bash. `dist/` deve receber AppImage, checksum específico,
  instalador e Tutorial.md vigente.

VALIDAÇÃO JÁ FEITA — NÃO REPETIR SEM MUDANÇA DE CÓDIGO
- `scripts/verificar.sh` completo: verde; binários release do atalho de
  desenvolvimento atualizados.
- Testes Rust, clippy -D warnings, C++/QML estritos e harnesses QML: verdes.
- Build Clang debug strict e `scripts/verificar-cpp.sh`: verdes.
- `scripts/verificar-qml.sh`: verde usando o response file do build strict
  atualizado; o antigo `build/dev-local` não tem precedência.
- tst_assistant_layout cobre divisor ativo/largura/Project; tst_terminal_scroll
  cobre nova saída, snap, troca de sessão, roda tradicional e `pixelDelta`;
  Rust cobre CSI 3 J entre chunks.
- AppImage final (33.737.208 bytes; SHA256 `fd5fe934599757b6980703d2c2529f9b50bdc026e03e5da34b6a0eb5f44629b9`),
  teste host e Debian mínimo sem rede: verdes. Ambos validam também instalador
  executado fora da pasta, `.desktop`, PNG e `Tutorial.md` idêntico à fonte.

PRÓXIMO GESTO
1. Criar o checkpoint Git local desta estabilização já verde.
2. Abrir a Kinein pelo AppImage novo em `dist/` e carregar este repositório.
3. Confirmar as ações Cargo e CMake, um script pela árvore, os ícones exatos e
   breakpoint/diagnóstico em arquivo com numeração larga.
4. No KV Context, gerar saída maior que a altura do painel; rolar enquanto a
   resposta ainda chega e confirmar que a leitura não salta nem perde o
   histórico. Confirmar que não há moldura/input desenhado pela IDE.
5. Depois do gesto, seguir A3 — responsividade medida.

RESULTADO PENDENTE
- Aceite humano dos fluxos acima usando o AppImage final.
- Se qualquer gesto falhar, registrar ação/esperado/observado/ambiente e
  priorizar a regressão antes de A3.

LIMITES
- Commit local somente após checkpoint verde; não fazer push/publicação.
- Não reabrir a discussão de chat embutido: KV Context é terminal dedicado.
```

### Loop de desenvolvimento a partir do dogfooding

```text
feedback real (autor ou testador)
        ↓
reprodução e causa-raiz
        ↓
teste/harness de regressão quando aplicável
        ↓
correção pequena na camada dona
        ↓
gate + gesto real
        ↓
docs sincronizadas e retorno ao uso
```

Se vários feedbacks chegarem juntos, usar esta prioridade:

```text
P0  perda/corrupção de dados, segurança, crash ou IDE não abre
P1  bloqueio de edição, build, run, debug, terminal, Git ou navegação
P2  comportamento incorreto/repetível que prejudica o fluxo diário
P3  conforto, polimento ou funcionalidade nova
```

Feedback de testador não vira feature automaticamente: reproduzir, conferir se
já existe solução no core/UI e encaixar no domínio/roadmap correto. Se for uma
ideia nova sem bloqueio, registrar atrás dos problemas reais e de A3.

## 1. TR0 — aceite funcional da rodada atual

Antes de ampliar o produto, validar a aplicação real em tela. Regressões
encontradas aqui têm prioridade e não autorizam outro remake visual.

### 1.1 Checklist de validação manual

- **Autocomplete:** a primeira sugestão estrutural aparece sem esperar o LSP;
  a resposta semântica de clangd/rust-analyzer substitui o fallback quando
  estiver pronta, sem popup duplicado ou salto de seleção.
- **Terminal sob rajada:** saída contínua acompanha a linha atual sem atrasos;
  a barra aparece assim que existe histórico; rolar ou arrastar preserva a
  leitura; nova entrada do usuário volta ao final; múltiplas abas permanecem
  independentes.
- **Menus Arquivo–Ajuda:** todas as opções ficam acima do editor, legíveis e
  acionáveis. `Ajuda → Manual da IDE` abre a documentação interna.
- **Criação no projeto:** menu Arquivo e clique direito oferecem adicionar
  arquivo/pasta e reutilizam o fluxo confinado ao workspace.
- **KV Context:** Claude/Codex instalados pelo usuário são descobertos; a
  escolha abre uma sessão própria sobre o terminal real; Codex preserva
  scrollback em modo inline; barra/roda/arrasto, teclado, seleção, copiar/colar
  e resize se comportam como no Terminal integrado, inclusive durante nova
  saída; a sessão tem largura livre/persistida, coexiste com `Project` e pode
  ser ampliada sem acoplar o painel ao terminal comum.
- **Estrutura e painéis:** a aba Estrutura redimensiona, recolhe e restaura;
  o layout inicial se adapta à janela sem cobrir editor ou menus.
- **Barras e tooltips:** editor, terminal e listas longas mostram posição e
  permitem arrastar; descrições de ícones nunca aparecem sob o editor.

### 1.2 Critério de saída do TR0

- checklist acima aceito em uma sessão real;
- qualquer falha reproduzível ganhou teste/harness quando aplicável;
- `bash scripts/verificar.sh` verde;
- aceite visual R7/C6 confrontado com as specs, sem mudança estética lateral.

O início do dogfooding não precisa esperar uma cerimônia separada de TR0: a
primeira sessão dentro da Kinein deve percorrer este checklist naturalmente.
Falhas encontradas nela interrompem a próxima fatia do roadmap até a regressão
correspondente ficar corrigida e protegida.

## 2. TR1 — distribuição e substituição de editores generalistas

### A3 — responsividade medida

- medir tempo de abertura, primeira estrutura Tree-sitter, primeira sugestão
  semântica, latência de digitação, estabilização do LSP, rajada do terminal e
  memória em projetos reais;
- estabelecer orçamento local reproduzível, sem telemetria;
- mover indexação, parse e I/O pesado para trabalho cancelável/assíncrono;
- priorizar regressões perceptíveis antes de aumentar profundidade semântica.

Aceite: métricas e cenários ficam versionados; não se depende apenas de
impressão visual para afirmar que autocomplete/editor/terminal são responsivos.

### A4 — confortos que bloquearem o dogfooding

Prioridade inicial, ajustada pelos motivos reais de saída para outro editor:

1. split editor;
2. multicursor;
3. EditorConfig;
4. zoom do editor;
5. links e busca no scrollback do terminal;
6. duplo clique para selecionar palavra no terminal.

Aceite do TR1: uma semana de desenvolvimento C/C++ e Rust sem abrir editor
generalista auxiliar. Quando o usuário disser **“estou no Kinein”**, registrar
cada exceção e corrigir primeiro o bloqueio reproduzível. Feedback dos
testadores entra no mesmo funil, identificado pela origem e pelo ambiente, sem
substituir evidência de reprodução.

## 3. Consolidações necessárias antes de declarar substituição diária

| Frente | Consolidação pendente | Evidência de aceite |
| --- | --- | --- |
| Distribuição | ampliar matriz Ubuntu/Fedora, canal de release, atualização e diagnóstico de runtime | AppImage validado em distros-alvo e procedimento de release repetível |
| Entrada no trabalho | validar workspaces recentes e restauração previsível em uso prolongado | retomar projeto em um clique, sem sessão cruzada |
| Modelo de projeto | aprofundar capacidades já detectadas em Project Graph/Context Matrix | targets e contextos Cargo/CMake/Qt explicáveis por arquivo |
| Editor | resposta imediata local + semântica progressiva | cenários e latências medidos |
| Terminal | rajadas, scrollback e múltiplas sessões prolongadas | teste de estresse sem atraso ou perda de interação |
| Build/Run/Test | fluxos reais e cancelamento em projetos externos | processos encerram sem órfãos e resultados são navegáveis |
| Debug | sessão básica confiável e inspeção útil | breakpoint, step, pilha e variáveis em fixture real |
| Segurança de dados | soak tests de save, mudança externa, crash e drafts | nenhuma escrita silenciosa sobre snapshot antigo |
| Ergonomia | confortos puxados pelo dogfooding | nenhum editor auxiliar necessário por lacuna diária |

## 4. TR2 — primeiro patamar “um nível abaixo do CLion”

Implementar o KSWE em fatias pequenas, mantendo CMake/Cargo/clangd/
rust-analyzer como fontes autoritativas e sem criar uma engine genérica
paralela.

### B1 — Project Model autoritativo

- ampliar CMake File API para codemodel, targets, configurações, sources,
  compile groups, includes, defines e artefatos;
- consumir Cargo Metadata para packages, targets, features e workspace;
- normalizar ambos em snapshot versionado de Project Graph + Context Matrix;
- atualizar por geração, descartar resultado obsoleto e respeitar orçamento;
- criar fixtures CMake, Cargo e híbrida.

### B2 — targets, perfis e toolchains como entidades

- seleção explícita de target/configuração/toolchain;
- contexto efetivo por arquivo e target;
- presets CMake, perfis Cargo, sysroot e compile database rastreáveis;
- interface visual fiel às Configuration Actions das specs.

### B3 — inteligência semântica coordenada

- scheduler LSP com prioridade ao arquivo visível e cancelamento;
- Symbol Broker e Diagnostic Broker sem duplicar clangd/rust-analyzer;
- Effective Compile Context explicável ao usuário;
- caches limitados e invalidação por geração/fingerprint;
- fallback Tree-sitter continua instantâneo e independente do LSP.

### B4 — debug de IDE

- watches/expressões, variáveis, pilha, breakpoints condicionais e
  pretty-printers;
- GDB/LLDB via DAP, sem UI chamar debugger diretamente;
- sessões reais C/C++ e Rust com encerramento/cancelamento confiável.

Aceite do TR2: desenvolver a Kinein por uma semana sem abrir CLion por falta de
compreensão do projeto, build, navegação semântica ou debug básico.

## 5. TR3 — profundidade equiparável e embarcados

- múltiplos targets/contextos concorrentes e cache semântico persistente;
- correlação de símbolos, diagnósticos e refatorações mais profundas;
- toolchains cruzadas, sysroots e SDKs Yocto/Buildroot;
- flash, serial, QEMU, OpenOCD/pyOCD e GDB remoto;
- testes prolongados em projetos C, C++, Rust e embarcados reais;
- otimização de CPU/RAM/latência sem transformar indexação em trabalho eager.

## 6. Backlog complementar, depois das consolidações imediatas

- opção guiada **Outra IA**: abrir uma sessão de terminal dedicada e instruir
  o usuário a digitar o comando de inicialização da CLI que já instalou;
- biblioteca de funções / Configuration Actions para configurar CMake/Cargo e
  ambiente com preview, evidência e controle do usuário;
- exportador allowlist para qualquer cópia do código entregue a terceiros, com
  `--dry-run`, auditoria de segredos e recusa de Markdown além de `README.md`,
  `MANUAL.md` e `Tutorial.md`; sem `.git/`/histórico privado e com o
  repositório-fonte permanecendo privado;
- Windows é uma frente futura separada; não diluir o objetivo Linux-first atual.

## 7. Gate e definição de pronto de cada fatia

1. Ler `GUIAIA.md` e as fontes do domínio antes de editar.
2. Manter UI → CoreClient → protocolo → handler → serviço; UI não chama
   ferramenta externa ou filesystem de workspace diretamente.
3. Operação longa vira Job cancelável e nunca bloqueia a UI.
4. Criar teste de core e harness QML quando houver estado visual.
5. Executar `bash scripts/verificar.sh` e a sonda específica do domínio.
6. Para UI/layout, validar o gesto em tela real; para packaging, executar
   `scripts/testar-appimage.sh` e `scripts/testar-appimage-portatil.sh`.
7. Atualizar contrato, schema, manual e arquitetura quando afetados.
8. Registrar conclusão em `ContextoIA.md` e no doc do domínio; remover o item
   concluído deste arquivo, sem manter listas riscadas ou post-mortems aqui.

## 8. Onde ficou o histórico concluído

Este é apenas um mapa para evitar duplicação:

- estado técnico, decisões e checkpoints: `ContextoIA.md`;
- rede de segurança, drafts e escrita atômica: `docs/23-rede-de-seguranca.md`;
- autocomplete, terminal, paridade diária e UI: `docs/24-paridade-e-fundacao.md`;
- Tree-sitter e workspace edits: `docs/25-syntax-tree-semantic-foundation.md`;
- plano de daily driver e longo prazo: `docs/18-daily-driver-plan.md` e
  `docs/21-long-horizon-roadmap.md`;
- apresentação pública: `README.md`; uso da IDE: `MANUAL.md`; distribuição e
  instalação: `Tutorial.md`;
- alterações exatas e checkpoints: histórico Git.
