# ADR-0003 — linuxdeploy para distribuição AppImage

- **Status:** aceita
- **Data:** 2026-07-14
- **Escopo:** tooling de build e distribuição Linux x86_64

## Contexto

Testadores da Kinein Vectis precisam abrir a IDE sem clonar o repositório nem
instalar Rust, CMake de desenvolvimento ou Qt. A aplicação Qt/QML, o
`kinein-core`, os plugins de plataforma e as bibliotecas Qt devem formar um
artefato único. Compiladores, LSPs, debugadores e CLIs de IA usados nos
projetos permanecem ferramentas externas e opcionais.

Empacotar ELF, RPATH, plugins Qt e imports QML manualmente duplicaria uma
solução madura e criaria risco de bibliotecas ausentes. A documentação oficial
do AppImage recomenda montar um AppDir e usar linuxdeploy; para Qt/QML, o
plugin oficial descobre módulos, bibliotecas e plugins necessários.

## Decisão

Adotar em **MODE-A, somente durante o build**:

- `linuxdeploy` `1-alpha-20251107-1`, commit
  `cc7b86472c3caa3fd729b9dc502fd2aa78394257`;
- `linuxdeploy-plugin-qt` `1-alpha-20250213-1`, commit
  `ce5291e25979d7d6417551d787f308b88c1b8d76`.

Os AppImages oficiais das ferramentas são obtidos por URL de release imutável
e verificados pelos SHA256 registrados em `OPEN_COMPONENT_REGISTRY.json`. O
canal mutável `continuous` não é aceito sem checksum. O runtime type-2
embutido atualmente só é publicado nesse canal; a receita fixa o commit
`75849dce7cc37e4319b633df1f116ca895c71a12` e recusa qualquer asset cujo
SHA256 não seja o auditado. As ferramentas rodam no builder oficial Rust
1.96.1/Debian 12 fixado pelo digest amd64
`sha256:d99f7b31f49909348dc59b51f3c95d1efded1701ffb222f095aaab7de3c4abd8`.

A receita é dividida em:

- `scripts/empacotar-appimage-portatil.sh`: cria/executa o builder Podman ou
  Docker e é a entrada canônica;
- `scripts/empacotar-appimage.sh`: quando chamado sem argumentos, encaminha
  para a entrada canônica; dentro do builder, o argumento interno
  `--baseline-worker` habilita a compilação, instalação no AppDir, verificações
  e geração do AppImage;
- `scripts/instalar-appimage.sh`: integra no menu do usuário o AppImage de
  maior versão encontrado na pasta, sobrescreve um único atalho e oferece
  remover versões anteriores;
- `scripts/testar-appimage.sh`: valida estrutura, core e primeiro frame
  offscreen;
- `scripts/testar-appimage-portatil.sh`: repete o smoke sem rede em DOIS
  runtimes mínimos sem Qt, Rust, CMake ou compiladores — Ubuntu 22.04 (o piso
  prometido) e Debian 12 (o baseline do builder);
- `scripts/verificar-piso-appimage.sh`: mede no AppDir (ou num `.AppImage`
  pronto) o teto de símbolo versionado e a ausência de dependência pendurada;
- `packaging/appimage/kinein-portable-graphics-hook.sh`: integra-se ao AppRun
  gerado pelo linuxdeploy e seleciona por padrão a adaptação `software` oficial
  do Qt Quick, sem depender do EGL/GL do host;
  `KINEIN_GRAPHICS_BACKEND=hardware` mantém aceleração como opt-in;
- `packaging/appimage/`: builders fixados, metadados do desktop e avisos de
  licença.

Build nativo e build no baseline não compartilham cache CMake: usam diretórios
`native-host` e `native-portable` distintos e configuração `cmake --fresh`.
Isso impede que um cache criado em `/workspace` dentro do container seja
reutilizado pelo checkout em `/home/...` (ou o inverso). O wrapper portátil
invoca os scripts montados com `bash`, sem depender do bit executável, e os
mounts Podman usam rótulo SELinux `:Z` com modo de acesso explícito.

O worker não é uma interface de build nativo. Distribuições e versões do Qt
podem representar o backend Wayland por nomes diferentes — por exemplo, o
builder Qt 6.4 fornece `libqwayland-egl.so` e `libqwayland-generic.so`, enquanto
ambientes mais recentes podem fornecer apenas `libqwayland.so`. Obrigar toda
entrada pública a passar pelo builder fixado evita validar um layout no host e
publicar outro no baseline suportado.

