#!/usr/bin/env python3
"""Sonda de SOAK do core: a IDE aguenta um dia de uso? (roadmap 30, etapa 7)

# Por que existe

A suite passa em centenas de testes CURTOS. Nenhum deles responde a pergunta
que separa "IDE que funciona" de "IDE que substitui a sua": **o que acontece
depois de horas?** As falhas desta classe nao aparecem em teste curto por
definicao — elas se acumulam:

    vazamento de MEMORIA      cache que so cresce, buffer que nunca sai
    vazamento de DESCRITOR    pipe/PTY/arquivo que ninguem fecha
    vazamento de THREAD       sessao encerrada cuja thread continua viva
    processo ORFAO            filho que sobrevive a quem o criou
    DEGRADACAO                a mesma requisicao fica mais lenta com o tempo

Esta sonda dirige o binario REAL por JSON-RPC em ciclos de trabalho realista e
mede as cinco coisas acima ao longo do tempo. Ela nao afirma "nao vaza"; ela
afirma "**nao vaza alem deste orcamento**, medido" — e imprime a serie para um
humano ver a tendencia, porque um numero final sozinho esconde a forma da curva.

# O que ela NAO faz, e por que

Nao entra no `verificar.sh`. Um soak util demora minutos; gate que demora e'
gate que se desliga. Ela e' sonda, como a `sonda_drafts.py` e a
`sonda_terminal.py`: rodada sob demanda, e antes de afirmar "substituicao
diaria".

Nao chama `lsp.*` diretamente, mas **um language server pode subir mesmo assim**
— e isso e realista, nao acidente: `fs.read` de um `.cpp`/`.rs` faz `did_open`,
e o core sobe o clangd/rust-analyzer se ele existir na maquina (medido em
2026-09-02: o filho que aparece na serie e o `clangd.main`). Isso NAO polui o
orcamento, porque o RSS medido e o do CORE, nao o do servidor; o que o servidor
acrescenta ao core e uma thread leitora e um par de descritores, UMA vez, dentro
do aquecimento. O que o cliente LSP vaza se mede com o servidor falso, em
`src/tests/lsp_server.rs`.

As superficies in-process cobertas aqui: Tree-sitter (cache LRU de 32 buffers),
a store SQLite de rascunhos, as sessoes de terminal (3 threads e um PTY cada),
a caminhada de busca e o dispatch.

# Isolamento

O binario real liga a persistencia GLOBAL, entao `XDG_CONFIG_HOME` vai para um
diretorio temporario — rodar a prova nao pode sujar os dados do autor.

Uso:
    python3 scripts/sonda_soak.py               # ~120 ciclos, cerca de 1 min
    python3 scripts/sonda_soak.py --ciclos 2000 # soak longo, de verdade
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def escolher_core():
    """Prefere o binario RELEASE; cai para o debug com aviso.

    Medido em 2026-09-02: um `syntaxTree.update` num arquivo de 400 linhas leva
    ~2.100 ms no debug e ~220 ms no release — 10x. Soak que mede o debug mede
    outra coisa: "aguenta um dia de uso" e uma afirmacao sobre o binario que o
    usuario roda, e o AppImage leva o release.
    """
    release = os.path.join(REPO, "target", "release", "kinein-core")
    debug = os.path.join(REPO, "target", "debug", "kinein-core")
    if os.path.exists(release):
        return release, "release"
    return debug, "debug"


CORE, PERFIL = escolher_core()

# Ciclos iniciais que NAO contam para o orcamento: e' quando os caches enchem,
# as gramaticas Tree-sitter carregam e o SQLite abre. Crescer aqui e' o
# esperado; o vazamento e' o que cresce DEPOIS.
AQUECIMENTO = 20

# Orcamentos. Sao generosos de proposito: a sonda existe para pegar vazamento
# (crescimento sem teto), nao para brigar por kilobytes.
ORCAMENTO_RSS_KB = 32 * 1024   # 32 MiB de crescimento apos o aquecimento
# Abaixo deste crescimento total, a FORMA da curva e ruido e nao se julga.
PISO_TENDENCIA_KB = 1024
ORCAMENTO_FD = 8               # descritores a mais no fim do que no aquecimento
ORCAMENTO_THREADS = 4          # threads a mais
ORCAMENTO_FILHOS = 1           # filhos a mais (0 = nem um terminal vazado)
FATOR_LATENCIA = 4.0           # quanto a p95 do fim pode piorar sobre o inicio


class Core:
    """O core real, falando JSON-RPC por stdio."""

    def __init__(self, xdg):
        env = dict(os.environ, XDG_CONFIG_HOME=xdg)
        self.proc = subprocess.Popen(
            [CORE], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            text=True, bufsize=1, env=env,
        )
        self.id = 0

    def rpc(self, method, params):
        """Manda um pedido e devolve (resposta, segundos)."""
        self.id += 1
        alvo = self.id
        inicio = time.monotonic()
        self.proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": alvo, "method": method, "params": params}) + "\n")
        self.proc.stdin.flush()
        while True:
            linha = self.proc.stdout.readline()
            if not linha:
                raise RuntimeError(f"o core morreu esperando {method}")
            msg = json.loads(linha)
            if msg.get("id") == alvo:
                return msg, time.monotonic() - inicio

    def amostra(self):
        """RSS (KiB), descritores, threads e filhos vivos AGORA."""
        pid = self.proc.pid
        rss = 0
        with open(f"/proc/{pid}/status", encoding="utf-8") as fh:
            for linha in fh:
                if linha.startswith("VmRSS:"):
                    rss = int(linha.split()[1])
                    break
        return {
            "rss_kb": rss,
            "fds": len(os.listdir(f"/proc/{pid}/fd")),
            "threads": len(os.listdir(f"/proc/{pid}/task")),
            "filhos": len(filhos_de(pid)),
        }


def filhos_de(pid):
    """PIDs VIVOS cujo pai e `pid`.

    Zumbi nao conta: ele ja saiu e esta so esperando o `wait()` da thread
    waiter da sessao. Conta-lo faria a sonda reprovar por uma corrida de
    milissegundos em vez de por vazamento — e gate que grita falso ensina a
    ignorar (`ARCHITECTURE.md` §4 regra 11).
    """
    achados = []
    for entrada in os.listdir("/proc"):
        if not entrada.isdigit():
            continue
        try:
            with open(f"/proc/{entrada}/stat", encoding="utf-8") as fh:
                campos = fh.read().rsplit(")", 1)
            depois = campos[1].split()
            estado, pai = depois[0], int(depois[1])
        except (OSError, IndexError, ValueError):
            continue
        if pai == pid and estado != "Z":
            achados.append(int(entrada))
    return achados


def vivos(pids):
    """Quais destes PIDs ainda existem e nao sao zumbis."""
    restantes = []
    for pid in pids:
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8") as fh:
                estado = fh.read().rsplit(")", 1)[1].split()[0]
        except (OSError, IndexError):
            continue
        if estado != "Z":
            restantes.append(pid)
    return restantes


# Arquivos distintos por onde os ciclos giram. Precisa ser MAIOR que o cache
# LRU de documentos do Tree-sitter (32, `lang/service.rs`): com dois arquivos so,
# um cache SEM TETO cresceria igual a um com teto, e a sonda nao teria como
# distinguir os dois — mediria o proprio workload, nao o produto.
ARQUIVOS_EM_RODIZIO = 48


def projeto(raiz):
    """Um projeto pequeno, porem realista: CMake + Cargo + fontes."""
    os.makedirs(os.path.join(raiz, "src"), exist_ok=True)
    # Arquivos de tamanho REALISTA (~500 linhas). Fonte de brinquedo faria a
    # sonda medir o proprio workload: um cache sem teto de 48 arquivos minusculos
    # cabe no orcamento e passaria despercebido.
    for indice in range(ARQUIVOS_EM_RODIZIO):
        corpo_cpp = "".join(
            f"int f{indice}_{linha}(int x) {{ return x + {linha}; }}\n"
            for linha in range(200)
        )
        escrever(raiz, f"src/modulo{indice}.cpp",
                 f"#include <string>\n#include <vector>\n{corpo_cpp}")
        corpo_rs = "".join(
            f"pub fn f{indice}_{linha}(x: i32) -> i32 {{ x + {linha} }}\n"
            for linha in range(200)
        )
        escrever(raiz, f"src/modulo{indice}.rs", corpo_rs)
    escrever(raiz, "CMakeLists.txt",
             "cmake_minimum_required(VERSION 3.24)\nproject(soak CXX)\n"
             "add_executable(soak src/main.cpp)\n")
    escrever(raiz, "Cargo.toml", '[package]\nname = "soak"\nversion = "0.1.0"\n'
                                 'edition = "2021"\n\n[dependencies]\n')
    escrever(raiz, "src/main.cpp",
             "#include <vector>\nint main() { std::vector<int> v; return v.size(); }\n")
    escrever(raiz, "src/lib.rs",
             "pub fn soma(a: i32, b: i32) -> i32 { a + b }\n")


def escrever(raiz, relativo, conteudo):
    caminho = os.path.join(raiz, relativo)
    with open(caminho, "w", encoding="utf-8") as fh:
        fh.write(conteudo)
    return caminho


def ciclo(core, raiz, n):
    """Um ciclo de trabalho realista. Devolve as latencias medidas."""
    latencias = []
    # Rodizio: cada ciclo edita um arquivo DIFERENTE, para que um cache sem
    # teto se distinga de um cache com teto.
    indice = n % ARQUIVOS_EM_RODIZIO
    cpp = os.path.join(raiz, f"src/modulo{indice}.cpp")
    rs = os.path.join(raiz, f"src/modulo{indice}.rs")

    def chamar(metodo, params, tolera_erro=False):
        resposta, segundos = core.rpc(metodo, params)
        if not tolera_erro and resposta.get("error"):
            raise RuntimeError(f"{metodo} falhou no ciclo {n}: {resposta['error']}")
        latencias.append(segundos)
        return resposta

    # Ler e editar: Tree-sitter re-parseia a cada versao.
    chamar("fs.read", {"path": cpp})
    with open(cpp, encoding="utf-8") as fh:
        original = fh.read()
    texto = original + f"// edicao do ciclo {n}\n"
    chamar("syntaxTree.update", {"path": cpp, "content": texto, "version": n})
    with open(rs, encoding="utf-8") as fh:
        original_rs = fh.read()
    chamar("syntaxTree.update", {"path": rs,
                                 "content": original_rs + f"// ciclo {n}\n",
                                 "version": n})

    # Autosave e limpeza: a store SQLite abre, grava e apaga.
    chamar("draft.save", {"path": cpp, "content": texto})
    chamar("draft.clear", {"path": cpp})

    # Busca no projeto: caminhada de arquivos a cada volta.
    chamar("fs.search", {"query": "int", "caseSensitive": False})
    chamar("fs.list", {"path": raiz})

    # Terminal: abre, usa e FECHA. Cada sessao sao 3 threads e um PTY —
    # a superficie de vazamento mais afiada do core.
    aberto = chamar("terminal.open", {}, tolera_erro=True)
    sessao = (aberto.get("result") or {}).get("id")
    if sessao:
        chamar("terminal.input", {"id": sessao, "data": f"echo ciclo {n}\n"})
        chamar("terminal.resize", {"id": sessao, "cols": 100, "rows": 30})
        chamar("terminal.close", {"id": sessao})

    # Consultas baratas que a UI faz o tempo todo.
    chamar("cmake.status", {})
    chamar("job.list", {})
    chamar("configAction.list", {})
    chamar("workspace.saveSession", {"openFiles": [cpp, rs], "activeFile": cpp})
    return latencias


def desatualizado():
    """O binario e mais velho que alguma fonte do core ou do protocolo?"""
    binario = os.path.getmtime(CORE)
    for base in ("crates/kinein-core/src", "crates/kinein-protocol/src"):
        for pasta, _, arquivos in os.walk(os.path.join(REPO, base)):
            for arquivo in arquivos:
                if not arquivo.endswith(".rs"):
                    continue
                if os.path.getmtime(os.path.join(pasta, arquivo)) > binario:
                    return True
    return False


def tendencia(serie):
    """Quanto o ultimo terco cresceu em relacao ao terco do meio.

    ~0 significa curva que assentou (cache cheio). ~1 significa que ainda sobe
    no mesmo ritmo — a assinatura de vazamento. `None` quando ha amostras de
    menos para afirmar qualquer coisa.
    """
    if len(serie) < 4:
        return None
    corte = len(serie) // 3
    if corte < 1:
        return None
    meio = serie[len(serie) - 2 * corte - 1][1]["rss_kb"] - serie[corte - 1][1]["rss_kb"]
    fim = serie[-1][1]["rss_kb"] - serie[len(serie) - corte - 1][1]["rss_kb"]
    if meio <= 0:
        return None if fim <= 0 else float(fim)
    return fim / meio


def p95(valores):
    if not valores:
        return 0.0
    ordenados = sorted(valores)
    return ordenados[min(len(ordenados) - 1, int(len(ordenados) * 0.95))]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ciclos", type=int, default=120)
    argumentos = parser.parse_args()

    if not os.path.exists(CORE):
        print(f"erro: {CORE} ausente. Rode: cargo build --release -p kinein-core",
              file=sys.stderr)
        return 2
    if PERFIL == "debug":
        print("aviso: usando o binario DEBUG — a latencia medida NAO e a do produto.")
        print("       Para a medicao que vale: cargo build --release -p kinein-core")

    # Sonda contra binario VELHO prova o passado. A armadilha e conhecida e ja
    # esta escrita (`arquitetura/04` §8), mas nada a checava: durante a
    # construcao desta sonda, uma mutacao que NAO COMPILOU deixou o binario
    # antigo no lugar e o soak passou verde "provando" um codigo que nao
    # existia. Recusar aqui e mais barato do que descobrir depois.
    if desatualizado():
        print(f"erro: {CORE} e mais VELHO que as fontes.", file=sys.stderr)
        print(f"      Rode: cargo build {'--release ' if PERFIL == 'release' else ''}"
              "-p kinein-core", file=sys.stderr)
        return 2
    if argumentos.ciclos <= AQUECIMENTO:
        print(f"erro: --ciclos precisa ser maior que o aquecimento ({AQUECIMENTO})",
              file=sys.stderr)
        return 2

    temporario = tempfile.mkdtemp(prefix="kinein-soak-")
    raiz = os.path.join(temporario, "projeto")
    os.makedirs(raiz)
    projeto(raiz)
    core = Core(os.path.join(temporario, "xdg"))
    falhas = []

    try:
        resposta, _ = core.rpc("workspace.open", {"path": raiz})
        if resposta.get("error"):
            raise RuntimeError(f"workspace.open falhou: {resposta['error']}")

        serie = []
        latencia_inicio = []
        latencia_fim = []
        base = None

        print(f"== soak: {argumentos.ciclos} ciclos, binario {PERFIL} "
              f"(aquecimento: {AQUECIMENTO}) ==")
        for n in range(1, argumentos.ciclos + 1):
            latencias = ciclo(core, raiz, n)
            if n <= AQUECIMENTO:
                pass
            elif n <= AQUECIMENTO + (argumentos.ciclos - AQUECIMENTO) // 4:
                latencia_inicio.extend(latencias)
            elif n > argumentos.ciclos - (argumentos.ciclos - AQUECIMENTO) // 4:
                latencia_fim.extend(latencias)

            if n == AQUECIMENTO:
                base = core.amostra()
                serie.append((n, base))
            elif n % max(1, argumentos.ciclos // 8) == 0 or n == argumentos.ciclos:
                if n == argumentos.ciclos:
                    # A thread waiter da ultima sessao pode ainda estar
                    # colhendo o shell recem-fechado. Medir vazamento no meio
                    # do reaper e medir a corrida, nao o vazamento.
                    time.sleep(0.5)
                serie.append((n, core.amostra()))

        print(f"{'ciclo':>7} {'RSS(KiB)':>10} {'fds':>5} {'threads':>8} {'filhos':>7}")
        for n, amostra in serie:
            print(f"{n:>7} {amostra['rss_kb']:>10} {amostra['fds']:>5} "
                  f"{amostra['threads']:>8} {amostra['filhos']:>7}")

        fim = serie[-1][1]
        delta_rss = fim["rss_kb"] - base["rss_kb"]
        delta_fd = fim["fds"] - base["fds"]
        delta_threads = fim["threads"] - base["threads"]

        print()
        if delta_rss > ORCAMENTO_RSS_KB:
            falhas.append(f"MEMORIA cresceu {delta_rss} KiB apos o aquecimento "
                          f"(orcamento {ORCAMENTO_RSS_KB} KiB)")
        else:
            print(f"  ok   memoria dentro do orcamento  [+{delta_rss} KiB]")

        # O TETO sozinho nao basta: um vazamento lento passa por ele num soak
        # curto e derruba a IDE num dia de uso. O que separa vazamento de cache
        # e a FORMA — cache estabiliza, vazamento nao. So se julga a forma
        # quando o crescimento ja saiu do ruido.
        forma = tendencia(serie)
        if delta_rss > PISO_TENDENCIA_KB and forma is not None and forma > 0.5:
            falhas.append(
                f"MEMORIA nao estabilizou: o ultimo terco cresceu {forma:.2f}x o "
                "que o terco do meio cresceu. Cache tem teto e assenta; "
                "vazamento continua subindo."
            )
        elif delta_rss > PISO_TENDENCIA_KB and forma is not None:
            print(f"  ok   memoria assentou  [ultimo terco = {forma:.2f}x o do meio]")

        if delta_fd > ORCAMENTO_FD:
            falhas.append(f"DESCRITORES cresceram {delta_fd} (orcamento {ORCAMENTO_FD}) "
                          "— pipe, PTY ou arquivo que ninguem fecha")
        else:
            print(f"  ok   descritores estaveis  [{base['fds']} -> {fim['fds']}]")

        if delta_threads > ORCAMENTO_THREADS:
            falhas.append(f"THREADS cresceram {delta_threads} "
                          f"(orcamento {ORCAMENTO_THREADS}) — sessao encerrada "
                          "cuja thread continua viva")
        else:
            print(f"  ok   threads estaveis  [{base['threads']} -> {fim['threads']}]")

        # O que importa e o CRESCIMENTO. Um language server vivo e' correto (um
        # por linguagem, mantido de proposito); um terminal por ciclo que nunca
        # morre e' vazamento. So o segundo faz a contagem subir.
        delta_filhos = fim["filhos"] - base["filhos"]
        if delta_filhos > ORCAMENTO_FILHOS:
            falhas.append(f"PROCESSOS FILHOS cresceram {delta_filhos} "
                          f"(orcamento {ORCAMENTO_FILHOS}) — sessao de terminal ou "
                          "job que nao morre")
        else:
            print(f"  ok   filhos estaveis  [{base['filhos']} -> {fim['filhos']}]")

        p95_inicio, p95_fim = p95(latencia_inicio), p95(latencia_fim)
        if p95_inicio > 0 and p95_fim > p95_inicio * FATOR_LATENCIA:
            falhas.append(f"DEGRADACAO: p95 foi de {p95_inicio * 1000:.1f}ms para "
                          f"{p95_fim * 1000:.1f}ms (fator {p95_fim / p95_inicio:.1f}x, "
                          f"limite {FATOR_LATENCIA}x)")
        else:
            print(f"  ok   latencia estavel  [p95 {p95_inicio * 1000:.1f}ms -> "
                  f"{p95_fim * 1000:.1f}ms]")

        # Encerramento limpo: nada pode ficar para tras.
        #
        # Os filhos sao anotados ANTES do shutdown de proposito. Depois que o
        # core morre, os filhos dele sao reparentados para o init — perguntar
        # "quem e filho do core agora?" devolveria vazio SEMPRE, e a checagem
        # nunca poderia reprovar. E a mesma armadilha que a `sonda_m43b.py`
        # evita ao guardar os PIDs primeiro.
        antes_do_shutdown = filhos_de(core.proc.pid)
        core.rpc("core.shutdown", {})
        core.proc.wait(timeout=15)
        time.sleep(0.3)
        sobreviventes = vivos(antes_do_shutdown)
        if sobreviventes:
            falhas.append(f"processos orfaos apos o shutdown: {sobreviventes}")
        else:
            print(f"  ok   shutdown limpo, sem orfaos "
                  f"[{len(antes_do_shutdown)} filho(s) antes]")
    finally:
        if core.proc.poll() is None:
            core.proc.kill()
        shutil.rmtree(temporario, ignore_errors=True)

    print()
    if falhas:
        for falha in falhas:
            print(f"  FALHA  {falha}", file=sys.stderr)
        print("\n✗ soak REPROVOU", file=sys.stderr)
        return 1
    print("✓ soak: memoria, descritores, threads, filhos e latencia sob controle")
    return 0


if __name__ == "__main__":
    sys.exit(main())
