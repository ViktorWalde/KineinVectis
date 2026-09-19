# 44 — Etapa 3: a arquitetura do frontend (tool windows à JetBrains, adaptadas)

> Escrito em 2026-09-19 a pedido do autor, depois do teste dele na IDE com a
> Etapa 2 fechada: *"faz a arquitetura dessa otimização do frontend"*. É
> desenho ANTES do código, como o `43`. Cada fatia tem a medida; a ordem
> está no §7. Depois desta etapa vem a **integração profunda com os
> compiladores** (§8), anotada aqui como a etapa seguinte.

## 1. O que o autor viu (2026-09-19, a lista, na ordem dele)

1. **Banco:** na caixa "Novo banco", o chip "MongoDB em container" não
   aparece inteiro (os quatro chips numa `Row` de ~400 px); e não dá para
   **remover/excluir** um banco criado pela tela.
2. **Containers:** funciona; falta polimento de UX.
3. **Grafana/Observabilidade:** a HUD precisa de melhoria de UI/UX (o
   autor testa num projeto real depois).
4. **Trilho lateral ("rodapé esquerdo") e "Ferramentas":**
   - "Busca" no trilho não é o que ele quer ali. Ele lembra da **aba
     escondida no lado direito** que listava as funções do arquivo e
     pesquisava por nome — mais prática; quer isso de volta, e que busque
     **funções do projeto inteiro e da pasta aberta**, para projetos grandes.
   - **Git:** a HUD nova está ótima, mas o posicionamento não é memória
     muscular JetBrains — quer o Git pelo ícone do trilho, **vertical, no
     lado esquerdo**, como as IDEs JetBrains.
   - **Build e Debug** no trilho repetem os botões do canto superior
     direito — remover os repetidos; usar o trilho para o que importa
     (PlatformIO/embarcados; distribuir o que está concentrado em
     "Ambiente").
   - **Embarcados:** polimento de HUD/UI/UX — dimensionamento e o que é
     intuitivo.
5. Depois do frontend: **integração profunda com os compiladores** e a
   exibição disso ao usuário, por confiabilidade.
6. Pergunta: dá para divulgar a versão atual para testes? (§9)

## 2. O modelo que se adota — e o que NÃO se copia

O que a referência (JetBrains) tem e o autor sente falta é o **modelo de
tool windows**: faixas nas bordas com botões que abrem janelas ancoradas
àquela borda; a memória muscular é "Git é à esquerda, em pé; Structure é à
direita, escondida; Terminal/Build/Problems embaixo; Run/Debug em cima".
O que se adota:

```text
ESQUERDA (trilho)          Projeto · Git (janela VERTICAL à esquerda) · Embarcados ·
                           Banco · Containers · Grafana · Ferramentas
                           (saem: Busca, Build, Debug — repetidos em cima/embaixo)
DIREITA (aba escondida)    Símbolos: a estrutura do arquivo (o que já existe) +
                           busca por nome no projeto inteiro e na pasta aberta
EMBAIXO (abas)             Terminal · Build · Problemas · Testes · Jobs · Debug ·
                           Busca (no projeto, texto) · Ferramentas · IDE
                           (sai: Git — foi para a esquerda)
EM CIMA (barra)            Projeto · Git (branch/↑↓/N) · Executar (▶ 🐞 …) — F1
ATALHO GLOBAL              Search Everywhere (Shift duplo / Ctrl+P): arquivos,
                           comandos, #símbolos — já existe
```

O que não se copia: o tema, os ícones, a densidade de cada painel da
referência; o painel Git em pé é ADAPTADO (o diff abre no editor, como lá,
mas a lista de mudanças por pasta e o commit ficam juntos, sem a árvore de
changelists). Sem código nem assets da JetBrains (licença e identidade).

## 3. O diagnóstico medido (2026-09-19, headless, este repositório)

- **Trilho hoje:** Projeto · Busca · Git · Build e jobs · Debug · Banco ·
  Containers · Observabilidade · Ferramentas — nove; três (Busca, Build,
  Debug) repetem o que a barra ou a faixa de abas já dão; Embarcados só
  pelo menu Ambiente. O modo expandido (0.128.0) mostra os rótulos.
- **"Novo banco":** `DataSourceCreateBox` põe quatro `KvToggleChip` numa
  `Row` — a coluna direita do painel tem ~400 px a 720 de largura; o quarto
  chip fica fora (o autor viu). Não há caminho de remoção: `datasource.
  remove` apaga só o PERFIL; o arquivo `.sqlite`, o container ou o banco no
  servidor ficam.
- **Símbolos à direita:** `EditorOutlinePanel` + `EditorOutlineHandle` (a
  "aba escondida") existem desde a etapa 11 — a estrutura do arquivo com
  filtro; `outlineCollapsed` fica `true` sozinho abaixo de 1180 px (é por
  isso que "sumiu" para o autor em janelas menores). Busca por nome no
  projeto: `lsp.workspaceSymbols` (por `#` no Search Everywhere) e
  `index.symbols` (o índice, sem LSP) existem; nenhum dos dois mora nessa aba.