Além do `SHA256SUMS` interno de automação, a entrega principal de `dist/` é:
AppImage, `.AppImage.sha256`, `instalar-kinein-vectis.sh` e `Tutorial.md`. O
teste local exige os quatro, exige o instalador executável e compara a cópia do
tutorial byte a byte com a fonte vigente.

Esses arquivos são montados integralmente em staging. Somente após AppImage,
checksum, instalador e tutorial passarem pelas verificações locais cada arquivo
é publicado por renomeação em `dist/`; uma falha anterior à publicação preserva
a última entrega válida e não deixa um binário parcial com nome definitivo.

O AppImage é dono do desktop id `kinein-vectis.desktop` e do nome público
**Kinein Vectis**. O checkout usa o desktop id separado
`kinein-vectis-development.desktop`, exibido como
**Kinein Vectis (Desenvolvimento)**; assim o mantenedor pode comparar o
artefato distribuído com a build local sem um atalho sobrescrever o outro.

### Piso de compatibilidade — revisado em 2026-08-21 por decisão do autor

**A decisão anterior era:** baseline Debian 12/glibc 2.36, sem promessa de
compatibilidade com distribuições mais antigas.

**A decisão vigente é:** o artefato deve rodar em **Ubuntu 22.04 LTS em
diante**, em distribuições baseadas ou não em Ubuntu. Formalmente, o contrato é
**glibc ≥ 2.35 e libstdc++ ≥ GCC 12 (`GLIBCXX_3.4.30`)**, em x86_64.

Isso cobre Ubuntu 22.04+, Debian 12+, Fedora 36+ e openSUSE Leap 15.6+. Fica
**fora**: glibc anterior a 2.35 (RHEL 9, Rocky 9, AlmaLinux 9 e Amazon Linux
2023 têm 2.34), musl e ARM64 — cada um exige outro artefato e outra validação.

O pedido do autor citava "kernel 6.x ou superior". A restrição que de fato
decide se o binário carrega é a **glibc**, não o kernel: o kernel GA do Ubuntu
22.04 é 5.15 (6.x apenas via HWE), e "toda distribuição com kernel 6.x" não é
satisfazível neste piso — o Amazon Linux 2023 tem kernel 6.1 e glibc 2.34.
Por isso o contrato é expresso em glibc/libstdc++, que é verificável.

O builder continua sendo Debian 12 (glibc 2.36), **um degrau acima do piso**.
Isso é seguro apenas porque é verificado: `scripts/verificar-piso-appimage.sh`
varre todo ELF do AppDir e **reprova o empacotamento** se algum exigir
`GLIBC > 2.35`, `GLIBCXX > 3.4.30` ou `CXXABI > 1.3.13`. Sem esse portão a
compatibilidade seria acidente — a medição de 2026-08-21 mostrou que o
artefato de 2026-07-19 cabia no piso por sorte, e nada impedia uma dependência
nova de quebrá-lo em silêncio.

Se o portão reprovar, a correção **não** é subir o teto: é mover o builder para
uma base jammy com Qt 6.4+ de fonte auditada (o jammy distribui Qt 6.2.4 e
`ui/CMakeLists.txt` exige 6.4) e repetir toda a validação. Subir o teto sem
mover o piso quebraria o contrato com quem já baixou o artefato.

Compatibilidade de distribuição e compatibilidade gráfica são limites
separados. Bibliotecas Qt/QML seguem dentro do AppImage, mas EGL, Vulkan e os
drivers da GPU pertencem ao host e não formam uma ABI portátil única. Como a
UI vigente é 2D e não usa efeitos dependentes de shader, o launcher distribuído
escolhe a adaptação raster `software` do Qt Quick. Ela funciona em Wayland e
X11 sem criar um contexto 3D. Aceleração é uma preferência reversível, nunca
pré-requisito para abrir a IDE.

## Auditoria

Os três projetos auditados usam licença MIT e têm builds automatizados. A
inspeção local dos commits não encontrou telemetria, analytics, upload de
conteúdo ou cliente de rede em runtime.

As ferramentas executam subprocessos por finalidade: linuxdeploy chama
`ldd`, `patchelf`, `strip` e plugins; o plugin Qt chama `qmake` e
`qmlimportscanner`. Essa capacidade fica confinada ao container e ao AppDir
gerado. A rede da receita baixa os três assets fixados, antes de validar cada
checksum. linuxdeploy e seu plugin não são empacotados no produto; o runtime
type-2 auditado é necessariamente o cabeçalho executável do AppImage.

O plugin Qt coleta os avisos de copyright/licença disponíveis na distribuição
de build. A Kinein instala também seu aviso de licença no artefato.

