# `container.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `container.*` — Docker e Podman como domínio nativo

Domínio novo no protocolo `0.92.0` (2026-09-12). A decisão é de 2026-07-17
(`roadmaps/28` §0: *"vão ser cidadãos nativos"*), e as invariantes registradas
lá em §4 são o desenho: **a UI nunca chama `docker`**, **toda ação é Job
cancelável**, **permissão é visível**, e **Podman é motor de primeira** — nesta
máquina `docker` é o shim `podman-docker`, medido em 2026-09-12.

```text
container.status  {}                       -> ContainerStatus
container.list    { all? = true }          -> { containers, engine, rawOutput, hint? }
container.images  {}                       -> { images, engine, rawOutput, hint? }
container.action  { id, action, volumes? } -> { jobId }            (job; start|stop|restart|remove;
                                                                     volumes só com remove, 0.170.0)
container.volumes { id }                   -> { id, volumes: [ContainerVolume] }   (0.170.0)
container.open    { id, mode }             -> { id, command }      (aba de terminal; logs|shell)
container.compose { action, file? }        -> { jobId }            (job; up|down; exige workspace
                                                                     E arquivo de compose, ou `file`)

ContainerStatus   engine? (docker|podman), binary?, version? (do servidor),
                  clientVersion?, serverName?, emulated, rootless?, socket?,
                  reachable, compose?, composeFile?, hint?, rawOutput
ContainerInfo     id, names[], image, state, status, ports[], created
ImageInfo         id, repository, tag, size, created
ContainerVolume   name, anonymous, sizeBytes?

event.container.finished { jobId, action, target, ok, message }
```

**A detecção pergunta ao binário, não ao nome.** `docker --version` do shim
escreve *"Emulate Docker CLI using podman"* em stderr; tratar isso como Docker
Engine erraria o formato de `ps` (o Podman devolve um **array** em `--format
json`; o Docker, **um objeto por linha** em `--format '{{json .}}'`) e o
diagnóstico (não há daemon nem grupo `docker` num Podman rootless). Quando o
`docker` é o shim, o core prefere o `podman` real e diz `emulated: true`.

**O parser aceita as duas formas e devolve a saída crua sempre** — a mesma
regra do `probe.list`. A forma do Docker foi lida da documentação e **não**
verificada contra um Docker Engine real (esta máquina não tem um); a do Podman
5.8.4 foi medida, inclusive o detalhe de que `images --format json` não traz
`repository`/`tag` e sim `Names ["repo:tag"]`.

**`status` é a tela de "ativar a ferramenta".** Sem motor: o passo de
instalação da distro. Motor que não responde: `permission denied` no socket →
grupo `docker` e relogar (ou o Podman, sem grupo); contexto do `docker`
apontando o socket do Podman (`podman.sock`) → `systemctl --user enable --now
podman.socket`; daemon parado → `systemctl enable --now docker`; Podman → a
saída crua e o `podman.socket` do usuário. A IDE **imprime** o comando; nunca
roda `sudo`.

**O motor é o que RESPONDE (`0.168.0`, 16b.1, `roadmaps/59` §2.5.2).** Na
ordem `docker`, `podman`, vale o primeiro cujo `ps` responde; nenhum
responde, vale o primeiro, com a dica dele. Toda pergunta curta ao motor
(`ps`, `version`, `info`, `context`) tem prazo de 5 s e, no estouro, a
árvore do processo morre e a resposta diz "não respondeu em 5 s".
`status`, `list` e `images` rodam fora do laço; `action` e `compose`
escolhem o motor dentro do job. `version` é a versão de quem **atende** o
CLI (`docker version --format '{{json .Server}}'`): um `docker` cujo contexto
aponta o Podman diz `serverName: "Podman Engine"` e a versão do Podman; a do
CLI fica em `clientVersion`, mesmo com o servidor fora.

**Motor que não responde é erro, nunca lista vazia (`0.168.0`).** `list` e
`images` com o motor negando ou mudo respondem `TOOL_NOT_FOUND` com a dica na
mensagem e `details: { engine, reachable: false, hint, rawOutput }`. Antes, o
`ps` negado devolvia stdout vazio e a tela dizia "nenhum container neste
motor" (medido em 2026-10-15 com o Docker 29.9.1 negando o socket).

**`action` e `compose` são jobs** (`JobRisk::Medium`; `remove` é `High`): cada
linha do motor vai para `event.job.output`, cancelar mata o filho, e o
`event.container.finished` leva as últimas linhas como motivo. `rm` vai **sem
`-f`**: remover o que roda é dois gestos (parar, remover), de propósito. O
"Apagar os dados" do Banco (`datasource.destroy`) é outro caminho e usa `rm
-f -v`: o `-v` leva os volumes anônimos, onde a imagem guarda os dados
(`0.168.0`).
`compose up` é `-d` — um job que nunca termina não é job; a saída viva mora
na aba de logs.

**O "Remover" pergunta pelos volumes (`0.170.0`, 16b.3, `roadmaps/59`
§2.5.2).** `container.volumes { id }` responde fora do laço, pelo motor que
responde (mudo é o erro da 16b.1): os `Mounts` do tipo `volume` do
`inspect`, `anonymous` quando o nome tem 64 hexadecimais, e `sizeBytes` da
tabela de texto do `system df -v` (decimal, 4 algarismos; o JSON verboso não
existe no Podman 5.7), ausente quando o motor não o dá. `container.action`
com `volumes: true` só vale com `remove`, e vira `rm -v` (os anônimos do
container; os nomeados ficam); com outra ação, `INVALID_PARAMS`.

**O compose é do PROJETO, e o core diz se ele existe (`0.108.0`,
2026-09-13).** A ferramenta é da máquina (`compose`); o arquivo é do workspace
aberto: `composeFile` é o primeiro dos nomes que a ferramenta procura sozinha
na raiz (`compose.yaml`, `compose.yml`, `podman-compose.*`,
`docker-compose.yml|yaml`, `container-compose.*` — a ordem do podman-compose
1.6.0, lida em `COMPOSE_DEFAULT_LS`; os `*.override.*` não contam porque
sozinhos não sobem nada), ausente sem workspace ou quando o projeto não tem
nenhum. `container.compose` sem `file` num projeto sem arquivo **recusa com
`INVALID_PARAMS` antes de subir um job** — medido: o podman-compose sai com
255 e *"no compose.yaml, docker-compose.yml or container-compose.yml file
found"*, e um job que só pode falhar não é job. Na UI, "compose up"/"compose
down" só acendem com motor respondendo + ferramenta + arquivo, e a linha do
motor diz o que falta ("abra um projeto" / "o projeto não tem compose.yaml").
A mesma tarde corrigiu a classe visual que escondia isso: um botão primário
DESLIGADO vestia o âmbar a 72% de opacidade e continuava a coisa mais chamativa
do painel — o clique que não fazia nada lia-se como "o Docker não funciona";
agora desligado é um botão comum e apagado (`KvButton`/`KvIconButton`,
provado por `tst_kvbutton_states.qml` e `tst_container_panel.qml`).

**`open` reaproveita o terminal.** `logs -f --tail 200 <id>` e `exec -it <id>
/bin/sh` sobem pelo mesmo `open_command` do domínio `terminal` e voltam como
uma sessão de terminal (`terminal.input`/`resize`/`close` a reconhecem). O
shell dentro do container é a semente do *contexto remoto* do `roadmaps/28`
§4 — o que falta para *dev containers* é o path mapping e o ciclo de vida,
não o transporte.
