# `remote.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `remote.*` — o alvo Linux por SSH

Domínio novo no protocolo `0.120.0` (P6 fatia 1 do `roadmaps/42`, P6 do `40`
§4.1, 2026-09-17): a Raspberry Pi, a placa com imagem própria, como recurso
do projeto — no molde do `datasource.*`. Transporte é o `ssh`/`rsync`/`scp`
do sistema como **processo** (OpenSSH BSD, rsync GPL-3 — nunca crate).

```text
remote.list    {}                                -> { targets: [RemoteTarget], contacts: [RemoteContact] }
               contacts (0.153.0): a última sonda de cada alvo, em
               `.kinein/remote-contacts.json` — { name, ok, at (s Unix), arch?, kernel?, failure? }
remote.save    { target }                        -> { targets }   (cria/substitui pelo nome)
remote.remove  { name }                          -> { targets }
remote.probe   { name }                          -> { jobId, command }   (job)
remote.deploy  { name, source?, dest? }          -> { jobId, command }   (job)
remote.command { name, kind, program?, port? }   -> { command, remoteTarget?, name, source[] }
               kind: run | debugServer | debugpy | shell | copyId   (PURO: nada roda)
               copyId (0.133.0) -> `ssh-copy-id [-p P] [-i K] [user@]host`; sem chave
               nenhuma (0.153.0) -> `ssh-keygen -t ed25519 -f K && ssh-copy-id …`
remote.hostKey   { name }                        -> { name, host, keys: [{ kind, fingerprint }] }
               (0.153.0; `ssh -G` + `ssh-keyscan` + `ssh-keygen -lf`; não loga, não grava)
remote.trustHost { name, fingerprints[] }        -> { name, file, recorded }
               (0.153.0; escaneia DE NOVO e grava só as chaves cujas impressões vieram;
               o arquivo é o `UserKnownHostsFile` que o `ssh -G` reporta)
remote.parseCommand { command }                  -> { target: RemoteTarget, source[] }
               (0.134.0; SEM workspace; LE' a linha, nunca a executa; nada e' salvo)
remote.discover {}                               -> { aliases: [{ name, source }], sources[] }
               (0.132.0; SEM workspace; lê arquivo local, não conecta)
remote.resolve  { host }                         -> { host, hostName?, user?, port?,
                                                     identities[], proxyJump?, proxyCommand }
               (0.132.0; SEM workspace; `ssh -G`, que não conecta)

RemoteTarget   name · host · user? · port? (22) · identityFile? · deployDir? (~/kinein/<projeto>)
               — SEM senha, estruturalmente: `deny_unknown_fields`, e um teste reprova
               qualquer chave que pareça segredo no arquivo gravado

event.remote.probed   { jobId, name, success, arch?, kernel?, tools: [{ id, found, path? }],
                        error?, failure?, raw }
                        failure (0.133.0, ausente quando success): authentication |
                        unknownHost | hostKeyChanged (0.153.0) | host | network |
                        other — a causa TIPADA, nao a frase
event.remote.deployed { jobId, name, success, source, dest, command, error? }
event.remote.directories { jobId, name, requestedPath, success, path, parent?,
                           entries: [{ name, path }], error? }
```

