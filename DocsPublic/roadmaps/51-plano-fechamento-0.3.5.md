# Plano executável antes das próximas etapas — 0.3.5

Data: 2026-10-01, America/Sao_Paulo. **Classe: PLANO OPERACIONAL.** Este plano foi escrito **antes** de continuar a implementação e a publicação, conforme pedido do autor. A cada etapa, registrar o resultado no roadmap público 40 e atualizar aqui o estado se a sessão for interrompida. O histórico anterior e as provas já obtidas estão no [`40.7`](40.7-registro-das-entregas.md).

## Autorização e estado de entrada

- O autor aprovou H0 em 2026-09-30. Em 2026-10-01, autorizou explicitamente concluir a 0.3, criar uma pasta chamada **`KV0.3`** para o AppImage, fazer commit e push, tentar publicar a release e atualizar o site. A documentação de cada etapa vem **antes** da implementação, para resistir a interrupção por limite semanal.
- Repositório principal: branch `main`, remoto `git@github.com:ViktorWalde/KineinVectis.git`, HEAD de partida `f962658`; há mais de cem caminhos modificados/não rastreados de trabalhos acumulados. `dist/` é ignorado por Git. Não resetar. GitHub CLI está autenticado como `ViktorWalde`, token com escopo `repo`; a conexão SSH de push ainda precisa ser provada.
- Release pública anterior: tag `v0.2.0`, pré-release, asset `Kinein_0.2.zip`. O site público é outro repositório, `ViktorWalde/KineinSite`, com homepage `https://viktorwalde.github.io/KineinSite/`; o projeto principal não tem GitHub Pages próprio.
- AppImage candidato 0.3.5 já existe em `dist/` e passou smoke local e Debian mínimo, mas ficará **obsoleto** quando o código Remote abaixo mudar. O hash candidato antigo está no handoff. O protocolo antes da nova fatia era `0.142.0`.

## Etapa 1 — fechar a escolha da pasta remota começando na home

**Por que:** `RemoteMirrorView.qml` ainda exige digitar caminho. O roteiro V2/R0.5 em roadmap 47 e 48 §8.1 pede iniciar na home do alvo. `remote.command` abre shell e a alternativa de digitar caminho não oferece escolha na interface; logo há motivo medido para o contrato tipado `remote.directories` previsto no 48. Reutilizar `remote::ssh_args`, job cancelável, bridge `CoreClient`, roteadores e `RemoteWorkspaceController`; nenhum segundo cliente SSH ou painel paralelo.

**Ponto exato de interrupção ao escrever este plano:** o protocolo novo `0.143.0` já está parcialmente no checkout: tipos `RemoteDirectories*` em `crates/kinein-protocol/src/remote.rs`; handler `crates/kinein-core/src/handlers/remote_directories.rs`; roteamento em `handlers.rs`/`lib.rs`; `remote::quote` tornado `pub(crate)` para reuso; método/signal de `CoreClient` e despacho de evento; `RemoteWorkspaceController.qml` e roteadores QML já possuem estado/fiação de browse. **Ainda faltam a view, as propriedades no painel/host, docs IPC, teste integrado com SSH e gates. Não compilar nem publicar como estado final.** Um teste unitário inicial de parsing passou antes da última edição parcial.

**Execução restante:**

