# Remote SSH — UI/HUD para uso diário

> **Classe: ALVO / PLANO.** Desenho escrito em 2026-09-22 depois de medir a
> implementação atual. O código vence qualquer inventário deste documento.
>
> Esta frente não substitui o domínio `remote.*`. Ela reforma a maneira como as
> capacidades existentes são apresentadas e identifica os contratos adicionais
> necessários para uso seguro e cotidiano.

## 1. Decisão de produto

Remote SSH é um contexto de trabalho de primeira classe para Linux embarcado e
edge, não uma caixa de configuração escondida em **Ambiente**.

O usuário precisa responder continuamente, sem reabrir um formulário:

```text
onde estou trabalhando?
qual alvo está ativo?
o que é local e o que é remoto?
o arquivo salvo chegou ao alvo?
há operação de sync/deploy em curso?
o alvo está alcançável?
qual programa e qual configuração serão executados?
```

O fluxo continua baseado no `ssh`, `rsync` e `scp` do sistema, executados como
processos. A Vectis não instala agente no alvo, não guarda senha e não cria um
segundo terminal, build system ou debugger.

**Régua de produto:** para o fluxo suportado, a UI precisa ser mais prática que
abrir o terminal e montar `ssh`/`rsync` manualmente. Se o segundo uso do mesmo
alvo ainda exigir percorrer campos de perfil, interpretar comandos brutos ou
procurar o estado do sync, a reformulação falhou mesmo que todas as operações
funcionem. A ergonomia do terminal reutilizado por Shell/Remote está em
[`terminal-ergonomia-0.3.md`](terminal-ergonomia-0.3.md).

Essa régua vale também para a configuração inicial. Um iniciante não deve
precisar saber de antemão a diferença entre alias, agente, chave privada,
`ssh-copy-id`, espelho, deploy e gdbserver para abrir uma pasta. Se `ssh pi`
já funciona no terminal, escolher **pi** na IDE precisa ser suficiente; se não
funciona, a IDE guia a preparação sem guardar senha nem esconder comandos.

## 2. Base implementada que deve ser preservada

O domínio atual já entrega:

- catálogo por workspace de `RemoteTarget` sem segredo;
- host, usuário, porta, chave e pasta de deploy;
- sonda de arquitetura, kernel, `gdbserver`, `python3`, `rsync` e `debugpy`;
- deploy por job, com `rsync` e fallback para `scp`;
- composição de comandos para Run, gdbserver, debugpy e shell;
- integração com configurações de execução e kit;
- terminal SSH reutilizando o terminal da IDE;
- abertura de pasta remota como espelho local;
- pull/push manual do espelho;
- push automático do arquivo salvo;
- Jobs e eventos de sonda, deploy e sincronização;
- marcador do espelho e `remote.status`.

Nada disso será reimplementado na UI.

## 3. Diagnóstico da UX atual

Hoje `RemotePanelHost` abre uma moldura de 680×520. Dentro dela:

```text
lista de alvos
veredito da sonda
seis campos de perfil
dois campos operacionais
abrir espelho
pull/push
deploy
criar configuração Run
gravar gdbserver no kit
criar configuração debugpy
abrir shell
salvar/remover perfil
comando bruto e erros
```

Os problemas de uso diário são estruturais:

1. configuração rara e operação frequente disputam a mesma coluna;
2. todas as ações aparecem com peso parecido;
3. o usuário precisa entender “config”, “kit”, “espelho”, “deploy” e
   “gdbserver” antes de executar o fluxo básico;
4. o painel mostra o último resultado, mas não oferece uma HUD persistente do
   contexto remoto atual;
5. “sondado uma vez” pode parecer “conectado agora”;
6. sync do arquivo salvo é pouco visível fora do painel;
7. o risco de edição nos dois lados é descrito em texto, não modelado como
   estado/confirmação;
8. o caminho feliz não forma uma sequência visual clara;
9. criar configuração de execução e configurar kit parecem tarefas cotidianas,
   embora sejam setup;
