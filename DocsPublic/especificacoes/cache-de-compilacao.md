# Cache de compilação — o que já existe no mercado, orquestrado

> **Classe: PLANO** (`../README.md`). Descreve o alvo, não o estado. Nada
> disto existe no código em 2026-09-26.
>
> **Origem:** relatório de pesquisa do autor (`deep-research-report.md`,
> entregue em 2026-09-26), quebrado em partes nesta data. O autor está
> preparando uma documentação técnica mais detalhada; este documento é o
> esqueleto **medido**, e cede lugar a ela onde as duas divergirem.
>
> **Fase: 0.4, junto dos embarcados — decisão do autor em 2026-09-26.**
> Nada disto entra na 0.3, pela mesma regra que mantém o trilho e os
> embarcados fora dela.

## 1. O que foi medido nesta máquina, em 2026-09-26

Medir antes de escrever é a regra do projeto, e aqui ela já mudou duas
afirmações do relatório de origem.

```text
ccache      4.12.3 instalado
            features: avx2 file-storage http-storage redis+unix-storage redis-storage
            licença GPL-3+ (declarada em /usr/share/doc/ccache/copyright)
sccache     não instalado
distcc      não instalado
icecc       não instalado
o projeto   nenhum COMPILER_LAUNCHER em CMakePresets.json, cmake/ ou CMakeLists.txt
            o cache local está vazio: os builds da Kinein não passam por ele
```

**A compile database NÃO carrega o launcher.** Medido com CMake + Ninja num
projeto mínimo: com `-DCMAKE_CXX_COMPILER_LAUNCHER=ccache`, a entrada do
`compile_commands.json` começa em `/usr/bin/c++`, e não em `ccache`. Isso
importa mais aqui do que na média dos projetos: o `clangd` da IDE lê aquele
arquivo, e um launcher vazando nele seria um defeito de navegação, não de
build.

**O cache serviu o objeto, de verdade.** Primeiro build: `cache_miss 1`.
Apagado o objeto e reconstruído: `direct_cache_hit 1`, com o *miss* parado em
1 — o compilador não rodou de novo.

**O ccache 4.12 já faz armazenamento remoto** (`http-storage`,
`redis-storage`). O relatório de origem separa "ccache local / sccache
remoto"; essa separação não corresponde ao que está instalado aqui. A escolha
entre os dois passa a ser sobre **Rust** e sobre distribuir *compilação*
(sccache faz; ccache não), e não sobre guardar em rede.

## 2. A forma: processo orquestrado, nunca biblioteca

O `ccache` é **GPL-3+**. Isso não é um problema — e a razão de não ser tem de
estar escrita, porque a pergunta volta:

```text
a IDE INVOCA o ccache como processo, exatamente como invoca cmake, ninja,
gdb e clangd. Não linka, não embute, não redistribui dentro do AppImage.
```

É a mesma relação que o projeto já tem com todo o resto da caixa de
ferramentas, e a regra que a sustenta é a do `integracoes/README.md`:
*importar invariante, modo de falha e estratégia de teste — nunca código*.
Linkar seria outra conversa, e não é esta.

**O que a IDE nunca faz:** instalar o cache sozinha, ligá-lo sem a pessoa
saber, ou esconder que um build veio de cache.

## 3. As partes

### C1 — o pronto do mercado (a primeira fatia)

O núcleo: **detectar → oferecer → medir**.

- `CacheProvider` no core, com um descritor tipado por ferramenta encontrada
  (nome, versão, caminho, o que ela suporta). Mesma disciplina do domínio de
  toolchain, que já responde "qual executável cumpre cada papel".
- A escolha é **por kit**, e explícita: a IDE propõe, a pessoa liga. Um build
  que muda de comportamento sozinho é o oposto do que este projeto faz.
- A ligação é `CMAKE_<LANG>_COMPILER_LAUNCHER` na configuração do preset —
  nada de `PATH` remendado, nada de symlink em `/usr/local/bin`.
- Para Rust, o caminho é o `RUSTC_WRAPPER`, e ele é do sccache: o ccache não
  compila Rust.
- **Estatística visível**: acertos, erros e tamanho, lidos do próprio
  `ccache --print-stats` (formato estável e legível por máquina, medido em
  2026-09-26) — nunca estimados pela IDE.

**Aceite:** ligar o cache num projeto C++ real reduz o tempo do segundo build
**e a IDE mostra o número que a ferramenta reporta**, não um número dela. O
`compile_commands.json` continua começando no compilador.

### C2 — cross e embarcado, que é onde dói

Compilação cruzada é lenta, e é justamente onde um cache errado envenena em
silêncio: um objeto de outro `--sysroot` compila, linka e roda errado na
placa.

- A identidade do cache tem de incluir **compilador, alvo e sysroot**. O
  ccache já compõe a sua chave a partir do comando e dos arquivos de entrada;
  o que falta é o projeto **provar** isso para os kits cross que ele mesmo
  instala (ARM GCC, `probe-rs`, o QEMU do gate).
- Teste de veneno, não de conveniência: mesmo fonte, dois sysroots, e o
  objeto do segundo **não** pode vir do primeiro.

**Aceite:** o teste de veneno existe e reprova quando se remove o que o evita.

### C3 — impacto de build

Responder *"o que eu mudei, e o que isso obriga a recompilar"* — antes de
compilar.

- A fonte é determinística e já existe: a **File API do CMake** e os
  `depfiles` que o Ninja consome.
- Sem heurística e sem adivinhação: é leitura de grafo.

**Aceite:** a lista prevista bate com a lista que o Ninja de fato recompilou,
num projeto real, medido.

### C4 — armazenamento compartilhado

Guardar o cache fora da máquina (equipe ou CI). O ccache faz por HTTP/Redis; o
sccache faz por S3 e distribui compilação.

Isto **abre superfície**: rede, credencial, cota, e a possibilidade de um
objeto de outra máquina entrar no seu build. Não entra sem decisão explícita
do autor, e quando entrar vem com o que a `seguranca/` já exige.

## 4. O que NÃO entra por este documento

Pular compilação por análise própria — diff de AST, *fingerprints* semânticas,
modo sombra. Isso é **pesquisa**, mora em [`../pesquisa/`](../pesquisa/README.md)
e não é alvo desta fase. A separação é deliberada: um documento persuasivo
sobre algo não decidido, guardado junto das especificações, é lido como alvo
pela próxima sessão — foi exatamente o que criou o `DocsPrivate/legado/`.

## 5. Decisões que são do autor

1. **Licença do projeto.** O relatório de origem recomenda MPL-2.0; o
   repositório é `MIT OR Apache-2.0`. Trocar a licença de um projeto é decisão
   de dono, não consequência de escolher uma ferramenta de build — e nada
   nesta especificação depende dela, porque a IDE **invoca** o cache em vez de
   linkar.
2. **Se C4 entra**, e com que política de rede.
3. **Se o sccache entra** já na primeira fatia (ele traz Rust) ou depois.