1. Revisar o handler parcial: evento inclui `requestedPath` para rejeitar resposta atrasada; manter o SSH num Job, quoting POSIX compartilhado, resultado delimitado por NUL e teto de saída. Corrigir erros de compilação e falha. Pastas com espaços ainda não podem ser abertas por `remote.open`; ocultá-las na lista e explicar o limite, sem prometer suporte.
2. Completar `RemoteMirrorView` com gesto **Escolher pasta**: primeira consulta sem `path` resolve `$HOME` no alvo, exibe a pasta e filhas, permite subir para `parent`, entrar numa filha e abrir a pasta atual como espelho. Encaminhar propriedades/sinais por `RemotePanel` e `RemotePanelHost`; não duplicar o campo/fluxo `openPath/openFolder` atual.
3. Provar core com um SSH simulado para casos de home, filho, nome inválido, autenticação/falha, e com `scripts/testar-remote-ssh.sh` contra `sshd` real. Provar QML: resposta atrasada de outro alvo/workspace, erro e escolha que chama o `remote.open` existente. Rodar arquitetura, fiação IPC, duplicação QML, lint/build e smoke do primeiro frame.
4. Atualizar `DocsPublic/arquitetura/03-protocolo-ipc.md`, manual, especificação Remote, roadmaps 40/47/48 e changelog com contrato, prova e limites reais. Só então marcar V2/R0.5 fechado.

**Aceite:** uma pessoa com alvo salvo clica em Escolher pasta, vê a home remota, navega para uma filha e abre o espelho sem decorar caminho; resposta atrasada não troca seleção; falha SSH é exibida; o mesmo fluxo passa em Linux com `sshd` real.

**Registro parcial de 2026-10-01:** core/protocolo/UI e roteiro real foram
implementados; o teste `sshd` navegou desde `/home/kinein` e abriu o espelho
por `rsync`. O teste revelou e levou à correção do `ControlPath` longo.
Clippy, harnesses Remote, arquitetura e fiação IPC passaram. O build final da
UI e o gate integrado ainda estavam em execução neste ponto; ver 40 §7.143
antes de considerar a etapa encerrada.

**Retomada de 2026-10-01 (Claude):** o primeiro `scripts/verificar.sh`
reprovou em `tests::remote_mirror::a_remote_folder_becomes_a_local_mirror_with_a_marker`.
O teste ainda esperava `ControlMaster`, mas a HOME de teste
(`/tmp/kinein-core-tests/<pid>-…/home/.cache/kinein-vectis/remote`) excede a
margem do socket, e o core corretamente cai para SSH comum. O teste passou a
afirmar esse ramo; os dois ramos seguem cobertos pelos testes unitários de
`remote/mirror.rs`. A segunda rodada reprovou no `clang-format` de
`ui/src/core_client.h` e `ui/src/markdown_document.cpp`, ambos com edições
pendentes do Codex, e eles foram formatados. Para as provas nativas da
Etapa 2, o input real no GNOME Wayland agora sai por `ydotool` (uinput),
porque XTest pelo Xwayland não entrega eventos.

## Etapa 2 — prova final de integração P3/H0

**Estado:** H0 já está aprovada/fechada. `fs.readExternal` e o harness da aba externa somente leitura passaram, mas o gesto nativo de arrasto de outro aplicativo para essa aba ainda não foi provado. Os gestos da árvore X11 estão no roadmap 40 §§7.131–7.140. Drag IDE → Nautilus e recuperação visual da lixeira em `/tmp` não foram confirmados; não anunciar esses efeitos como feitos.

**Execução:** tentar teste real com arquivo externo **apenas em `/tmp`**, observar aba somente leitura, ausência de importação e arquivo fonte intacto; tentar em sessão gráfica confiável/isolada se a sessão GNOME Wayland atual não aceitar injeção de ponteiro. Não usar prints que contenham notificações privadas. Registrar comandos e resultado/limitação no roadmap 40. Se a matriz P3 exigir provas não obtíveis nesta máquina, registrar bloqueio honesto e manter release pendente até haver prova ou uma decisão explícita de escopo.

**Resultado em 2026-10-01 (roadmap 40 §7.144):** drop externo no editor
provado em X11 e Wayland. O defeito do ponto de "modificado" na troca de aba
foi corrigido, com harness provado por mutação. A IDE entrega a URL a
receptores Qt e GTK4. Na mesma data, a pedido do autor, os três limites foram
resolvidos antes da release: arrastar para o Nautilus funciona com o gesto
humano (40 §7.147), a recuperação pela Lixeira foi provada na HOME (§7.146) e
as pastas com espaço passaram a funcionar no Remote (protocolo 0.144.0,
§7.145).