10. Remote fica escondido como painel de ambiente mesmo quando o workspace
    aberto é um espelho.
11. o backend já deixa `host` ser um alias e delega usuário/porta/chave ao
    OpenSSH, mas a UI não lista os aliases de `~/.ssh/config`;
12. não há caminho separado para **usar SSH existente** e **configurar um
    servidor novo**;
13. abrir espelho exige digitar a pasta remota, sem início pela home nem
    seletor de diretório;
14. a mensagem “use `ssh-copy-id`” transfere o problema ao terminal sem guiar
    quando a autenticação por chave ainda não está pronta.

## 4. Princípios da reformulação

- separar **usar** de **configurar**;
- mostrar primeiro contexto, saúde e próximo gesto;
- uma ação primária por estado;
- conservar comandos e detalhes técnicos, mas por progressive disclosure;
- integrar Run/Debug/Terminal/Jobs existentes em vez de duplicá-los;
- não chamar de “conectado” sem sessão/conexão viva que prove isso;
- toda afirmação de saúde mostra origem e idade da medição;
- sync e deploy são jobs visíveis na HUD e na aba Jobs;
- conflitos e operações destrutivas nunca são escondidos por automação;
- teclado e paleta alcançam os mesmos gestos da tool window.

## 5. Arquitetura de superfícies

### 5.1 Tool window Remote

Remote ganha uma tool window diária. Posição inicial proposta: **esquerda**, ao
lado de Projeto/Git/Embarcados, porque troca o contexto de onde o projeto é
operado. Essa posição é alvo inicial, não decisão irreversível; mover para a
direita exige teste de uso, não preferência abstrata.

Ela fica disponível quando houver ao menos um perfil remoto ou quando o
workspace aberto for um espelho. O menu Ambiente e a paleta continuam como
entradas equivalentes.

Estrutura:

```text
Remote
├── alvo/contexto ativo
├── Overview
├── Workspace
├── Run & Debug
├── Sistema
└── Configurar alvo
```

As seções podem ser abas compactas ou navegação interna. Não devem ser cinco
cards longos na mesma rolagem.

### 5.2 HUD na status bar

Quando o workspace é remoto/espelhado, a status bar mostra um item compacto:

```text
SSH · pi@192.168.1.42 · sincronizado
SSH · pi · enviando main.cpp…
SSH · pi · atenção: mudanças no alvo
SSH · pi · última sonda falhou
```

O item abre/foca a tool window Remote. Tooltip/detalhe mostra:

- host e pasta remota;
- raiz do espelho local;
- última sonda e horário;
- última sincronização e direção;
- job em curso;
- política de save/sync;
- limitações ativas, como “watcher remoto indisponível”.

Sem workspace espelhado, um perfil meramente selecionado não ocupa a status bar
permanentemente. Pode aparecer no seletor de execução quando for o contexto da
configuração ativa.

### 5.3 Header e Run/Debug

O header não ganha uma nova coleção de botões Remote. A configuração ativa de
execução já deve dizer o alvo:

```text
app · Local
app · SSH: pi
debugpy · SSH: bancada
```

Run e Debug continuam sendo os gestos normais. Se falta setup, o próximo passo
leva à seção correta da tool window.

### 5.4 Jobs, terminal e Problems

- deploy/sync aparecem em Jobs e na status bar enquanto ativos;
- shell abre uma sessão do terminal existente, nomeada `SSH · <alvo>`;
- `journalctl`, `dmesg` e comandos do sistema devem reutilizar terminal/output,
  não criar console próprio;
- falhas estruturadas com próximo passo entram também em Problems/Project
  Health quando afetam o workspace, não apenas como texto efêmero no painel.

## 6. Fluxo diário alvo

### 6.1 Primeiro uso de um alvo