**Persistência:** `.kinein/remotes.json` (`schemaVersion: 1`, ordenado pelo
nome; arquivo inválido = nenhum alvo; porta 22 e campos vazios ficam
ausentes). **Probe:** `ssh -o BatchMode=yes -o ConnectTimeout=5 [-p P] [-i K]
[user@]host 'uname -m; uname -sr; for t in gdbserver python3 rsync; do
printf "%s=" $t; command -v $t || echo; done'` — uma linha por fato, lida
sem adivinhar; `BatchMode` faz a falta de chave falhar em segundos, e o
`error` diz o passo (`ssh-copy-id user@host`; host não resolvido; sem
rota em 5 s). **Deploy:** `rsync -az --delete -e 'ssh [-p P] [-i K]'
<origem> [user@]host:<dest>/` quando há `rsync` nesta máquina, senão `scp
[-P P] [-i K] -r`; a origem padrão é `<root>/build`, o destino o `deployDir`
do alvo ou `~/kinein/<projeto>` (o `~` fica sem aspas para o shell REMOTO
expandir); origem inexistente é recusa síncrona ("compile antes"). **Antes de copiar**, o
job roda `ssh -o BatchMode=yes … 'mkdir -p <dest>'`: medido em 2026-09-24 contra
um alvo real, nem `rsync` nem `scp -r` criam diretório intermediário, e o padrão
`~/kinein/<projeto>` não existe numa placa recém-instalada — o **primeiro deploy
de qualquer alvo novo** falhava com um erro de `rsync` que não dizia o que fazer.
É `mkdir -p` e não `rsync --mkpath` porque o `--mkpath` exige rsync 3.2.3+ dos
dois lados, e o projeto usa o que o sistema tem. Falha no `mkdir` não
interrompe: se for real, a cópia falha em seguida com a mensagem dela.
**Comandos:** `run` → `ssh -tt … '<programa>'`; `debugServer` → `ssh -tt …
'gdbserver :2345 <programa>'` com `remoteTarget = host:2345` (o `gdb -i
dap` faz `attach` a servidor desde 0.89.0); `debugpy` → `ssh -tt … 'python3
-m debugpy --listen 0.0.0.0:5678 --wait-for-client <script>'` com
`remoteTarget = host:5678` (o attach TCP de 0.109.0); `shell` → `ssh …
[user@]host` sem `BatchMode`, para o terminal da IDE. `program` relativo
entra no `deployDir`; absoluto ou `~` vai como dado; ausente vira
`<binario>` com a fonte dizendo "edite". O `-tt` força pty mesmo sem
terminal local (o `debugServer` do kit roda por `sh -c` com stdio fechado):
matar o `ssh` local derruba o processo remoto por SIGHUP — sem isso o
`gdbserver` ficaria órfão na placa. Na UI: painel **Alvo remoto (SSH)** (menu
Ambiente; comando `remote.list`): lista, formulário sem campo de senha,
**Sondar** (primário), **Enviar**, e os quatro botões que levam o resultado
do `remote.command` ao dono certo — `runConfig.save` ("Rodar em pi",
"debugpy em pi"), `toolchain.setKit { remoteTarget, debugServer }` (só os dois
campos; a ponte ganhou `toolchainSetKitRemote`) ou
`runtimeController.submitShellInput`. **Não entrou na fatia 1 (dito):**
workspace remoto, LSP do outro lado, mapeamento de caminhos, `sshd` local
no gate — o `RemoteContext` inteiro do 28 §4 segue no `42` §P6.