Os módulos Debian `QtQuick`, `QtQuick.Window`, `QtQml`, `QtQml.Models` e
`QtQml.WorkerScript` são dependências explícitas do builder. Isso evita que um
build bem-sucedido gere um bundle sem imports QML que só falharia na máquina
do testador. O smoke confere `libqxcb.so`, `offscreen`, `minimal`, plugins de
plataforma Wayland, integrações gráficas/shell Wayland, suas dependências ELF e
os diretórios QML.

## Evidência de validação inicial

Em 2026-07-14, o artefato `Kinein-Vectis-0.1.0-x86_64.AppImage` foi gerado com
33.573.368 bytes e SHA256
`5e8ee15779dfad1fef14c46c335e42c4ebce36f86dd824e9cd0f8486e68a7210`.

Passaram:

1. validação AppStream e do arquivo `.desktop`;
2. checksum, extração e presença de UI/core/manual/licenças/Qt/QML;
3. `core.ping` pelo `kinein-core` empacotado;
4. primeiro frame offscreen no host Arch;
5. o mesmo smoke, com rede desligada, no runtime
   `debian:12-slim@sha256:63a496b5d3b99214b39f5ed70eb71a61e590a77979c79cbee4faf991f8c0783e`,
   contendo apenas a pilha gráfica/fontes esperada de um desktop e nenhuma
   instalação de Qt, Rust, CMake ou compilador.

Esse teste prova independência do ambiente de desenvolvimento, não suporte
literal a qualquer versão histórica de Linux. A matriz Ubuntu/Fedora e um
canal automatizado de release continuam trabalho futuro.

Ainda em 2026-07-14, a separação da documentação pública moveu instalação e
distribuição para `Tutorial.md` e deixou o `MANUAL.md` empacotado restrito ao
uso da IDE. O AppImage `0.1.0` foi reempacotado, preservou 33.573.368 bytes e
passou novamente pelos smokes do host e do Debian mínimo. Seu SHA256 atual é
`ef5970f322aced46905c66dbca5f1360964cde3a4871030bb8f321d771277fe7`; o hash
acima permanece registrado apenas como evidência do primeiro build.

Em 2026-07-15, após endurecer plugins Wayland, separar caches host/container e
adicionar o instalador/tutorial à entrega, o artefato `0.1.0` foi regenerado
com 33.737.208 bytes e SHA256
`fd5fe934599757b6980703d2c2529f9b50bdc026e03e5da34b6a0eb5f44629b9`.
Passaram o smoke do host e o Debian mínimo sem rede. O teste também executou o
instalador a partir de um diretório de trabalho diferente da entrega e
confirmou `.desktop`, PNG extraído e `Exec` apontando para o AppImage correto;
isso protege a regra de resolver por padrão a pasta do próprio script.

Ainda em 2026-07-15, a entrada pública foi protegida contra execução acidental
do worker no Qt do host e a publicação passou a usar staging. Uma falha forçada
antes do container preservou byte a byte os cinco arquivos existentes em
`dist/`. A reconstrução seguinte gerou 33.737.208 bytes, SHA256
`86b335b2b1ba8c81d958df4f1e45f7d9c0838fdad2a2567c84199f84b8dbdc0d`,
sem staging residual, e passou novamente pelos smokes do host e do Debian 12
mínimo sem rede.

No primeiro uso real desse artefato no Fedora/Wayland, o `wayland-egl` foi
encontrado, mas o RHI não conseguiu criar um contexto OpenGL e abortou. A
correção integrou ao AppRun o hook de renderer portátil e tornou o smoke capaz
de rejeitar a ausência dessa política. A entrega regenerada tem 33.749.496
bytes e SHA256
`6c1a3c24a2b13ac36509ec615971eea75d604d36855f601726e083aa8af00312`.
Ela passou no Debian 12 mínimo sem rede e, sem override gráfico, no desktop
Fedora/Wayland real, onde confirmou `Loading backend software` e o primeiro
frame em 939 ms pelo modo de extração e 987 ms pela montagem type-2 usada pelo
atalho, sem abortar.

Em 2026-08-21, com o piso revisado, o artefato de 2026-07-19 foi **medido e
executado** no runtime mínimo Ubuntu 22.04 (`glibc 2.35`), sem rede:

1. teto de símbolo do bundle: `GLIBC_2.35`, `GLIBCXX_3.4.30`, `CXXABI_1.3.13`
   — no limite exato do piso, nenhum acima;
2. `kinein-core` empacotado respondeu `core.ping` no jammy;
3. o Qt/QML abriu o primeiro frame offscreen em **692 ms**, selecionando
   `Loading backend software`, com código de saída 0;