```text
Adicionar alvo
→ escolher “usar SSH existente” ou “configurar servidor”
→ selecionar alias ou informar host/usuário
→ testar acesso e resolver autenticação, se necessário
→ escolher pasta remota ou deploy do projeto atual
→ confirmar resumo
→ abrir workspace / criar configuração necessária
```

O formulário avançado fica recolhido. `~/.ssh/config` e agente são o caminho
preferido; usuário, porta e chave explícita continuam disponíveis.

#### SSH já configurado

```text
Usar SSH existente
→ aliases concretos de ~/.ssh/config aparecem com a origem
→ escolher “pi”
→ a IDE resolve somente o resumo seguro via OpenSSH
→ testar
→ começar na home remota e escolher a pasta
```

O perfil pode continuar usando o modelo atual: `name` amigável, `host` igual
ao alias e overrides ausentes. Não se copiam valores resolvidos desnecessários
para o workspace. Entradas `Host *` ou com curingas podem participar da
resolução do OpenSSH, mas não aparecem como alvos selecionáveis.

#### SSH ainda não configurado

```text
Configurar servidor
→ host + usuário; porta fica em avançado
→ testar
→ se faltar confiança/autenticação, explicar a causa
→ oferecer abrir terminal guiado para primeiro acesso/ssh-copy-id
→ testar novamente
```

A IDE não pede nem guarda senha. Operações interativas ficam no terminal real,
onde host key, passphrase e senha são tratados pelo próprio `ssh`. O comando é
visível antes de executar. Geração/cópia de chave nunca ocorre silenciosamente.
Se o servidor só aceita senha, a UI explica que shell interativo funciona, mas
sync/deploy automático exige agente ou chave não interativa.

#### Escolha da pasta remota

Depois do teste, a home remota é o ponto inicial. Uma listagem de diretórios
permite navegar e selecionar, com campo de caminho ainda disponível para uso
avançado. Isso exige contrato remoto tipado; não se interpreta a saída de `ls`
no QML e não se concatena texto não escapado a shell.

### 6.2 Reabrir um workspace espelhado

```text
workspace abre
→ marcador identifica RemoteMirror
→ HUD mostra SSH + alvo + estado conhecido
→ sonda leve opcional/solicitada atualiza alcançabilidade
→ edição local funciona imediatamente
```

Abrir não deve bloquear o editor esperando rede. Estado inicial honesto:
**“estado remoto ainda não verificado nesta sessão”**.

### 6.3 Editar e salvar

```text
Ctrl+S/autosave
→ escrita local protegida
→ job de push do arquivo
→ HUD: enviando
→ sucesso: sincronizado
→ falha: arquivo local salvo, envio pendente/falhou
```

Falha de rede nunca faz parecer que o save local falhou nem que o arquivo já
chegou ao alvo. A UI distingue os dois resultados.

### 6.4 Mudança feita no alvo

Enquanto não houver watcher remoto, a HUD declara isso. **Puxar do alvo** é
gesto explícito. Quando watcher/conferência de estado existir:

```text
mudança remota detectada
→ comparar base/local/remoto
→ sem conflito: oferecer/aplicar política decidida
→ conflito: preview e escolha explícita
```

“Último rsync vence” não é experiência final aceitável para uso diário. É
limitação atual, preservada e exibida até existir contrato de conflito.

### 6.5 Build, deploy, run e debug

O fluxo normal deve parecer uma configuração de execução, não cinco botões de
infraestrutura:

```text
Build local/cross
→ Deploy se necessário
→ Run ou subir debug server
→ anexar debugger
```

A tool window mostra a pipeline resolvida e sua origem. A primeira versão pode
continuar usando as configurações e o kit atuais; o objetivo da UI é preparar e
explicar uma vez, depois deixar Run/Debug fazerem o trabalho.

## 7. Conteúdo de cada seção

### Overview

- alvo ativo e destino `user@host:port`;
- alcançabilidade: não verificada / verificando / alcançável / falhou;
- arquitetura e kernel da última sonda, com horário;
- workspace: local, espelho de `<path>` ou projeto com deploy;
- última sincronização;
- configuração ativa de Run/Debug;
- uma ação primária contextual.

