#!/usr/bin/env python3
"""Medições de performance do kinein-core via stdio (fatia M4.2).

Chamado por scripts/medir-performance.sh. Mede, tudo LOCAL (sem rede):
  B  workspace.open no repo Kinein (o "workspace grande"), mediana de N.
  C  fs.read de um .txt sintético de 10k linhas (proxy de abrir arquivo
     grande no editor — lado core), mediana de N.
  D2 RSS do core em regime + RSS do LSP Rust (rust-analyzer, best-effort)
     após workspace.open + fs.read de um .rs (que dispara did_open).
  A3.1 Tree-sitter: primeiro `syntaxTree.update` FRIO e atualização
     incremental de 1 caractere no mesmo documento, sobre fixture Rust
     determinística de tamanho explícito. Sem rede, sem LSP. O snapshot é
     validado estruturalmente: número rápido com resposta vazia é o parser
     falhando, não acertando.
  A3.2 LSP: primeira resposta ÚTIL de `lsp.semanticTokens` e `lsp.completion`
     por servidor (rust-analyzer e clangd, separados), em projeto próprio e
     mínimo. Ferramenta ausente => `n/d` explícito. O contrato diz que
     servidor sem suporte responde lista vazia, então espera-se resposta com
     token/item real e `path`/`version` conferidos — a primeira resposta não
     serve. Separa `first` (inclui indexação externa) de `warm` (round-trip da
     Kinein) e mede o RSS do processo externo.

Imprime linhas `chave=valor` para o shell parsear.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time


def spawn(core):
    return subprocess.Popen(
        [core], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, text=True,
    )


def make_rpc(proc):
    state = {"id": 0}

    def rpc(method, params):
        state["id"] += 1
        want = state["id"]
        proc.stdin.write(json.dumps(
            {"jsonrpc": "2.0", "id": want, "method": method, "params": params}) + "\n")
        proc.stdin.flush()
        while True:
            line = proc.stdout.readline()
            if not line:
                raise RuntimeError(f"EOF esperando {method}")
            msg = json.loads(line)
            if msg.get("id") == want:
                return msg
    return rpc


def median_ms(samples):
    s = sorted(samples)
    n = len(s)
    if n == 0:
        return "n/d"
    mid = s[n // 2] if n % 2 else (s[n // 2 - 1] + s[n // 2]) / 2
    return f"{mid:.1f}"


def vmrss_kb(pid):
    try:
        with open(f"/proc/{pid}/status") as fh:
            for ln in fh:
                if ln.startswith("VmRSS:"):
                    return int(ln.split()[1])
    except OSError:
        pass
    return 0


def descendants_rss_kb(pid):
    """Soma o VmRSS dos processos filhos (rust-analyzer é filho do core)."""
    total = 0
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as fh:
                ppid = int(fh.read().split()[3])
        except (OSError, IndexError, ValueError):
            continue
        if ppid == pid:
            total += vmrss_kb(int(entry))
    return total


def measure_workspace_open(core, root, n):
    samples = []
    for _ in range(n):
        proc = spawn(core)
        rpc = make_rpc(proc)
        t0 = time.perf_counter()
        rpc("workspace.open", {"path": root})
        samples.append((time.perf_counter() - t0) * 1000.0)
        proc.stdin.close()
        proc.wait(timeout=5)
    return median_ms(samples)


def measure_fs_read_big(core, n):
    tmp = tempfile.mkdtemp(prefix="kinein-perf-")
    big = os.path.join(tmp, "big.txt")
    with open(big, "w") as fh:
        for i in range(10_000):
            fh.write(f"linha {i:05d} — conteudo sintetico para medir leitura de arquivo grande\n")
    proc = spawn(core)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": tmp})
    samples = []
    for _ in range(n):
        t0 = time.perf_counter()
        rpc("fs.read", {"path": "big.txt"})
        samples.append((time.perf_counter() - t0) * 1000.0)
    proc.stdin.close()
    proc.wait(timeout=5)
    return median_ms(samples)


def fixture_rust(functions):
    """Fixture Rust DETERMINISTICA: mesmo texto byte a byte a cada run.

    Gerada em vez de commitada para o tamanho ser explicito no codigo e nao um
    blob que ninguem revisa. Precisa ser Rust REAL e parseavel — medir o
    Tree-sitter sobre `linha 00001` mediria o parser falhando rapido, nao o
    parser trabalhando. Por isso ha struct, impl, match, generico e closure:
    e o formato de codigo que o editor de fato encontra.
    """
    partes = [
        "use std::collections::HashMap;\n",
        "use std::fmt;\n\n",
    ]
    for i in range(functions):
        partes.append(
            f"#[derive(Debug, Clone, PartialEq)]\n"
            f"pub struct Registro{i} {{\n"
            f"    pub id: u64,\n"
            f"    pub nome: String,\n"
            f"    pub tags: Vec<String>,\n"
            f"    pub indice: HashMap<String, u32>,\n"
            f"}}\n\n"
            f"impl Registro{i} {{\n"
            f"    pub fn novo(id: u64, nome: &str) -> Self {{\n"
            f"        Self {{\n"
            f"            id,\n"
            f"            nome: nome.to_owned(),\n"
            f"            tags: Vec::new(),\n"
            f"            indice: HashMap::new(),\n"
            f"        }}\n"
            f"    }}\n\n"
            f"    pub fn classificar(&self) -> &'static str {{\n"
            f"        match self.id {{\n"
            f"            0 => \"vazio\",\n"
            f"            1..=9 => \"pequeno\",\n"
            f"            _ if self.tags.is_empty() => \"sem tags\",\n"
            f"            _ => \"grande\",\n"
            f"        }}\n"
            f"    }}\n\n"
            f"    pub fn filtrar<F>(&self, predicado: F) -> Vec<&String>\n"
            f"    where\n"
            f"        F: Fn(&String) -> bool,\n"
            f"    {{\n"
            f"        self.tags.iter().filter(|t| predicado(t)).collect()\n"
            f"    }}\n"
            f"}}\n\n"
            f"impl fmt::Display for Registro{i} {{\n"
            f"    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{\n"
            f"        write!(f, \"Registro{i}({{}})\", self.nome)\n"
            f"    }}\n"
            f"}}\n\n"
        )
    return "".join(partes)


def syntax_snapshot_is_valid(result):
    """Um numero rapido pode ser o parser FALHANDO, nao acertando.

    A3.1 exige validacao estrutural: sem isto, uma gramatica ausente devolveria
    resposta vazia em ~0 ms e a tabela mostraria "otimo desempenho".
    """
    if not result or "result" not in result:
        return False, "sem result"
    r = result["result"]
    if r.get("language") != "rust":
        return False, f"linguagem={r.get('language')!r} (esperado rust)"
    if r.get("hasErrors") is True:
        return False, "parse com erros"
    if not r.get("highlights"):
        return False, "highlights vazio"
    if not r.get("outline"):
        return False, "outline vazio"
    return True, ""


def measure_syntax(core, n, functions=60):
    """A3.1: primeiro snapshot FRIO e atualizacao incremental de 1 caractere.

    Frio exige core novo a cada amostra: o core mantem cache LRU de 32 buffers,
    entao a segunda chamada no mesmo path ja seria incremental. Sem PTY, sem
    rede, sem LSP — so Tree-sitter.

    Tamanho da fixture EXPLICITO (`functions`), nao herdado de arquivo do
    repo: arquivo do repo muda e a serie historica perde comparabilidade.

    O numero e o round-trip visto por um cliente — inclui a serializacao JSON
    dos dois lados, nao so o parse. E o que o editor sente, entao e o que o
    orcamento deve limitar; separar parse de payload e assunto de A3.4.

    Devolve (frio_ms, incremental_ms, linhas, bytes, resposta_kb, diagnostico).
    """
    conteudo = fixture_rust(functions)
    linhas = conteudo.count("\n")
    tamanho = len(conteudo.encode())
    tmp = tempfile.mkdtemp(prefix="kinein-perf-syntax-")
    caminho = os.path.join(tmp, "fixture.rs")
    with open(caminho, "w") as fh:
        fh.write(conteudo)

    frios = []
    for _ in range(n):
        proc = spawn(core)
        rpc = make_rpc(proc)
        rpc("workspace.open", {"path": tmp})
        t0 = time.perf_counter()
        resposta = rpc("syntaxTree.update",
                       {"path": caminho, "content": conteudo, "version": 1})
        frios.append((time.perf_counter() - t0) * 1000.0)
        ok, motivo = syntax_snapshot_is_valid(resposta)
        if not ok:
            proc.kill()
            return "n/d", "n/d", linhas, tamanho, 0, motivo
        resposta_kb = len(json.dumps(resposta).encode()) // 1024
        proc.stdin.close()
        proc.wait(timeout=5)

    # Incremental: MESMO documento, um caractere a mais, versao seguinte. O
    # buffer ja esta no cache, entao mede o reparse incremental, nao o frio.
    proc = spawn(core)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": tmp})
    rpc("syntaxTree.update", {"path": caminho, "content": conteudo, "version": 1})
    incrementais = []
    for i in range(n):
        # Comentario no fim: edicao valida que nao quebra o parse.
        editado = conteudo + "// " + "x" * (i + 1) + "\n"
        t0 = time.perf_counter()
        resposta = rpc("syntaxTree.update",
                       {"path": caminho, "content": editado, "version": i + 2})
        incrementais.append((time.perf_counter() - t0) * 1000.0)
        ok, motivo = syntax_snapshot_is_valid(resposta)
        if not ok:
            proc.kill()
            return median_ms(frios), "n/d", linhas, tamanho, resposta_kb, motivo
    proc.stdin.close()
    proc.wait(timeout=5)
    return (median_ms(frios), median_ms(incrementais), linhas, tamanho,
            resposta_kb, "")


PROJETO_RUST = {
    "Cargo.toml": (
        '[package]\nname = "fixture_lsp"\nversion = "0.1.0"\nedition = "2021"\n'
        "\n[dependencies]\n"
    ),
    "src/main.rs": (
        "use std::collections::HashMap;\n"
        "\n"
        "pub struct Config {\n"
        "    pub nome: String,\n"
        "    pub valores: HashMap<String, u32>,\n"
        "}\n"
        "\n"
        "impl Config {\n"
        "    pub fn novo(nome: &str) -> Self {\n"
        "        Self { nome: nome.to_owned(), valores: HashMap::new() }\n"
        "    }\n"
        "\n"
        "    pub fn total(&self) -> u32 {\n"
        "        self.valores.values().sum()\n"
        "    }\n"
        "}\n"
        "\n"
        "fn main() {\n"
        "    let config = Config::novo(\"teste\");\n"
        "    println!(\"{} {}\", config.nome, config.total());\n"
        "}\n"
    ),
}

PROJETO_CPP = {
    "main.cpp": (
        "#include <string>\n"
        "#include <vector>\n"
        "\n"
        "struct Config {\n"
        "    std::string nome;\n"
        "    std::vector<int> valores;\n"
        "\n"
        "    int total() const {\n"
        "        int soma = 0;\n"
        "        for (int v : valores) soma += v;\n"
        "        return soma;\n"
        "    }\n"
        "};\n"
        "\n"
        "int main() {\n"
        "    Config config;\n"
        "    config.nome = \"teste\";\n"
        "    return config.total();\n"
        "}\n"
    ),
}


def escrever_projeto(arquivos, prefixo):
    raiz = tempfile.mkdtemp(prefix=prefixo)
    for rel, conteudo in arquivos.items():
        destino = os.path.join(raiz, rel)
        os.makedirs(os.path.dirname(destino), exist_ok=True)
        with open(destino, "w") as fh:
            fh.write(conteudo)
    return raiz


def esperar_resposta_util(rpc, metodo, params, valida, limite_s):
    """Repete `metodo` ate `valida` aceitar a resposta, ou estourar `limite_s`.

    Existe porque servidor LSP indexa em background: as primeiras respostas
    chegam VAZIAS e sao indistinguiveis de "servidor nao suporta". O contrato
    diz literalmente que servidor sem suporte responde lista vazia — entao
    aceitar a primeira resposta produziria sucesso falso, que e o que A3.2
    proibe. Espera-se resposta UTIL, com token/item de verdade.

    Devolve (ms_ate_primeira_util, ultima_resposta) ou (None, ultima_resposta).
    """
    inicio = time.perf_counter()
    ultima = None
    while (time.perf_counter() - inicio) < limite_s:
        ultima = rpc(metodo, params)
        if valida(ultima):
            return (time.perf_counter() - inicio) * 1000.0, ultima
        time.sleep(0.25)
    return None, ultima


def tokens_uteis(resposta, caminho, versao):
    """Valida `path`/`version` e exige ao menos um token (A3.2, aceite)."""
    r = (resposta or {}).get("result") or {}
    if r.get("path") != caminho or r.get("version") != versao:
        return False
    return bool(r.get("tokens"))


def itens_uteis(resposta, caminho):
    r = (resposta or {}).get("result") or {}
    if r.get("path") not in (None, caminho):
        return False
    return bool(r.get("items"))


def measure_lsp(core, tool, arquivos, rel, posicao, n, limite_s=90):
    """A3.2: primeira resposta semantica e completion, por servidor.

    Separa o que e da Kinein do que e da ferramenta externa:
      - `first`  = do documento sincronizado ate a primeira resposta UTIL.
        Inclui startup e indexacao do servidor — e custo EXTERNO, dominante.
      - `warm`   = round-trip com o servidor ja quente. Esse e o custo da
        Kinein, e e o unico que faz sentido orcar.
      - `rss`    = footprint do processo externo, informativo.

    Ferramenta ausente => `n/d` explicito. Nunca sucesso falso.
    """
    if shutil.which(tool) is None:
        return {"estado": f"n/d ({tool} ausente no PATH)"}

    raiz = escrever_projeto(arquivos, f"kinein-perf-{tool}-")
    caminho = os.path.join(raiz, rel)
    conteudo = arquivos[rel]
    proc = spawn(core)
    resultado = {"estado": "ok"}
    try:
        rpc = make_rpc(proc)
        rpc("workspace.open", {"path": raiz})
        rpc("lsp.didChange", {"path": caminho, "content": conteudo})

        tokens_ms, _ = esperar_resposta_util(
            rpc, "lsp.semanticTokens",
            {"path": caminho, "content": conteudo, "version": 1},
            lambda r: tokens_uteis(r, caminho, 1), limite_s)
        if tokens_ms is None:
            resultado["estado"] = f"n/d (sem token util em {limite_s}s)"
            return resultado
        resultado["tokens_first_ms"] = f"{tokens_ms:.1f}"

        quentes = []
        for versao in range(2, 2 + n):
            t0 = time.perf_counter()
            resposta = rpc("lsp.semanticTokens",
                           {"path": caminho, "content": conteudo, "version": versao})
            if tokens_uteis(resposta, caminho, versao):
                quentes.append((time.perf_counter() - t0) * 1000.0)
        resultado["tokens_warm_ms"] = median_ms(quentes)

        linha, coluna = posicao
        itens_ms, _ = esperar_resposta_util(
            rpc, "lsp.completion",
            {"path": caminho, "content": conteudo, "line": linha, "column": coluna},
            lambda r: itens_uteis(r, caminho), limite_s)
        if itens_ms is None:
            resultado["completion_first_ms"] = f"n/d (sem item em {limite_s}s)"
        else:
            resultado["completion_first_ms"] = f"{itens_ms:.1f}"
            quentes = []
            for _ in range(n):
                t0 = time.perf_counter()
                resposta = rpc("lsp.completion", {
                    "path": caminho, "content": conteudo,
                    "line": linha, "column": coluna})
                if itens_uteis(resposta, caminho):
                    quentes.append((time.perf_counter() - t0) * 1000.0)
            resultado["completion_warm_ms"] = median_ms(quentes)

        resultado["rss_mb"] = descendants_rss_kb(proc.pid) // 1024
        return resultado
    finally:
        # A3.2 item 4: nenhum filho sobrevive a amostra. O servidor LSP e filho
        # do core; matar o core sem esperar deixaria orfao segurando RAM.
        try:
            proc.stdin.close()
            proc.wait(timeout=10)
        except (subprocess.TimeoutExpired, OSError, ValueError):
            proc.kill()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                pass


def measure_rss_with_lsp(core, root):
    proc = spawn(core)
    rpc = make_rpc(proc)
    rpc("workspace.open", {"path": root})
    # fs.read de um .rs dispara did_open -> rust-analyzer sobe e indexa.
    rpc("fs.read", {"path": "crates/kinein-core/src/lib.rs"})
    time.sleep(8)  # janela fixa de indexação do rust-analyzer
    core_rss = vmrss_kb(proc.pid)
    lsp_rss = descendants_rss_kb(proc.pid)
    proc.stdin.close()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()
    return core_rss, lsp_rss


def main():
    core, root, n = sys.argv[1], sys.argv[2], int(sys.argv[3])
    print(f"workspace_open_ms={measure_workspace_open(core, root, n)}")
    print(f"fs_read_10k_ms={measure_fs_read_big(core, n)}")
    frio, incremental, linhas, tamanho, resposta_kb, motivo = measure_syntax(core, n)
    print(f"syntax_first_snapshot_ms={frio}")
    print(f"syntax_incremental_update_ms={incremental}")
    print(f"syntax_fixture_lines={linhas}")
    print(f"syntax_fixture_bytes={tamanho}")
    # Tamanho da resposta: cada update devolve highlights/outline do arquivo
    # INTEIRO. Se o incremental nao ganha do frio, e aqui que se olha primeiro.
    print(f"syntax_response_kb={resposta_kb}")
    if motivo:
        print(f"syntax_invalido={motivo}")

    # A3.2: cada servidor separado. Ausente => n/d explicito, nunca sucesso
    # falso. `first` carrega startup+indexacao EXTERNOS; `warm` e o round-trip
    # da Kinein — sao numeros de naturezas diferentes e nao se somam.
    for tool, arquivos, rel, pos in [
        ("rust-analyzer", PROJETO_RUST, "src/main.rs", (22, 18)),
        ("clangd", PROJETO_CPP, "main.cpp", (18, 11)),
    ]:
        r = measure_lsp(core, tool, arquivos, rel, pos, n)
        prefixo = tool.replace("-", "_")
        if r["estado"] != "ok":
            print(f"{prefixo}_semantic={r['estado']}")
            continue
        for chave in ("tokens_first_ms", "tokens_warm_ms",
                      "completion_first_ms", "completion_warm_ms", "rss_mb"):
            if chave in r:
                print(f"{prefixo}_{chave}={r[chave]}")
    core_rss, lsp_rss = measure_rss_with_lsp(core, root)
    print(f"core_rss_mb={core_rss // 1024}")
    print(f"lsp_rss_mb={(lsp_rss // 1024) if lsp_rss else 'n/d (rust-analyzer ausente?)'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
