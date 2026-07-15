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
  Docker;
- `scripts/empacotar-appimage.sh`: compila, instala no AppDir, verifica as
  ferramentas e gera o AppImage;
- `scripts/testar-appimage.sh`: valida estrutura, core e primeiro frame
  offscreen;
- `scripts/testar-appimage-portatil.sh`: repete o smoke sem rede em um runtime
  Debian mínimo sem Qt, Rust, CMake ou compiladores;
- `packaging/appimage/`: builders fixados, metadados do desktop e avisos de
  licença.

O baseline atual é Debian 12/glibc 2.36. Portanto, o primeiro artefato cobre
distribuições Linux x86_64 atuais com glibc igual ou posterior; não se promete
compatibilidade binária com distribuições mais antigas que o baseline. Para
ampliar essa faixa será necessário construir Qt 6.4+ sobre uma base anterior e
repetir toda a validação.

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
do testador. O smoke confere também `xcb`, `offscreen` e os diretórios QML.

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
2. AppDir conter UI, core, manual, desktop, Qt, módulos QML e plugins `xcb` e
   `offscreen`;
3. `core.ping` responder usando o binário empacotado;
4. primeiro frame Qt/QML aparecer no smoke offscreen;
5. `scripts/testar-appimage-portatil.sh` passar sem rede e sem Qt/Rust de
   desenvolvimento.

Rollback: remover a receita/instalação AppDir e as duas entradas do registry.
O build normal por Cargo/CMake não depende dessas ferramentas.
