# Tutorial para instalar, atualizar e distribuir o Kinein Vectis

Este guia foi escrito para quem recebeu a Kinein Vectis e nunca viu o projeto
por dentro. Siga as seções na ordem: confira se o computador é compatível,
verifique o download, instale o atalho e só depois abra seus projetos. A parte
final é reservada a quem gera e distribui novas versões.

O AppImage é um executável portátil para Linux: ele já contém a interface, o
core, o Qt/QML e os plugins gráficos necessários. O uso das funções da IDE,
depois que ela estiver aberta, está no [MANUAL.md](MANUAL.md).

## 1. O que enviar aos seus amigos agora

Na pasta `dist/`, envie estes quatro arquivos juntos:

```text
Kinein-Vectis-0.1.0-x86_64.AppImage
Kinein-Vectis-0.1.0-x86_64.AppImage.sha256
instalar-kinein-vectis.sh
Tutorial.md
```

É possível compartilhar os arquivos por armazenamento em nuvem, anexo, mídia física
ou outro canal que aceite arquivos binários. Não é necessário enviar o
repositório nem pedir que o testador compile a IDE.

`SHA256SUMS` também contém o checksum correto, mas é destinado à automação e a
possíveis releases com vários artefatos. Para uma pessoa recebendo um único
AppImage, o arquivo com o mesmo nome e sufixo `.sha256` é menos ambíguo.

Antes de enviar, confira o par na sua máquina:

```bash
cd dist
sha256sum -c Kinein-Vectis-0.1.0-x86_64.AppImage.sha256
```

O resultado esperado termina em `OK`. Se quiser confirmar a origem além da
integridade, envie também o texto do hash por um segundo canal. Um checksum
recebido junto do binário detecta corrupção, mas não prova autoria se ambos
forem substituídos pelo mesmo atacante.

O terceiro arquivo cria o ícone no menu de aplicativos. Ele também atualiza um
atalho existente para a versão mais recente e pergunta se as versões antigas
devem ser apagadas. A pessoa ainda pode executar somente o AppImage, sem usar o
instalador. O quarto arquivo é esta cópia autônoma e atualizada do tutorial,
para que instalação, atualização, rollback e diagnóstico continuem disponíveis
sem acesso ao repositório.

## 2. Conferir se o computador é compatível

- Linux x86_64 (`uname -m` deve mostrar `x86_64`);
- glibc 2.36 ou posterior;
- sessão desktop Wayland ou X11 e fontes normais.

Windows, ARM64 e distribuições baseadas somente em musl não estão cobertos por
este artefato. A IDE abre sem Rust, Qt, CMake, compiladores ou aceleração 3D
instalados: o AppImage usa por padrão o renderer raster oficial do Qt Quick,
evitando depender da combinação EGL/Mesa/NVIDIA presente na distribuição.

## 3. Verificar o download

Coloque os dois arquivos recebidos na mesma pasta, abra um terminal nela e
rode:

```bash
sha256sum -c Kinein-Vectis-0.1.0-x86_64.AppImage.sha256
```

Só prossiga se aparecer:

```text
Kinein-Vectis-0.1.0-x86_64.AppImage: OK
```

Se aparecer `FAILED`, não execute o arquivo. Apague o AppImage e o checksum,
baixe ambos novamente e repita a verificação. Confira também se o navegador
não renomeou um deles acrescentando `(1)` ou outro sufixo.

## 4. Instalar o ícone e abrir pelo menu

Mantenha o AppImage, o checksum, `instalar-kinein-vectis.sh` e `Tutorial.md`
juntos na mesma pasta. Uma pasta pessoal permanente, como
`~/Applications/KineinVectis`, é preferível a Downloads: o atalho aponta para
o AppImage naquele local.

Abra um terminal nessa pasta e execute:

```bash
chmod +x instalar-kinein-vectis.sh
./instalar-kinein-vectis.sh
```

O instalador não usa `sudo`. Ele procura todos os AppImages da Kinein na pasta
onde o próprio script está (ou no caminho passado como argumento),
compara os números de versão, dá permissão de execução ao mais recente e cria
um único item chamado **Kinein Vectis** no menu de aplicativos. Ao atualizar,
esse mesmo item é sobrescrito e passa a abrir a versão mais recente; portanto,
não ficam vários ícones acumulados.

Se houver versões anteriores na pasta, o script pergunta se você quer
apagá-las. Responda `s` para remover os AppImages antigos e seus checksums ou
pressione Enter para conservá-los. Essa escolha não muda o atalho: ele sempre
aponta para a versão mais recente encontrada.

