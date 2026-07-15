# Tutorial de distribuição e instalação do Kinein Vectis

Este tutorial cobre tudo que acontece **fora da IDE**: enviar o AppImage para
testadores, verificar sua integridade, executá-lo sem depender da distribuição,
instalar ferramentas opcionais, atualizar manualmente e gerar uma nova versão.
O uso dos recursos da IDE está no [MANUAL.md](MANUAL.md).

## 1. O que enviar aos seus amigos agora

Na pasta `dist/`, envie estes dois arquivos juntos:

```text
Kinein-Vectis-0.1.0-x86_64.AppImage
Kinein-Vectis-0.1.0-x86_64.AppImage.sha256
```

É possível compartilhar o par por armazenamento em nuvem, anexo, mídia física
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

## 2. Como o testador verifica e executa

### 2.1 Requisitos do computador

- Linux x86_64 (`uname -m` deve mostrar `x86_64`);
- glibc 2.36 ou posterior;
- sessão desktop com pilha gráfica e fontes normais.

Windows, ARM64 e distribuições baseadas somente em musl não estão cobertos por
este artefato. A IDE abre sem Rust, Qt, CMake ou compiladores instalados.

### 2.2 Verificar o download

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

### 2.3 Dar permissão e abrir

```bash
chmod +x Kinein-Vectis-0.1.0-x86_64.AppImage
./Kinein-Vectis-0.1.0-x86_64.AppImage
```

Não use `sudo`. O AppImage é portátil e deve rodar com a conta normal do
usuário. Ele pode permanecer na pasta de downloads ou ser movido para uma
pasta pessoal, por exemplo `~/Applications/`.

O AppImage leva a UI, o core e o runtime Qt necessários. Configurações globais
ficam em `~/.config/kinein-vectis/`; sessões e rascunhos de um projeto ficam na
pasta `.kinein/` dentro do próprio workspace.

## 3. Ferramentas opcionais dos projetos

Essas ferramentas não são necessárias apenas para abrir a Kinein. Instale
somente as que correspondem ao trabalho do testador, usando o gerenciador de
pacotes da distribuição ou a fonte oficial da ferramenta:

| Uso | Ferramentas externas recomendadas |
| --- | --- |
| C/C++ | Clang ou GCC, CMake, Ninja, clangd, clang-format e LLDB/GDB |
| Rust | rustup/Cargo, rust-analyzer, rustfmt, Clippy e LLDB |
| Busca e Git | ripgrep, fd/fdfind e Git |
| KV Context | Claude CLI ou Codex CLI previamente instalado e autenticado |

A aba **Ferramentas** e o banner **Project Health** mostram o que foi detectado
e o que falta para o projeto aberto. A Kinein não instala, autentica ou envia
dados para essas ferramentas silenciosamente.

## 4. Atualização manual segura

Quando você enviar uma versão nova, o testador deve:

1. baixar o novo AppImage e o novo `.sha256` para uma pasta separada;
2. verificar o checksum antes de executar;
3. salvar o trabalho e fechar a versão antiga;
4. dar permissão de execução ao AppImage novo e abri-lo;
5. testar a abertura do workspace, um arquivo e as funções principais;
6. manter o AppImage antigo por alguns dias para rollback.

Não sobrescreva nem apague a versão antiga antes de validar a nova. Como cada
versão tem o número no nome, elas podem coexistir. Para voltar atrás, feche a
nova e execute o AppImage anterior; os arquivos do projeto não ficam dentro do
executável.

O checksum é específico de cada build. Nunca reutilize o `.sha256` de uma
versão anterior e nunca altere o nome ou o conteúdo citado dentro dele sem
gerar o hash novamente.

## 5. Solução de problemas de execução

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

## 6. Gerar uma nova versão portátil

Esta seção é para quem tem acesso autorizado ao repositório privado. O método
usa um builder Debian 12 em Podman ou Docker, evitando que o AppImage dependa
da distribuição instalada no host.

### 6.1 Preparação

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

### 6.2 Empacotar

```bash
bash scripts/empacotar-appimage-portatil.sh
```

O script:

- compila o `kinein-core` em release;
- compila e instala a UI Qt/QML no AppDir;
- inclui runtime Qt, plugins, manual e licenças;
- gera `dist/Kinein-Vectis-<versão>-x86_64.AppImage`;
- gera `SHA256SUMS` e o `.AppImage.sha256` correspondente.

Ele pode substituir um artefato da **mesma versão**. Se quiser preservar um
build anterior para rollback, copie o par AppImage/checksum para outro local
antes de reconstruir com o mesmo número.

### 6.3 Validar no host e no baseline portátil

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

### 6.4 Entregar

Envie somente o novo par:

```text
Kinein-Vectis-<versão>-x86_64.AppImage
Kinein-Vectis-<versão>-x86_64.AppImage.sha256
```

Não edite o AppImage depois que o checksum foi gerado. Qualquer alteração exige
novo hash e nova validação.

## 7. Entrega do código a terceiros

O repositório de desenvolvimento não deve ser tornado público. Quando houver
necessidade de fornecer o código, gere outra árvore/cópia por allowlist. Essa
cópia contém o código do projeto e, entre Markdown, **somente**:

```text
README.md
MANUAL.md
Tutorial.md
```

Não inclua `.git/` nem o histórico do repositório privado. Também não inclua
`AGENTS.md`, `ContextoIA.md`, `GUIAIA.md`, `PONTO_ATUAL.md`, `docs/`,
`prompts/`, roadmaps, specs ou outras notas de agentes. Antes de entregar, o
futuro exportador deve oferecer dry-run, rejeitar Markdown extra, auditar
segredos e mostrar a lista final de arquivos. Até esse exportador existir, não
monte a cópia externa por exclusões improvisadas e não altere a visibilidade do
repositório-fonte. Um eventual espelho deve começar com histórico próprio,
criado a partir da árvore sanitizada.

## 8. Responsabilidade dos três documentos públicos

- `README.md`: apresenta o projeto e seu estado atual;
- `MANUAL.md`: ensina exclusivamente a usar a IDE;
- `Tutorial.md`: ensina distribuição, instalação, atualização e geração do
  executável portátil.