- **Git:** a HUD (43 §9.4) é horizontal no painel de baixo, 340 px; em pé
  ela pede outra composição: lista em cima, commit embaixo, o diff FORA
  (no editor).
- **Embarcados:** nove seções empilhadas numa coluna de 560 px (F8 fez
  rolar; não fez caber): Projeto, Sonda, Portas, Gravar, Kit, Chip/Alvo/
  Sysroot/SVD, Tamanho, Depurador, Instalar, Sysroot/SDK.
- **Containers:** lista com cinco ações por linha; imagens na grade;
  motor como veredito. Falta: filtro por nome, a linha selecionada com
  as ações num só lugar, logs/shell sem exigir projeto quando possível.
- **Grafana:** formulário + veredito + achados numa coluna; sem o
  cabeçalho comum; sem "abrir no navegador" por achado; sem a grade.

## 4. As fatias, cada uma com o que muda e a medida

```text
E3-1  Banco: os chips cabem; remover o que foi criado           contrato 0.129.0
      - DataSourceCreateBox: Row -> Flow (os chips quebram linha); a caixa
        cresce; foto.
      - datasource.destroy { name, data: bool }: com data=false so' o perfil
        (o que remove faz hoje); com data=true, por motor: SQLite apaga o
        arquivo (so' dentro do workspace; recusa fora); containerServer para
        e remove o container `kinein-<name>` (job High, comando visivel);
        banco DENTRO de um PostgreSQL = DROP DATABASE pelo query confirmado
        (a tela compoe, como o CREATE). O dialogo diz o que vai sumir.
      Medida: testes de despacho (SQLite real; podman falso com `rm -f`);
      harness; foto do dialogo de remocao.

E3-2  Simbolos a direita (a aba escondida volta, e busca no projeto)   sem contrato
      - A aba direita vira "Simbolos": [estrutura do arquivo] + campo de
        busca; com texto, a lista mostra simbolos do PROJETO (index.symbols,
        sem LSP, instantaneo) e da PASTA do arquivo aberto (filtro por
        prefixo do caminho), com o arquivo:linha; Enter abre. O LSP
        (workspaceSymbols) entra como segunda fonte quando o servidor esta'
        `running`, deduplicado por arquivo:linha.
      - `outlineCollapsed` deixa de virar true sozinho por largura (o autor
        perdeu a aba por isso); a aba escondida (handle) fica sempre, o
        painel abre por clique/atalho (Alt+7 como a referencia? — o atalho
        entra no command.list e no gate de atalhos).
      - O icone "Busca" sai do trilho; a busca por TEXTO no projeto segue
        na aba de baixo (Ctrl+Shift+F) e no Search Everywhere.
      Medida: harness do controller de simbolos (fontes, dedupe, pasta);
      foto com um `fn` procurado.

E3-3  Git a esquerda, em pe'                                            sem contrato
      - Uma janela lateral esquerda (ao lado do trilho, no lugar do
        explorer OU sobre ele — decisao: ALTERNA com o explorer, como a
        referencia alterna Project/Commit): abas Commit | Log em cima.
        Commit: branch/pull/push/stash numa linha; a lista de mudancas por
        pasta com checkbox de pasta; a caixa de commit (Amend, Commit e
        Push, Commit) no pe'. Log: filtro (texto, branch), a lista com o
        grafo e os refs.
      - O diff/o commit selecionado abre NO EDITOR como uma aba de
        visualizacao ("diff: caminho" / "commit abc1234"), read-only, com o
        GitPatchView e o cabecalho do inspetor; fecha como aba.
      - O trilho: o icone Git abre/fecha essa janela; a aba Git do painel de
        baixo SAI; `git.commit`/`git.log`/`git.branches` da paleta apontam
        para a janela.
      Medida: fotos (Commit e Log em pe' a 1280 e a 1024); harness do
      controller da janela (qual aba, alterna com explorer); o gate de
      atalhos.

E3-4  O trilho e o menu Ambiente                                        sem contrato
      - Saem do trilho: Busca (E3-2), Build, Debug. Entram: Embarcados (o
        painel de E3-5). Ordem: Projeto · Git · Embarcados · Banco ·
        Containers · Grafana · Ferramentas; o modo expandido mostra os
        rotulos; atalhos no tooltip.
      - O menu Ambiente fica como esta' (e' o catalogo completo); o trilho
        e' o acesso rapido.
      Medida: foto; `verificar-atalhos` verde; nenhum sinal orfao.

E3-5  Embarcados: a HUD                                                  sem contrato
      - O painel vira ABAS dentro da moldura comum: Placa (portas, sonda,
        identidade, permissoes) · Projeto (framework, modelo, tamanho) ·
        Gravar (motor, previa, firmware) · Kit (chip/alvo/sysroot/SVD,
        depurador, importar, instalar). O cabecalho comum com o veredito
        da placa ("ESP32-D0WD-V3 em /dev/ttyUSB0" ou "nenhuma placa").
        Cada aba cabe sem rolar a 800 px; a moldura fixa em 640x560.
      - PlatformIO como cidadao: quando `platformio.ini` existe, a aba
        Projeto mostra os ambientes do `pio` e o Gravar usa o `pio run -t
        upload` (ja' existe no bloco E; falta a tela dizer).
      Medida: fotos das quatro abas com a placa do autor plugada (so' ele)
      e sem placa (headless); harness do controller de abas.

E3-6  Containers: polimento                                               sem contrato
      - Filtro por nome; a linha selecionada e as acoes numa barra (em vez
        de cinco icones por linha); "abrir logs/shell" habilitado sem
        projeto quando o core aceitar cwd vazio (medir; senao, dizer o
        porque no tooltip como hoje); a grade para os containers tambem.
      Medida: foto; harness.

E3-7  Grafana: polimento                                                  sem contrato
      - KvPanelHeader (Sondar como primaria), o veredito comum, achados na
        grade com "abrir no navegador" por linha, o token com o mesmo
        desenho de "de onde vem a senha" do banco.
      Medida: foto; o autor testa num Grafana real (ele tem um projeto).
```