**A guarda (2026-10-16, 16.4a do `roadmaps/59` §7.1.4; sem mudança de
forma).** O `remotes.json` pode vir de um projeto clonado, então a regra do
`remote.save` vale também no USO: `probe`, `deploy`, `command`, `hostKey`,
`trustHost`, `open` e a sincronização recusam com `INVALID_REQUEST` ("não
pode ser usado: <motivo>") o alvo cujo host não é nome, IPv4 ou `[IPv6]`
ou começa com `-`, cujo usuário sai de `[A-Za-z0-9._-]` ou começa com `-`,
cuja chave tem aspas ou começa com `-`, ou com caractere de controle em
chave, pasta, programa ou origem. O `remote.list` continua listando todos.
As linhas de texto (`command` do `remote.command`, o `-e` do `rsync`, o
`mkdir` remoto, o destino do deploy) põem entre aspas simples o argumento
fora de `[A-Za-z0-9_@%+=:,./-]`, com o `~/` do começo fora delas. O
`remote.trustHost` recusa quando o `known_hosts` já tem chave para o host
(`ssh-keygen -F`): confiar é só o primeiro contato. O marcador
`.kinein/remote-mirror.json` só faz de uma pasta um espelho dentro do cache
da IDE (`<cache>/kinein-vectis/remote/`), onde o `remote.open` o cria.
Os caminhos também não saem (16.4c): a origem do `remote.deploy`, resolvida
com os links, tem de ficar dentro da raiz do projeto (`..`, absoluta ou link
para fora: `INVALID_REQUEST`, "fica fora do projeto"), e os `paths` do
`remote.sync` (e o push depois de gravar) não atravessam um link do espelho
que aponta para fora.

### Descobrir e explicar (`0.132.0`, fatia R0.5)

Fonte: [`remote-ssh-ui-hud.md`](../../especificacoes/remote-ssh-ui-hud.md) §6.1 e
[`roadmap 48`](../../roadmaps/48-arquitetura-executavel-da-serie-0.3.md) §8.1. A
lacuna não era guardar o alias — `RemoteTarget.host` sempre aceitou um alias com
user/port/identity ausentes. Era **descoberta e explicação**.

**`remote.discover {}`** lê `~/.ssh/config` e, no ponto em que aparecem, os
`Include` dele. Só um `Host` **concreto** vira alias selecionável: padrão com
`*`, `?` ou `!` continua valendo na resolução do OpenSSH, mas não é algo que o
usuário possa escolher. `Host "um dois"` é **um** padrão com espaço, recusado —
não dois aliases inventados. Cada alias diz o arquivo que o declarou
(`source`, com `~` no lugar da home), porque um alias de `config.d/` não é a
mesma coisa que um do arquivo principal quando algo destoa. Limites explícitos,
para um `Include` mal escrito não virar varredura de disco: 16 arquivos, 8
níveis, 256 KiB por arquivo, 512 aliases, e `*` casa dentro de **uma** pasta.
Máquina sem `~/.ssh/config` devolve lista vazia — "não há SSH configurado" é
estado do produto, não erro do protocolo.

**`remote.resolve { host }`** roda `ssh -G <host>`, que imprime a configuração
**efetiva** sem abrir sessão, e devolve uma **allowlist**. O dump inteiro não
sai: `localforward`, `sendenv` e o resto são descartados. `proxyJump` é uma
especificação de host, segura de mostrar; de um `ProxyCommand` sai apenas
`proxyCommand: true` — o texto é uma linha de comando arbitrária, que pode
conter segredo, e por isso **nunca** é devolvido. `host` volta ecoado para a UI
descartar resposta atrasada de outro alvo. O core recusa antes de rodar
processo um host vazio, com espaço, com `@`, com controle, longo demais ou
começando por `-` (o `ssh` leria como opção). Como `ssh -G` imprime valores
efetivos, `port` vem resolvido (22 quando ninguém mudou) e `identities` são as
que o `ssh` **tentaria** — não prova que alguma exista. Medido nesta máquina em
2026-09-24: sem `~/.ssh/config`, `ssh -G localhost` imprime 90 linhas e lista
**cinco** `identityfile` padrão, nenhuma presente no disco. Por isso a UI diz
"que o ssh tentaria", e o core não filtra por existência: filtrar trocaria "o
que o OpenSSH diz" por "o que nós achamos que vai funcionar". A UI não persiste um
override que só repete o que o OpenSSH já faria: escolher um alias grava
`{ name, host }` e nada mais.

`ssh -G` roda com `-F` apontando **o mesmo arquivo** que a descoberta leu.
Medido em 2026-09-24 contra um alvo real: o OpenSSH **não honra `$HOME`** para
achar o `~/.ssh/config` — usa a base de senhas do sistema. Sem o `-F`, a
descoberta podia listar aliases de um arquivo enquanto a resolução explicava
outro, e a IDE afirmaria sobre um config que o `ssh` não usaria. Sem arquivo
nenhum vai só `-G`: `ssh -F <inexistente>` é erro, e "não há config" é estado
normal.

**Ressalva dita:** `ssh -G` avalia `Match exec` do config do próprio usuário,
o que executa um comando local. Rodar síncrono (como git, probe e size já
fazem) é aceito porque `ssh -G` é local e instantâneo; um `Match exec` lento
atrasaria o `ssh` do usuário do mesmo jeito.

**Não entrou nesta fatia (dito):** `remote.directories` — a navegação tipada da
pasta remota. A §8.1 do 48 a propôs "se confirmado necessário no teste", e o
teste desta fatia não chegou à escolha de pasta. O campo de caminho continua
sendo o caminho.

**Diferenças frente à proposta da §8.1**, registradas: `discover` devolve também
`sources` (os arquivos lidos, para a UI dizer de onde veio); `resolve` ecoa
`host` e troca o `proxy?` genérico por `proxyJump?` + `proxyCommand: bool`,
porque juntar os dois num campo exigiria devolver o texto do comando.

### A linha colada vira um perfil (`0.134.0`, fatia R0.5)

O segundo caminho da §6.1 da especificação — **configurar servidor** —, e ele
não é um formulário. A ideia vem do `Remote-SSH: Add New SSH Host…` do VS Code
(comparação na §3.1 daquele documento): a pessoa cola o comando que já usa.

Cabe aqui sem quebrar a regra "a UI não monta linha de `ssh`": quem **fornece** a
linha é o usuário, e quem a **interpreta** é o core. E ela é lida, nunca
executada — texto entrando, perfil saindo.

Entende `ssh [-p N] [-i chave] [-l usuário] [-o Port=/User=/IdentityFile=]
[usuário@]host`, aceita só o destino, tolera um `$ ` colado junto do prompt e
respeita aspas. O `name` é **proposto** a partir do host (o primeiro rótulo de um
nome; um IPv4 inteiro vai inteiro, porque `192` não é nome de nada) e a UI deixa
editar.

**O que ele RECUSA, em vez de limpar em silêncio:**

- qualquer `;`, `|`, `&`, crase, `$(`, `>` ou `<` — só fazem sentido para um
  shell, e o que vem depois deles não é assunto de um perfil;
- opção que o perfil não modela (`-J bastion`, `-o ProxyCommand=…`), **dizendo
  qual** e mandando deixá-la no `~/.ssh/config` para escolher o alias aqui;
- um comando remoto depois do destino, porque isso seria um comando, não um
  perfil.

Aceitar e ignorar prometeria um alvo que não se comporta como a linha colada —
e a pessoa não saberia por quê.

### A falha vira um gesto (`0.133.0`, fatia R0.5)

A §3 da [`remote-ssh-ui-hud.md`](../../especificacoes/remote-ssh-ui-hud.md) lista
como defeito nº 14: *"a mensagem 'use `ssh-copy-id`' transfere o problema ao
terminal sem guiar"*. O core já separava as causas para escolher a frase — dizê-la
em **tipo** expõe uma decisão que ele já tomava, não inventa dado. A UI escolhe o
gesto por `failure`, nunca casando texto: frase muda de idioma, tipo não.

Só `authentication` tem um gesto que a resolve, e é `ssh-copy-id`. O botão aparece
**onde a causa foi explicada**, no veredito da sonda. Clicar **compõe** a linha e
a mostra; rodar exige um segundo gesto. Isso é a §10 da especificação: *"geração/
cópia de chave nunca ocorre silenciosamente"* e *"o comando é visível antes de
executar"*. A IDE não gera chave, não digita senha — o `ssh` pede no terminal, e
o host key é aceito ali, uma vez.

`copyId` reusa o `-p`/`-i` do perfil: copiar a chave por outra porta que não a do
alvo copiaria para a máquina errada. Com `-i <privada>` o `ssh-copy-id` procura a
`.pub` correspondente. Uma linha armada **não sobrevive à troca de alvo**, porque
ela carrega um host.

### O workspace espelhado (`0.122.0`, P6 fatia 2)

```text
remote.open   { name, path }                       -> { jobId, command, mirror }   (job: pull)
remote.directories { name, path? }                 -> { jobId, command }           (job: SSH)
remote.sync   { direction: pull | push, paths? }   -> { jobId, command }           (job)
remote.status {}                                   -> { mirror?: RemoteMirror }
RemoteMirror  name · host · path (no alvo) · mirrorRoot (local)

event.remote.synced { jobId, name, direction, success, command, changed: [caminho],
                      error?, mirror }
workspace.open de um espelho  -> … , remote: RemoteMirror
```

A pergunta era "como abrir a pasta da Pi". O VS Code (Remote-SSH,
proprietário) sobe um servidor no alvo; aqui a resposta é a outra escola
(o "deployment" do PyCharm, o fluxo Zephyr/Yocto): **a pasta remota vira
um espelho local por `rsync`, e a IDE abre o espelho como workspace
comum** — zero mudança em `fs.*`, índice, busca, git e LSP. `remote.open`
puxa a árvore (`rsync -az -i --exclude .kinein -e 'ssh [-p] [-i] -o
ControlMaster=auto -o ControlPath=<cache>/ssh-%C -o ControlPersist=60'
[user@]host:<path>/ <espelho>/`) para `~/.cache/kinein-vectis/remote/
<alvo>/<hash do path>/<basename>` — o basename vira o nome do workspace —,
grava `.kinein/remote-mirror.json` (`schemaVersion`, `name`, `host`,
`path`) e copia o alvo para o `.kinein/remotes.json` **do espelho**, que é
autossuficiente (usuário, porta, chave); o `.kinein` nunca sincroniza,
então nada disso chega ao alvo. Quando o `event.remote.synced { direction:
pull }` chega, a UI abre o espelho pelo `workspace.open` de sempre, cuja
resposta traz `remote`. **Salvar empurra:** um `fs.write` num espelho
dispara o job `Empurrar <arquivo> para <alvo>` só com aquele caminho (o
`ControlMaster` do ssh mantém uma conexão só entre gravações). `remote.sync`
move a árvore inteira ou `paths` relativos (sem `..`, nunca `.kinein`),
**nunca com `--delete`** — apagar do outro lado é gesto explícito, fora
desta fatia. `changed` é a saída `-i` (itemize) do `rsync`, lida linha a
linha. **Dito como não feito:** watcher do lado remoto (o que muda no alvo
só aparece ao Puxar); renomear/apagar não propaga; o LSP resolve contra
ESTA máquina (C/C++ cross usa o sysroot do kit; Python/Rust do alvo não);
edição dos dois lados = o `rsync` mais recente vence.