4. o runtime type-2 embutido é `static-pie` com libfuse3 ligada estaticamente:
   **não depende de libfuse2**, que é a causa mais comum de AppImage não abrir
   em Ubuntu 24.04 e posteriores.

O mesmo exame encontrou um defeito real de empacotamento, corrigido no mesmo
gesto: `wayland-shell-integration/libwl-shell-plugin.so` exigia
`libQt6WlShellIntegration.so.6`, **ausente do AppImage**. A causa era dupla e
vale como lição sobre validação: os grupos de plugin Wayland entram por `cp -a`
depois do linuxdeploy, que não reanalisa o que não instalou; e a checagem de
dependência usava `ldd` **dentro do builder**, onde o Qt do sistema está
instalado — uma biblioteca ausente do AppDir mas presente em `/usr/lib`
resolvia e passava. Um gate que faz a pergunta errada ("o linker acha?" em vez
de "está dentro do pacote?") fica verde sobre um artefato quebrado. A validação
passou a cobrir todos os plugins presentes no AppDir, e não uma lista escrita à
mão de quatro.

Ainda em 2026-08-21, o AppImage foi **regenerado** sob o piso novo, por decisão
do autor. O artefato vigente tem 36.514.296 bytes e SHA256
`8a833e76818718420a2c97d4dce00ccc268e6c43c01884affddf6de284a34f7b`, e o
`kinein-core` empacotado responde no protocolo `0.63.0` — o do HEAD.

Passou, na ordem em que a receita cobra:

1. o fechamento de dependência funcionou na produção: a receita empacotou
   `libQt6WlShellIntegration.so.6`, exigida por `libwl-shell-plugin.so`, que
   faltava no artefato anterior;
2. a validação de dependência dinâmica cobriu **27 plugins**, e não os quatro
   de uma lista escrita à mão;
3. o piso de compatibilidade aprovou os **98 ELFs** do AppDir — nenhum acima
   do teto de símbolo, nenhuma dependência pendurada;
4. o smoke portátil passou **nos dois runtimes**, Ubuntu 22.04 e Debian 12,
   sem rede e sem Qt/Rust/CMake instalados.

O artefato anterior (2026-07-19) reprovava o smoke estrutural por um motivo
legítimo, e ele fica registrado porque a verificação fez o seu trabalho: o
`dist/Tutorial.md` era anterior ao commit `98a7e47`, e a receita compara a
cópia byte a byte com a fonte. Entrega desatualizada não passa por acidente.

## Alternativas consideradas

- **Bundler próprio:** rejeitado por duplicar resolução ELF/RPATH/Qt/QML.
- **linuxdeployqt:** rejeitado; linuxdeploy é o sucessor flexível recomendado
  pelo próprio projeto e separa o suporte Qt em plugin.
- **Qt CMake Deployment API + CPack:** opção futura para pacotes DEB/RPM, mas
  não substitui sozinha o artefato AppImage e exige Qt 6.5 para o fluxo Linux
  documentado atualmente.
- **Pacote Arch/PKGBUILD:** útil como alternativa local, porém não atende o
  objetivo entre distribuições.

## Verificação e rollback

O artefato só pode ser distribuído após:

1. SHA256 das ferramentas e do runtime type-2 validado;
2. AppDir conter UI, core, manual, desktop, Qt, módulos QML e plugins `xcb`,
   `offscreen`, `minimal` e Wayland, inclusive integrações gráficas e de shell
   e suas dependências dinâmicas;
3. `core.ping` responder usando o binário empacotado;
4. primeiro frame Qt/QML aparecer no smoke offscreen;
5. o smoke confirmar que o AppRun limpo seleciona `Loading backend software`;
6. `dist/` conter AppImage, checksum específico, instalador e `Tutorial.md`
   idêntico à fonte;
7. `scripts/testar-appimage-portatil.sh` passar sem rede e sem Qt/Rust de
   desenvolvimento, **nos dois runtimes**: Ubuntu 22.04 (o piso prometido) e
   Debian 12 (o baseline do builder);
8. `scripts/verificar-piso-appimage.sh` aprovar o AppDir — nenhum ELF acima do
   teto de símbolo e nenhuma dependência pendurada. Ele roda dentro do
   `empacotar-appimage.sh` como último portão antes de existir arquivo
   distribuível, e também aceita um `.AppImage` já pronto como argumento.

Rollback: remover a receita/instalação AppDir e as duas entradas do registry.
O build normal por Cargo/CMake não depende dessas ferramentas.