### Workspace

- pasta remota e espelho local;
- política atual de save → push;
- Puxar, Empurrar alterações e Revisar mudanças;
- lista curta do último sync; detalhes completos em Jobs/output;
- aviso explícito enquanto delete/rename e watcher não forem suportados;
- ação de reparar/recriar espelho sem perder alterações locais.

### Run & Debug

- programa resolvido e origem;
- pasta de deploy;
- configuração de execução ligada ao alvo;
- debugger, porta e `remoteTarget` efetivos;
- ações **Configurar Run** e **Configurar Debug** somente quando faltarem;
- depois de configurado, abrir/selecionar a configuração em vez de recriá-la.

### Sistema

- ferramentas medidas no alvo;
- abrir shell;
- `journalctl`, `dmesg`, `systemctl` quando implementados;
- arquitetura/kernel e diagnósticos de permissão/rede;
- comando bruto em detalhe expansível/copiar, não como conteúdo dominante.

### Configurar alvo

- nome, host, usuário, porta, chave e deploy dir;
- testar antes de salvar não é assumido pelo contrato atual; até existir, a UI
  explica que a sonda usa o perfil salvo;
- remover exige confirmação e explica que o alvo remoto não é apagado;
- nenhum campo de senha.

## 8. Estados necessários

O frontend deve modelar explicitamente, sem inventar fato do backend:

```text
availability    unknown | probing | reachable | unreachable
workspace       local | mirror
sync            idle | pulling | pushing | synced | failed | attention
deployment      idle | running | success | failed
setup           incomplete | ready
discovery       idle | loading | ready | failed
authentication  unknown | ready | interactive_required | failed
```

`reachable` significa “a última sonda nesta sessão terminou com sucesso”, não
conexão permanente. O horário faz parte da apresentação.

Uma futura resposta do core pode fornecer snapshot próprio. Até lá, o
controller deriva estado de apresentação apenas dos resultados já recebidos e
não o persiste como fato autoritativo.

## 9. Commands e atalhos

Reutilizar IDs atuais e acrescentar apenas gestos sem equivalente:

```text
remote.list/openPanel
remote.probe
remote.open
remote.sync.pull
remote.sync.push
remote.shell
remote.focus
```

Os nomes finais devem respeitar o catálogo existente; isto é inventário de
gestos, não autorização para duplicar métodos IPC. `remote.sync.pull` e
`remote.sync.push`, por exemplo, podem ser commands de UI sobre o mesmo
`remote.sync` tipado.

Não fixar atalho global para todas as ações. Um atalho para focar Remote pode
entrar depois de medir colisões e frequência.

## 10. Segurança e integridade

- senha e token SSH não entram em perfil, log ou command line;
- chave privada explícita é caminho, nunca conteúdo;
- comandos exibidos devem mascarar qualquer segredo futuro por construção;
- host key e prompts interativos pertencem ao `ssh` do sistema no terminal;
- pull/push em massa não usa `--delete` sem um gesto destrutivo específico;
- remoção de perfil nunca remove arquivos locais ou remotos;
- sync futuro com delete/rename exige preview;
- conflito nunca é resolvido silenciosamente;
- abrir espelho não envia `.kinein/` ao alvo;
- Workspace Trust continua valendo para comandos vindos do projeto.

## 11. Fatias de implementação

### R0 — baseline de uso

- capturar painel atual em 1280×800 e 1024×700;
- roteiro real: adicionar, sondar, abrir espelho, editar/salvar, pull, deploy,
  Run, Debug e shell;
- registrar cliques, rolagem, pontos sem feedback e termos confusos;
- exercitar com alvo real quando disponível; falsos continuam no gate.

### R0.5 — entrada simples e descoberta