## 5. As restrições que valem

Contrato primeiro (só a E3-1 tem); catracas (view 300 / controller-host
400 / Rust 500 / ui/src 500) — o `GitController` está em 398: a janela
lateral do Git ganha um controller próprio (`GitWindowController`), e o
que hoje é o `GitPanel` do painel de baixo vira a janela; `ShellWorkspaceHost`
em 343 e `BottomPanelHost` em 298 têm folga para perder o Git e ganhar a
janela. Medir antes de afirmar; registro datado + evidências por fatia; um
commit por fatia; nunca `git checkout <arquivo>`; nada de push/release sem
o autor.

## 6. O que o autor mede (e o agente não consegue)

O clique real em cada uma das sete; a placa nos Embarcados; um Grafana
real; um PostgreSQL/Mongo real no "Novo banco" (o `run` baixa a imagem).

## 7. A ordem

E3-1 (banco — o que ele viu primeiro e é backend) → E3-2 (símbolos à
direita) → E3-3 (Git em pé) → E3-4 (trilho) → E3-5 (embarcados) → E3-6
(containers) → E3-7 (grafana). Documentação sincronizada a cada fatia;
esta página ganha o "feito" de cada uma no §7.1.

### 7.1 Feito

(vazio em 2026-09-19 de manhã)

## 8. A etapa seguinte, anotada: integração profunda com os compiladores

Decisão do autor (2026-09-19): depois do frontend, **a integração profunda
com os compiladores e a exibição disso ao usuário**, por polimento e
confiabilidade. O que isso significa aqui, para desenhar quando chegar:
o modelo de compilação por arquivo (a CDB do file-api, os flags reais)
exposto na tela (por que este arquivo compila assim; qual toolchain, qual
sysroot, quais defines); os diagnósticos do COMPILADOR (não só do LSP)
com a linha de comando que os produziu; a saída de build estruturada
(alvo → passo → erro) em vez de texto; o tempo por alvo; o cache/ccache
dito; o "mesmo erro" entre compilador e LSP casado (a dedup da F6-b
generalizada); o cross-compile explicado (o kit, o alvo, o que falta). A
fundação existe (`project.model`, `index.context`, `cmake.*`, `cargo.*`,
`quality.*`); a etapa é ligar tudo à tela com confiabilidade medida.

## 9. "Posso divulgar a versão atual para testes?" — a resposta honesta

**Ainda não como versão pública; sim como teste fechado com você e uma
ou duas pessoas que reportem.** O porquê, medido nos últimos dois dias:

- Três defeitos de **quinze dias** só apareceram quando alguém OLHOU a tela
  (oito domínios sem resposta, §7.64; o painel Git sem nomes, §7.67; a
  lista de variáveis do depurador sem tamanho, §7.72). Os gates ficaram
  verdes o tempo todo. Antes de divulgar, o roteiro do `40` §4.2.6 tem de
  ser clicado por alguém que não escreveu a IDE.
- O que está provado só com **falsos**: PostgreSQL/Mongo, a Pi, os SDKs,
  a sonda JTAG. Um testador vai bater nisso no primeiro dia.
- A **distribuição**: o AppImage em `dist/` é de antes da Etapa 2 (0.1.0);
  regenerar e passar o `testar-appimage.sh` é uma sessão; instalar limpo
  numa máquina sem Qt é o teste que falta.
- **Documentação de usuário**: o `manual.md` acompanhou, mas não tem um
  "comece aqui em 5 minutos" nem o que fazer quando algo falha.

**O critério de "divulgável para teste" que se propõe** (para você
decidir depois destas correções): (1) as sete fatias desta etapa feitas e
clicadas por você; (2) um AppImage novo que abre numa máquina limpa;
(3) o roteiro do `40` §4.2.6 passado por uma pessoa de fora, com os
achados corrigidos; (4) um `COMECE-AQUI.md` de uma página; (5) um canal
para reportar (issues) e a versão dita na tela (`Ajuda › Sobre`). Quando
os cinco estiverem, a resposta muda para "sim".