Depois, procure por **Kinein Vectis** no menu de aplicativos e abra normalmente.

### Alternativa: executar sem instalar o ícone

```bash
chmod +x Kinein-Vectis-0.1.0-x86_64.AppImage
./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Não use `sudo`. Se mover o AppImage depois de criar o ícone, execute novamente
`instalar-kinein-vectis.sh` na nova pasta para atualizar o caminho do atalho.

O AppImage leva a UI, o core e o runtime Qt necessários. Configurações globais
ficam em `~/.config/kinein-vectis/`; sessões e rascunhos de um projeto ficam na
pasta `.kinein/` dentro do próprio workspace.

O modo portátil prioriza compatibilidade. Para testar aceleração gráfica no
desktop atual, sem mudar permanentemente o atalho, execute:

```bash
KINEIN_GRAPHICS_BACKEND=hardware ./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Se houver qualquer erro de EGL, OpenGL, Vulkan ou RHI, volte a abrir pelo menu
ou sem essa variável. Variáveis Qt explícitas como `QT_QUICK_BACKEND` e
`QSG_RHI_BACKEND` são respeitadas para diagnóstico avançado.

## 5. Ferramentas opcionais dos projetos

Essas ferramentas não são necessárias apenas para abrir a Kinein. Instale
somente as que correspondem ao trabalho do testador, usando o gerenciador de
pacotes da distribuição ou a fonte oficial da ferramenta:

| Uso | Ferramentas externas recomendadas |
| --- | --- |
| C/C++ | Clang ou GCC, CMake, Ninja, clangd, clang-format e LLDB/GDB |
| Rust | rustup/Cargo, rust-analyzer, rustfmt, Clippy e LLDB |
| Busca e Git | ripgrep, fd/fdfind e Git |
| Agentes de IA (opcional) | Claude Code, Codex ou similar — rodam no terminal, como qualquer programa |

A aba **Ferramentas** e o banner **Project Health** mostram o que foi detectado
e o que falta para o projeto aberto. A Kinein não instala, autentica ou envia
dados para essas ferramentas silenciosamente.

## 6. Atualizar para uma versão nova

Quando você enviar uma versão nova, o testador deve:

1. baixar o novo AppImage, o novo `.sha256`, o instalador e o tutorial atualizados;
2. verificar o checksum antes de executar;
3. salvar o trabalho e fechar a versão antiga;
4. colocar os quatro arquivos na mesma pasta da versão anterior;
5. executar `./instalar-kinein-vectis.sh`;
6. escolher se quer apagar as versões antigas quando o script perguntar;
7. abrir a Kinein pelo mesmo ícone do menu e testar o workspace.

Se quiser uma possibilidade imediata de rollback, responda `N` à remoção até
validar a nova versão. Os AppImages podem coexistir porque cada nome contém a
versão, mas existe sempre apenas um ícone: ao rodar o instalador ele aponta para
a maior versão disponível. Para voltar atrás, remova ou mova a versão nova e
execute o instalador novamente; ele passará a apontar para a anterior.

O checksum é específico de cada build. Nunca reutilize o `.sha256` de uma
versão anterior e nunca altere o nome ou o conteúdo citado dentro dele sem
gerar o hash novamente.

## 7. Solução de problemas de execução

### O ícone não apareceu ou abre uma versão movida

Execute `./instalar-kinein-vectis.sh` novamente na pasta onde estão os
AppImages. Em alguns ambientes gráficos pode ser necessário encerrar e entrar
novamente na sessão para o menu atualizar.

### `Permission denied`

Repita o `chmod +x`. Se o arquivo estiver numa partição montada com `noexec`,
mova o AppImage para uma pasta no diretório pessoal, como `~/Applications/`, e
tente novamente.

### Erro relacionado a FUSE

Use o modo de extração temporária, que dispensa a montagem FUSE:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Esse modo pode abrir um pouco mais devagar, mas não instala nada no sistema.

### A janela não abre

Execute pelo terminal, copie toda a saída e envie junto do relato:

```bash
./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Se o aplicativo chegou a abrir, inclua também o log
`~/.cache/kinein-vectis/logs/kinein-ui-erros.txt`, a distribuição usada e a
descrição do que aconteceu.

## 8. Para mantenedores: gerar uma nova versão portátil

Esta seção é para quem tem acesso autorizado ao repositório privado. O método
usa um builder Debian 12 em Podman ou Docker, evitando que o AppImage dependa
da distribuição instalada no host.

### 8.1 Preparação

Na raiz do projeto:

1. confira `git status --short` e preserve mudanças não relacionadas;
2. atualize a versão em `project(kinein-vectis VERSION ...)`, no
   `CMakeLists.txt` da raiz;
3. atualize a mesma versão e a data em
   `packaging/appimage/io.github.viktorwalde.KineinVectis.appdata.xml`;
4. confirme que `README.md`, `MANUAL.md` e este `Tutorial.md` representam a
   versão que será entregue;
5. tenha Podman ou Docker instalado e funcionando.

O primeiro build precisa de rede para obter a imagem e os assets de packaging
fixados. Os downloads de `linuxdeploy`, do plugin Qt e do runtime AppImage são
aceitos somente quando seus SHA-256 coincidem com os pins auditados.

### 8.2 Empacotar

```bash
bash scripts/empacotar-appimage-portatil.sh
```

Esse é o comando canônico. Por segurança, a chamada mais curta abaixo também
encaminha automaticamente para o mesmo builder Debian 12:

```bash
bash scripts/empacotar-appimage.sh
```

Não tente executar a etapa interna com `--baseline-worker` no host. Os nomes e
a disposição dos plugins Qt Wayland variam entre versões e distribuições; o
contrato auditado da entrega é o ambiente fixado pelo script portátil.

O script:

- compila o `kinein-core` em release;
- compila e instala a UI Qt/QML no AppDir;
- inclui runtime Qt, plugins, manual e licenças;
- gera `dist/Kinein-Vectis-<versão>-x86_64.AppImage`;
- gera `SHA256SUMS` e o `.AppImage.sha256` correspondente;
- copia `dist/instalar-kinein-vectis.sh`, responsável pelo único atalho do
  usuário;
- copia o `Tutorial.md` vigente para `dist/`, sem depender da árvore-fonte na
  entrega.

O conjunto completo é preparado em staging e só então publicado em `dist/`.
Falha de compilação, plugin, validação ou geração preserva a última entrega
válida. Um build concluído pode substituir um artefato da **mesma versão**; se
quiser conservar também esse build anterior após uma reconstrução bem-sucedida,
copie o par AppImage/checksum para outro local antes de repetir o mesmo número.

### 8.3 Validar no host e no baseline portátil

```bash
bash scripts/testar-appimage.sh
bash scripts/testar-appimage-portatil.sh
```

O primeiro teste verifica checksum, estrutura do bundle, resposta do core e o
primeiro frame offscreen. O segundo repete o smoke sem rede em um Debian 12
mínimo, sem Qt, Rust, CMake ou compiladores de desenvolvimento.

Depois, entre em `dist/` e confira também o checksum específico da nova versão:

```bash
cd dist
VERSION=0.2.0
sha256sum -c "Kinein-Vectis-${VERSION}-x86_64.AppImage.sha256"
```

Troque `0.2.0` pelo número real da nova versão. A release só está pronta quando
os dois testes e essa verificação terminarem com sucesso.

### 8.4 Entregar

Envie o executável, seu checksum, o instalador e o tutorial:

```text
Kinein-Vectis-<versão>-x86_64.AppImage
Kinein-Vectis-<versão>-x86_64.AppImage.sha256
instalar-kinein-vectis.sh
Tutorial.md
```

Não edite o AppImage depois que o checksum foi gerado. Qualquer alteração exige
novo hash e nova validação.

## 9. Para mantenedores: entrega do código a terceiros

O repositório de desenvolvimento não deve ser tornado público. Quando houver
necessidade de fornecer o código, gere outra árvore/cópia por allowlist. Essa
cópia contém o código do projeto e, entre Markdown, **somente**:

```text
README.md
MANUAL.md
Tutorial.md
```

Não inclua `.git/` nem o histórico do repositório privado. Também não inclua
`docsprivate/AGENTS.md`, `docsprivate/ContextoIA.md`, `docsprivate/GUIAIA.md`, `docsprivate/PONTO_ATUAL.md`, `docs/`,
`prompts/`, roadmaps, specs ou outras notas de agentes. Antes de entregar, o
futuro exportador deve oferecer dry-run, rejeitar Markdown extra, auditar
segredos e mostrar a lista final de arquivos. Até esse exportador existir, não
monte a cópia externa por exclusões improvisadas e não altere a visibilidade do
repositório-fonte. Um eventual espelho deve começar com histórico próprio,
criado a partir da árvore sanitizada.

## 10. Onde continuar a leitura

- `README.md`: apresenta o projeto e seu estado atual;
- `MANUAL.md`: ensina exclusivamente a usar a IDE;
- `Tutorial.md`: ensina distribuição, instalação, atualização e geração do
  executável portátil.