- separar “usar SSH existente” de “configurar servidor”;
- descobrir aliases concretos de `~/.ssh/config` com origem explícita;
- resolver detalhes efetivos pelo OpenSSH, retornando apenas campos seguros;
- guiar primeiro acesso/chave pelo terminal, sem persistir segredo;
- iniciar seleção da pasta na home remota;
- contrato tipado para listar diretórios, se confirmado necessário no teste.

Aceite: se `ssh <alias>` já funciona, criar e testar o alvo não exige redigitar
usuário, porta nem caminho da chave; se não funciona, o erro leva a um gesto
concreto e seguro, não a uma parede de termos.

### R1 — reorganização sem protocolo novo

- separar Overview/Workspace/Run & Debug/Configurar;
- uma ação primária por estado;
- detalhes técnicos recolhíveis;
- preservar todos os signals e métodos atuais;
- harness cobre a máquina de estados visual.

### R2 — Remote no shell e HUD

- entrada diária como tool window;
- item contextual na status bar para workspace espelhado;
- foco via command/paleta;
- job de sync/deploy refletido na HUD;
- nenhum novo fato inventado no QML.

### R3 — sincronização segura

- snapshot/base para detectar conflito;
- watcher remoto ou verificação periódica opt-in;
- preview para pull/push conflitante;
- propagação explícita de rename/delete;
- recuperação de falha e fila/retry do push de arquivo.

### R4 — pipeline Run/Debug

- configuração declara alvo e política de deploy;
- Run/Debug executam build → deploy → remote command de forma observável;
- configuração existente é editada, não recriada a cada clique;
- origem de programa, porta, debugger e kit fica explicável.

### R5 — sistema remoto

- journalctl/dmesg/systemctl;
- interpretador/LSP do alvo quando realmente necessário;
- Yocto/Buildroot reconhecidos no modelo;
- gate com `sshd` local e exercício em hardware real.

R1 e R2 são a reformulação de UX/HUD. R3–R5 aprofundam o backend e não devem
ser fingidos por UI antes do contrato.

## 12. Critérios de aceite da reformulação

- o workspace espelhado é identificável sem abrir painel;
- o usuário diferencia save local de push remoto;
- o caminho feliz diário não exige editar perfil;
- no segundo uso do mesmo alvo, a UI exige menos memória e menos gestos que
  montar o fluxo equivalente diretamente no terminal;
- um alias OpenSSH funcional é reutilizado sem reconstruir seus campos;
- a configuração do zero explica e guia confiança/chave sem guardar senha;
- a pasta remota pode ser escolhida sem conhecer previamente seu caminho
  absoluto;
- Run/Debug não exigem recriar configuração;
- sonda antiga não é mostrada como conexão viva;
- a ação primária muda conforme o estado e há no máximo uma por seção;
- detalhes de ssh/rsync continuam acessíveis e copiáveis;
- todas as capacidades atuais permanecem alcançáveis;
- keyboard focus e tab order são previsíveis;
- 1024×700 não exige rolar para descobrir o gesto principal;
- falha de rede deixa próximo passo e não perde alteração local;
- gates QML, fiação, propriedades, alcance, duplicação e binário abrindo ficam
  verdes;
- teste real em Linux remoto registra o que falsos não provam.

## 13. Decisões do autor ainda necessárias

Estas opções foram preservadas; nenhuma foi descartada:

1. Remote fica inicialmente na área esquerda, como troca de contexto, ou na
   direita, como inspeção do ambiente?
2. Ao abrir um espelho, a IDE deve apenas informar mudanças remotas, oferecer
   pull, ou aplicar pull automático quando não houver edição local?
3. Rename/delete remoto entra por preview em lote ou inicialmente por comandos
   explícitos separados?
4. Build no alvo faz parte do fluxo prioritário, ou a 0.3 foca build
   local/cross + deploy?
5. A seleção do alvo é por workspace, por configuração de execução ou ambas
   com uma precedência visível?

Até essas decisões, o plano conserva o comportamento atual e não escolhe em
silêncio.