## Etapa 3 — gates e artefato final V8

1. Depois da última edição de código, executar `bash scripts/verificar.sh` completo; corrigir falhas reais. Medir build e primeiro frame debug/release, Clippy/testes, fiação, arquitetura, QML e documentação. Repetir o roteiro local CMake/Cargo/Python e `scripts/testar-remote-ssh.sh` se a fatia Remote o afetar.
2. Recriar `dist/Kinein-Vectis-0.3.5-x86_64.AppImage` com `bash scripts/empacotar-appimage-portatil.sh`. Repetir `bash scripts/testar-appimage.sh dist/Kinein-Vectis-0.3.5-x86_64.AppImage` e `bash scripts/testar-appimage-portatil.sh`; verificar SHA-256 novo e conteúdo `Tutorial.md`/manual. Testar migração 0.2 pela UI se possível. Atualizar versões Cargo/CMake/AppStream/Sobre, README, manual, tutorial, changelog e release notes para a entrega efetiva.
3. Registar no roadmap 40 o hash **final**, os comandos, resultados e limites. O AppImage que existe agora é candidato anterior à fatia Remote; não publicar esse arquivo se o código mudar.

**Aceite:** gate completo verde, artefato final gerado depois do último código, smoke em Debian mínimo sem SDK, versões e documentação coerentes, regressões 0.2 e Remote reais medidas.

## Etapa 4 — pasta `KV0.3`, commit e push

- Criar **`KV0.3/` na raiz do repositório principal** para esta entrega, pois `dist/` é ignorado; copiar o AppImage final, `.sha256`, `SHA256SUMS`, instalador e tutorial. Incluir notas da release/checksum em arquivo texto dentro da pasta; manter o nome `KV0.3` exato. Conferir todos os arquivos e o checksum após a cópia. Gerar também `KV0.3.zip` como asset de download, contendo essa pasta, sem depender de uma instalação Qt na máquina da pessoa.
- Revisar `git diff --check`, `git status --short` e o conteúdo do commit para evitar arquivos temporários, chaves, prints privados, `target/`, `build/` e `dist/`. O binário na pasta `KV0.3` será versionado por pedido explícito do autor. Commit no projeto principal com mensagem que identifique o fechamento 0.3.5; push para `origin/main`. Registar hash do commit e estado remoto. Se o push falhar por autenticação/rate limit, conservar o commit local e documentar o comando de retomada.

## Etapa 5 — release pública e site

- Com commit/push e gates finalizados, criar tag/release `v0.3.5` no GitHub do projeto principal, preferencialmente como **pré-release** enquanto a série pública continuar beta, com notas reais e assets `KV0.3.zip`, AppImage e checksum. Verificar a URL e checksum baixável após publicar. A autorização do autor para "colocar a nova release no ar" já foi dada; não pedir de novo.
- Clonar/abrir o repositório público `ViktorWalde/KineinSite` em uma área de trabalho separada, ler a estrutura atual e atualizar versão, download, link da release, destaque e instruções pertinentes. Rodar o build/teste próprios do site antes de commit e push, sem alterar conteúdo alheio. Verificar a página publicada em `https://viktorwalde.github.io/KineinSite/` após deploy, respeitando eventual atraso de Pages.
- Se GitHub, quota, acesso ou deploy impedir uma parte: documentar erro exato, o que já foi publicado, comandos/arquivos preparados e próximo gesto. Não afirmar que a release/site estão no ar antes de conferir.

## Ordem e disciplina de documentação

Esta ordem é deliberada: **plano escrito → código → teste → registro do resultado → próxima etapa**. Antes de cada mutação de release/site, reler este plano e o estado do gate. A 0.3.6 começa depois da 0.3.5 validada e publicada, sob roadmaps 49/50.
