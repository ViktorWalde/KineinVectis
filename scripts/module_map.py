#!/usr/bin/env python3
"""Mapa de modulos: quem fala com quem no repositorio, por que e como — MEDIDO.

POR QUE GERADO (2026-10-01). Diagrama desenhado a mao envelhece em silencio:
na mesma data, 12 citacoes de caminho apontavam para documentos renomeados
meses antes, e nenhum gate via. Este script le o CODIGO e escreve
DocsPublic/arquitetura/01-mapa-de-modulos.md; com --check, reprova se o
documento divergir do que o codigo diz (o gate do verificar.sh).

O QUE E' DECLARADO E O QUE E' DERIVADO. Open/closed de proposito:

  declarado (aqui)  as arestas entre PASTAS do repositorio, cada uma com o
                    porque, o como e uma EVIDENCIA conferida no codigo; e os
                    CONTEXTOS (git, editor, embarcados...): o nome, os
                    roteadores QML que os servem, as ferramentas externas e o
                    porque. Contexto novo = uma entrada nova em CONTEXTS.
  derivado          tudo o mais: controllers (propriedades dos roteadores),
                    metodos C++ do CoreClient (chamadas dos roteadores),
                    metodos IPC (corpo C++), o handler Rust que os roteia (o
                    mesmo extrator do verificar_fiacao_ipc.py — dono unico), os
                    modulos do core (`crate::` reais), os ciclos entre eles e o
                    "o que faz" de cada arquivo (o comentario de cabecalho dele).

CICLOS. Dependencia circular entre modulos do core e' acoplamento que impede
mudar um sem o outro. Os ciclos medidos em 2026-10-01 estao em KNOWN_CYCLES,
com o motivo; um ciclo NOVO reprova o --check.

Uso:
    python3 scripts/module_map.py            # reescreve o documento
    python3 scripts/module_map.py --check    # gate: reprova se divergir
"""
from __future__ import annotations

import importlib
import json
import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
# O extrator de metodos tem um dono so': o gate de fiacao IPC. O modulo dele tem
# nome legado em portugues (anterior ao G0.1); importar pelo nome em string evita
# repetir o legado num arquivo novo. Renomear o gate e' divida registrada.
core_methods_by_file = importlib.import_module("verificar_fiacao_ipc").core_methods_by_file

ROOT = Path(__file__).resolve().parent.parent
DOC = ROOT / "DocsPublic" / "arquitetura" / "01-mapa-de-modulos.md"
CORE_SRC = ROOT / "crates" / "kinein-core" / "src"
QML = ROOT / "ui" / "qml"
CPP = ROOT / "ui" / "src"

# ---------------------------------------------------------------------------
# Nivel 1: as pastas do repositorio. Cada aresta: (de, para, como, porque,
# evidencia). A evidencia e' (arquivo ou pasta, regex) que TEM de casar; a
# aresta do Cargo e' conferida contra o `cargo metadata` (sem sobra, sem falta).
# ---------------------------------------------------------------------------


@dataclass
class Edge:
    source: str
    target: str
    how: str
    why: str
    evidence: tuple[str, str] | None = None
    cargo: bool = False
    label: str = ""


FOLDERS = {
    "ui/qml": "a interface: telas, controllers e roteadores de IPC (QML)",
    "ui/src": "a ponte C++/Qt: CoreClient (processo, JSON-RPC, despacho), janela, CLI",
    "crates/kinein-core": "o core Rust: todo o trabalho (projeto, build, LSP, git, debug, terminal...)",
    "crates/kinein-protocol": "os tipos de mensagem do IPC (contrato unico, serde)",
    "crates/kinein-config": "leitura e validacao das configuracoes do usuario",
    "crates/kinein-cli": "gera UM pedido JSON-RPC por invocacao, para scripts e smokes",
    "schemas": "o formato dos arquivos persistidos (JSON Schema)",
    "scripts": "os gates e as provas; ninguem importa estes arquivos",
    "packaging": "o AppImage: junta ui + core num pacote portavel",
    "DocsPublic": "a documentacao; o 03 e' lido por gate como lista canonica do IPC",
}

EDGES = [
    Edge("ui/qml", "ui/src",
         "propriedade `coreClient` (QML_ELEMENT) chamada pelos roteadores de `ui/qml/ipc`",
         "a QML nunca fala processo nem JSON: so' o CoreClient sabe que o core existe",
         ("ui/qml/ipc", r"coreClient\.\w+\("), label="coreClient (QML_ELEMENT)"),
    Edge("ui/src", "crates/kinein-core",
         "processo filho (QProcess); JSON-RPC 2.0 por linha em stdin/stdout",
         "a UI nao linka Rust: um crash do core nao derruba a janela, e o core reinicia",
         ("ui/src/core_client_process.cpp", r"kinein-core"), label="processo + JSON-RPC (stdio)"),
    Edge("crates/kinein-core", "crates/kinein-protocol", "dependencia Cargo",
         "o formato de cada mensagem tem um dono so', compartilhado com a CLI", cargo=True),
    Edge("crates/kinein-core", "crates/kinein-config", "dependencia Cargo",
         "ler e validar configuracao nao e' trabalho de cada dominio", cargo=True),
    Edge("crates/kinein-cli", "crates/kinein-protocol", "dependencia Cargo",
         "a CLI monta pedidos com os MESMOS tipos que o core le", cargo=True),
    Edge("crates/kinein-core", "schemas",
         "o formato persistido e' documentado pelo schema",
         "arquivo que o usuario guarda precisa de contrato fora do codigo",
         ("crates/kinein-core/src/workspace/mod.rs", r"schemas/workspace\.schema\.json"), label="contrato do arquivo"),
    Edge("scripts", "DocsPublic",
         "o verificar_fiacao_ipc.py le o 03-protocolo-ipc.md",
         "o documento do protocolo nao pode divergir dos metodos roteados",
         ("scripts/verificar_fiacao_ipc.py", r"03-protocolo-ipc\.md"), label="le o 03 (lista canonica)"),
    Edge("scripts", "crates/kinein-core",
         "os gates sobem o binario do core e falam JSON-RPC com ele",
         "prova contra o core REAL, nao contra um falso",
         ("scripts/verificar_python_debug.py", r"kinein-core"), label="sobe o core real"),
    Edge("packaging", "ui/src",
         "o empacotador compila a UI release e a poe no AppDir",
         "o pacote e' o que o usuario roda; tem de ser o mesmo codigo",
         ("scripts/empacotar-appimage.sh", r"kinein-vectis"), label="compila a UI release"),
    Edge("packaging", "crates/kinein-core",
         "o empacotador compila o core release e o poe ao lado da UI",
         "UI e core viajam juntos e na mesma versao",
         ("scripts/empacotar-appimage.sh", r"--package kinein-core"), label="compila o core release"),
]

# ---------------------------------------------------------------------------
# Contextos: os roteadores QML de cada um, as ferramentas externas e o porque.
# ---------------------------------------------------------------------------


@dataclass
class Context:
    key: str
    title: str
    routers: list[str]
    tools: list[str]
    why: str


CONTEXTS = [
    Context("workspace", "Workspace e arvore do projeto",
            ["ProjectTreeRequestRouter", "WorkspaceEventRouter"], ["sistema de arquivos", "lixeira do desktop"],
            "Abrir uma pasta e mexer em arquivos e' o unico caminho de escrita em disco fora do editor: "
            "passa pelo core para que a vigilancia de arquivos, os rascunhos e o indice vejam a mesma mudanca."),
    Context("editor", "Editor e inteligencia de linguagem (LSP)",
            ["EditorRequestRouter", "EditorEventRouter"], ["clangd", "rust-analyzer", "pyright/basedpyright"],
            "O editor nao conhece servidor de linguagem: pede ao core, que sobe um servidor por linguagem, "
            "traduz LSP para o protocolo da IDE e devolve diagnosticos e simbolos."),
    Context("search", "Busca, indice e simbolos",
            ["SearchRequestRouter", "SearchEventRouter", "IndexRequestRouter", "IndexEventRouter"],
            ["ripgrep", "tree-sitter (embutido)"],
            "Buscar e substituir mexe em disco; o indice e' do core para responder sem reler a arvore. "
            "O roteador da busca conhece o editor: substituir com arquivo sujo perderia edicao."),
    Context("git", "Git",
            ["GitRequestRouter", "GitEventRouter"], ["git"],
            "O core executa o git do sistema. O roteador e' a unica aresta do git para o editor: "
            "checkout, branch, pull e stash nao rodam com arquivo modificado."),
    Context("runtime", "Executar, terminal, build e testes",
            ["RuntimeRequestRouter", "RuntimeEventRouter", "JobsEventRouter", "CoverageRequestRouter",
             "CoverageEventRouter"],
            ["cargo", "cmake/ninja", "ctest", "pytest", "sh -lc"],
            "Toda execucao e' uma sessao de terminal do core (PTY), e todo trabalho longo e' um Job: "
            "a UI so' recebe o render e os eventos, e cancelar e' um pedido como outro qualquer."),
    Context("debug", "Depuracao (DAP)",
            ["DebugRequestRouter", "DebugEventRouter"], ["gdb -i dap", "lldb-dap", "debugpy"],
            "O core e' o cliente DAP: sobe o adaptador, guarda a sessao e manda para a UI frames, "
            "variaveis e paradas ja' no formato da IDE."),
    Context("embedded", "Embarcados: sonda, serial, gravacao",
            ["EmbeddedRequestRouter", "EmbeddedEventRouter"],
            ["esptool", "OpenOCD", "probe-rs", "mpremote", "tio/picocom", "QEMU"],
            "Gravar e monitorar passam pelo mesmo caminho de Executar (configuracao de execucao): "
            "a IDE compoe a linha da ferramenta oficial e nunca grava sem pedido."),
    Context("toolchain", "Toolchains e kits",
            ["ToolchainRequestRouter", "ToolchainEventRouter"], ["compiladores cross", "rustup"],
            "O kit diz qual compilador, sysroot e alvo valem; o core detecta, explica a origem e "
            "nunca 'corrige' sozinho."),
    Context("remote", "Remote SSH",
            ["RemoteRequestRouter", "RemoteEventRouter"], ["ssh", "rsync"],
            "O alvo remoto e' espelhado e comandado pelo core; o resultado de `remote.command` vai para "
            "o dono certo (configuracao de execucao, kit ou terminal)."),
    Context("data", "Banco de dados e observabilidade",
            ["DataSourceRequestRouter", "DataSourceEventRouter", "GrafanaRequestRouter", "GrafanaEventRouter"],
            ["PostgreSQL/SQLite/...", "Grafana"],
            "Credenciais atravessam o roteador uma vez e nao ficam: a senha vai no `datasource.test`, "
            "o token no `grafana.probe`, e o cliente os redige do log."),
    Context("container", "Containers",
            ["ContainerRequestRouter", "ContainerEventRouter"], ["docker", "podman"],
            "A UI nunca chama docker/podman: o roteador e' a unica ponte, e o core fala com o motor."),
    Context("python", "Python",
            ["PythonRequestRouter", "PythonEventRouter"], ["python", "uv", "ruff"],
            "O interpretador do projeto (venv, uv) e' decidido no core e usado por executar, testar, "
            "depurar e formatar — um dono so'."),
    Context("config", "Configuracao: acoes, biblioteca, setup e ajustes",
            ["ConfigActionRequestRouter", "ConfigActionEventRouter", "LibraryRequestRouter",
             "LibraryEventRouter", "SetupRequestRouter", "SetupEventRouter", "SettingsEventRouter"],
            ["os gerenciadores oficiais de cada ecossistema"],
            "Mudanca de configuracao do projeto e' sempre list -> preview -> apply, com escopo e "
            "documentacao; o controller nunca escreve arquivo de configuracao."),
]

# Ciclos entre modulos do core medidos em 2026-10-01. Nao e' licenca: e' a
# divida declarada. Ciclo fora daqui reprova o --check.
KNOWN_CYCLES = {
    frozenset({"python", "run"}): "o run pede o interpretador ao python, e o python executa pelo run",
    frozenset({"configaction", "library"}): "a biblioteca propoe acoes, e as acoes aplicam bibliotecas",
}

# ---------------------------------------------------------------------------
# Leitura do codigo
# ---------------------------------------------------------------------------


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def header_comment(path: Path) -> str:
    """A primeira frase do comentario de cabecalho: o que o arquivo diz de si."""
    lines = []
    for line in read(path).splitlines():
        stripped = line.strip()
        if stripped.startswith(("//!", "///")) or (stripped.startswith("//") and not lines and False):
            pass
        if stripped.startswith("//!"):
            lines.append(stripped[3:].strip())
        elif stripped.startswith("//") and path.suffix in (".qml", ".cpp", ".h"):
            lines.append(stripped[2:].strip())
        elif stripped.startswith(("import ", "#include", "#pragma", "pragma ")) or not stripped:
            if lines:
                break
            continue
        else:
            break
    text = " ".join(part for part in lines if part)
    sentence = re.split(r"(?<=[.:])\s", text, maxsplit=1)[0] if text else ""
    sentence = sentence.replace("|", "/").replace("`", "")
    return sentence[:160] + ("…" if len(sentence) > 160 else "")


def core_modules() -> list[str]:
    names = set()
    for entry in CORE_SRC.iterdir():
        if entry.name in ("lib.rs", "main.rs", "tests"):
            continue
        if entry.suffix == ".rs":
            names.add(entry.stem)
        elif entry.is_dir():
            names.add(entry.name)
    return sorted(names)


def module_files(module: str) -> list[Path]:
    files = []
    if (CORE_SRC / f"{module}.rs").exists():
        files.append(CORE_SRC / f"{module}.rs")
    if (CORE_SRC / module).is_dir():
        files += [p for p in (CORE_SRC / module).rglob("*.rs") if "tests" not in p.parts]
    return sorted(files)


def crate_refs(text: str, modules: set[str]) -> set[str]:
    return {name for name in re.findall(r"crate::([a-z_]+)", text) if name in modules}


def core_graph() -> dict[str, set[str]]:
    modules = core_modules()
    known = set(modules)
    graph: dict[str, set[str]] = {}
    for module in modules:
        refs: set[str] = set()
        for path in module_files(module):
            refs |= crate_refs(read(path), known)
        graph[module] = refs - {module}
    return graph


def cycles(graph: dict[str, set[str]]) -> list[frozenset[str]]:
    """Componentes fortemente conexos com mais de um modulo (Tarjan)."""
    index: dict[str, int] = {}
    low: dict[str, int] = {}
    stack: list[str] = []
    on_stack: set[str] = set()
    found: list[frozenset[str]] = []
    counter = [0]

    def visit(node: str) -> None:
        index[node] = low[node] = counter[0]
        counter[0] += 1
        stack.append(node)
        on_stack.add(node)
        for nxt in sorted(graph.get(node, ())):
            if nxt not in index:
                visit(nxt)
                low[node] = min(low[node], low[nxt])
            elif nxt in on_stack:
                low[node] = min(low[node], index[nxt])
        if low[node] == index[node]:
            component = set()
            while True:
                item = stack.pop()
                on_stack.discard(item)
                component.add(item)
                if item == node:
                    break
            if len(component) > 1:
                found.append(frozenset(component))

    for node in sorted(graph):
        if node not in index:
            visit(node)
    return sorted(found, key=sorted)


def cpp_functions() -> dict[str, tuple[Path, set[str]]]:
    """CoreClient::metodo -> (arquivo, metodos IPC que o corpo envia)."""
    functions: dict[str, tuple[Path, set[str]]] = {}
    start = re.compile(r"^[\w:<>&*, ]*\bCoreClient::(\w+)\(", re.MULTILINE)
    for path in sorted(CPP.glob("core_client*.cpp")):
        text = read(path)
        matches = list(start.finditer(text))
        for current, nxt in zip(matches, matches[1:] + [None]):
            body = text[current.end():nxt.start() if nxt else len(text)]
            methods = set(re.findall(r'QStringLiteral\("([a-z][A-Za-z]*(?:\.[a-zA-Z]+)+)"\)', body))
            methods = {m for m in methods if not m.startswith("event.")}
            name = current.group(1)
            if name in functions:
                methods |= functions[name][1]
            functions[name] = (path, methods)
    return functions


def cpp_signal_owners() -> dict[str, set[Path]]:
    owners: dict[str, set[Path]] = defaultdict(set)
    for path in sorted(CPP.glob("core_client*.cpp")):
        for signal in re.findall(r"\bemit\s+(\w+)\(", read(path)):
            owners[signal].add(path)
    return owners


def find_qml(name: str) -> Path | None:
    hits = sorted(QML.rglob(f"{name}.qml"))
    return hits[0] if hits else None


@dataclass
class Chain:
    controllers: set[Path] = field(default_factory=set)
    routers: list[Path] = field(default_factory=list)
    links: set[tuple[str, str, bool]] = field(default_factory=set)
    cpp_files: set[Path] = field(default_factory=set)
    dispatch_files: set[Path] = field(default_factory=set)
    methods: set[str] = field(default_factory=set)
    handlers: dict[Path, set[str]] = field(default_factory=dict)
    modules: set[str] = field(default_factory=set)
    missing: list[str] = field(default_factory=list)


CORE_CLIENT_ID = re.compile(r"\bCoreClient\s*\{\s*id:\s*(\w+)")


def core_client_calls(text: str) -> set[str]:
    """Metodos do CoreClient chamados no QML: pela propriedade `coreClient` e,
    no arquivo que o instancia, pelo `id` dele (2026-10-08: o `Main.qml` passou
    a chamar `core`, porque o qmllint 6.12 recusa id com nome de propriedade;
    sem isto o mapa dizia "so' pela ponte C++" de metodos que o QML envia)."""
    names = {"coreClient"} | set(CORE_CLIENT_ID.findall(text))
    pattern = r"(?<![\w])(?:[\w.]*\.)?(?:" + "|".join(sorted(names)) + r")\.(\w+)\("
    return set(re.findall(pattern, text))


def resolve(context: Context, functions, owners, by_file, modules: set[str]) -> Chain:
    chain = Chain()
    for router in context.routers:
        path = QML / "ipc" / f"{router}.qml"
        if not path.exists():
            chain.missing.append(f"roteador {router}.qml nao existe")
            continue
        chain.routers.append(path)
        text = read(path)
        is_request = path.stem.endswith("RequestRouter")
        for prop in re.findall(r"property\s+var\s+(\w+Controller)\b", text):
            found = find_qml(prop[0].upper() + prop[1:])
            if found:
                chain.controllers.add(found)
                # ida: controller -> roteador de pedido; volta: roteador de evento -> controller
                if is_request:
                    chain.links.add((rel(found), rel(path), False))
                else:
                    chain.links.add((rel(path), rel(found), True))
        for call in core_client_calls(text):
            if call in functions:
                cpp_path, methods = functions[call]
                chain.cpp_files.add(cpp_path)
                chain.methods |= methods
                chain.links.add((rel(path), rel(cpp_path), False))
                for method in methods:
                    chain.links.add((rel(cpp_path), "ipc_" + method.split(".")[0], False))
        for handler in re.findall(r"function\s+on([A-Z]\w*)\s*\(", text):
            signal = handler[0].lower() + handler[1:]
            for dispatch in owners.get(signal, set()):
                chain.dispatch_files.add(dispatch)
                chain.links.add((rel(dispatch), rel(path), True))
    for path, methods in by_file.items():
        mine = methods & chain.methods
        if mine:
            chain.handlers[path] = mine
            chain.modules |= crate_refs(read(path), modules)
    return chain


# ---------------------------------------------------------------------------
# Escrita
# ---------------------------------------------------------------------------


def node_id(text: str) -> str:
    return "n_" + re.sub(r"[^A-Za-z0-9]", "_", text)


def check_level_one() -> list[str]:
    errors = []
    metadata = json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
        cwd=ROOT, capture_output=True, text=True, check=True).stdout)
    names = {p["name"] for p in metadata["packages"]}
    actual = {(f"crates/{p['name']}", f"crates/{d['name']}")
              for p in metadata["packages"] for d in p["dependencies"] if d["name"] in names}
    declared = {(e.source, e.target) for e in EDGES if e.cargo}
    for missing in sorted(actual - declared):
        errors.append(f"dependencia Cargo nao declarada no mapa: {missing[0]} -> {missing[1]}")
    for stale in sorted(declared - actual):
        errors.append(f"aresta Cargo declarada que o Cargo.toml nao tem: {stale[0]} -> {stale[1]}")
    for edge in EDGES:
        for end in (edge.source, edge.target):
            if end not in FOLDERS:
                errors.append(f"pasta sem descricao em FOLDERS: {end}")
        if edge.evidence:
            where, pattern = edge.evidence
            target = ROOT / where
            texts = [read(p) for p in target.rglob("*") if p.is_file()] if target.is_dir() else (
                [read(target)] if target.exists() else [])
            if not any(re.search(pattern, t) for t in texts):
                errors.append(f"evidencia sumiu: {edge.source} -> {edge.target} ({where} ~ /{pattern}/)")
    return errors


def render() -> tuple[str, list[str]]:
    errors = check_level_one()
    graph = core_graph()
    modules = set(graph)
    functions = cpp_functions()
    owners = cpp_signal_owners()
    by_file = core_methods_by_file()
    all_methods = set().union(*by_file.values())

    out: list[str] = []
    w = out.append
    w("# 01 — Mapa de módulos: quem fala com quem, por quê e como")
    w("")
    w("> **GERADO** por `scripts/module_map.py` a partir do código — não edite à mão.")
    w("> O porquê de cada aresta e de cada contexto mora nas tabelas `EDGES` e `CONTEXTS`")
    w("> do script; o resto (controllers, métodos, handlers, módulos, ciclos) é **medido**.")
    w("> O gate `scripts/module_map.py --check` reprova se este documento divergir do código")
    w("> ou se surgir um ciclo novo entre módulos do core.")
    w("")
    w("## Como ler")
    w("")
    w("- **Nível 1** mostra as pastas do repositório e as únicas formas pelas quais elas se")
    w("  comunicam. Tudo o que não está ali **não acontece** (a QML não fala com o core; o")
    w("  core não conhece a UI).")
    w("- **O caminho de um pedido** é o mesmo em todos os contextos: o que muda é o domínio.")
    w("- **Nível 2** é o grafo interno do `kinein-core`, domínio a domínio.")
    w("- **Contextos** repetem o caminho para cada área da IDE, com os arquivos reais de cada")
    w("  etapa e o que cada um diz de si (o comentário de cabeçalho dele).")
    w("")
    w("## Nível 1 — as pastas do repositório")
    w("")
    w("```mermaid")
    w("flowchart LR")
    for folder, what in FOLDERS.items():
        w(f'  {node_id(folder)}["<b>{folder}</b><br/>{what}"]')
    for edge in EDGES:
        label = "Cargo" if edge.cargo else edge.label
        w(f'  {node_id(edge.source)} -->|"{label}"| {node_id(edge.target)}')
    w("```")
    w("")
    w("| De | Para | Como | Por quê |")
    w("| --- | --- | --- | --- |")
    for edge in EDGES:
        w(f"| `{edge.source}` | `{edge.target}` | {edge.how} | {edge.why} |")
    w("")
    w("## O caminho de um pedido (todos os contextos)")
    w("")
    w("```mermaid")
    w("sequenceDiagram")
    w("  participant C as ui/qml/(domínio)/XController")
    w("  participant R as ui/qml/ipc/XRequestRouter")
    w("  participant K as ui/src/CoreClient")
    w("  participant H as kinein-core handlers/(x).rs")
    w("  participant M as kinein-core (módulos)")
    w("  participant E as ui/qml/ipc/XEventRouter")
    w("  C->>R: sinal xRequested (intenção)")
    w("  R->>K: coreClient.metodo(args)")
    w("  K->>H: JSON-RPC 'dominio.metodo' por stdin")
    w("  H->>M: chamada Rust")
    w("  H-->>K: resposta ou event.* por stdout")
    w("  K-->>E: sinal C++ (core_client_dispatch*.cpp)")
    w("  E-->>C: atualiza o estado do controller")
    w("```")
    w("")
    w("Por que assim: o **controller** guarda estado e intenção e não sabe que existe IPC;")
    w("o **RequestRouter** é o único que chama o `CoreClient` naquele domínio (uma guarda")
    w("cross-domain, quando existe, mora nele e só nele); o **EventRouter** é o espelho da")
    w("volta. Trocar o transporte muda o `CoreClient`, e nenhuma tela.")
    w("")
    w(f"Medido: {len(all_methods)} métodos IPC roteados pelo core.")
    w("")
    w("## Nível 2 — os domínios do `kinein-core`")
    w("")
    found_cycles = cycles(graph)
    in_cycle = {m for c in found_cycles for m in c}
    w("```mermaid")
    w("flowchart LR")
    for module in sorted(graph):
        style = ":::cycle" if module in in_cycle else ""
        w(f"  {node_id('core_' + module)}[{module}]{style}")
    for module in sorted(graph):
        for dep in sorted(graph[module]):
            w(f"  {node_id('core_' + module)} --> {node_id('core_' + dep)}")
    w("  classDef cycle stroke:#d33,stroke-width:3px")
    w("```")
    w("")
    w(f"{len(graph)} módulos, {sum(len(v) for v in graph.values())} dependências"
      " (`crate::<módulo>` fora de testes). Em vermelho, os que estão num ciclo.")
    w("")
    w("### Ciclos")
    w("")
    if not found_cycles:
        w("Nenhum.")
    for component in found_cycles:
        reason = KNOWN_CYCLES.get(component)
        names = " ↔ ".join(sorted(component))
        if reason is None:
            errors.append(f"ciclo NOVO entre modulos do core: {names}")
            w(f"- **{names}** — NOVO, sem motivo declarado")
        else:
            w(f"- **{names}** — dívida medida em 2026-10-01: {reason}")
    for known in KNOWN_CYCLES:
        if known not in found_cycles:
            errors.append(f"ciclo declarado que nao existe mais (tire de KNOWN_CYCLES): {' <-> '.join(sorted(known))}")
    w("")
    w("### O que cada domínio diz de si")
    w("")
    w("| Domínio | Depende de | Cabeçalho |")
    w("| --- | --- | --- |")
    for module in sorted(graph):
        files = module_files(module)
        head = ""
        for candidate in ([CORE_SRC / module / "mod.rs"] if (CORE_SRC / module).is_dir() else []) + files:
            if candidate.exists():
                head = header_comment(candidate)
                if head:
                    break
        dependencies = ", ".join(sorted(graph[module])) or "—"
        w(f"| `{module}` | {dependencies} | {head} |")
    w("")
    chains = {c.key: resolve(c, functions, owners, by_file, modules) for c in CONTEXTS}
    covered = set().union(*(chain.methods for chain in chains.values())) & all_methods
    direct: dict[str, set[str]] = defaultdict(set)
    for path in sorted(QML.rglob("*.qml")):
        if path.parent == QML / "ipc":
            continue
        for call in core_client_calls(read(path)):
            if call in functions:
                for method in functions[call][1] & all_methods:
                    if method not in covered:
                        direct[method].add(rel(path))
    called_from_qml = {call for path in QML.rglob("*.qml")
                       for call in core_client_calls(read(path))}
    cpp_only = {method for name, (_, methods) in functions.items() if name not in called_from_qml
                for method in methods} & all_methods - covered - set(direct)
    nobody = all_methods - covered - set(direct) - cpp_only
    w("## Cobertura: todo método IPC tem um lugar")
    w("")
    w(f"Dos {len(all_methods)} métodos roteados pelo core, {len(covered)} seguem o caminho padrão")
    w(f"e estão num contexto abaixo. Os outros {len(all_methods) - len(covered)} estão aqui,")
    w("nomeados, para nada ficar invisível:")
    w("")
    w("**Chamados direto da QML, sem RequestRouter** (fora do padrão — dívida medida em")
    w("2026-10-01: o composition root fala com o `CoreClient` no lugar do roteador do domínio):")
    w("")
    w("| Método | Quem chama |")
    w("| --- | --- |")
    for method in sorted(direct):
        w(f"| `{method}` | {', '.join(f'`{f}`' for f in sorted(direct[method]))} |")
    w("")
    w("**Enviados só pela ponte C++** (ciclo de vida do processo, respostas encadeadas): "
      + (", ".join(f"`{m}`" for m in sorted(cpp_only)) or "—") + ".")
    w("")
    w("**Sem cliente na UI** (a CLI, os gates ou nenhum): "
      + (", ".join(f"`{m}`" for m in sorted(nobody)) or "—") + ".")
    w("")
    w("## Contextos")
    w("")
    w("Cada contexto é o caminho de um pedido aplicado a uma área da IDE, com os arquivos")
    w("reais de cada etapa. Ida em linha cheia, volta (resposta e eventos) tracejada.")
    for context in CONTEXTS:
        chain = chains[context.key]
        errors += [f"contexto {context.key}: {m}" for m in chain.missing]
        if not chain.methods:
            errors.append(f"contexto {context.key}: nenhum metodo IPC derivado dos roteadores")
        prefixes: dict[str, int] = defaultdict(int)
        for method in chain.methods & all_methods:
            prefixes[method.split(".")[0]] += 1
        w("")
        w(f"### {context.title}")
        w("")
        w(context.why)
        w("")
        w("```mermaid")
        w("flowchart LR")
        w('  subgraph QML["ui/qml"]')
        for path in sorted(chain.controllers):
            w(f'    {node_id(rel(path))}["{path.stem}"]')
        for path in chain.routers:
            w(f'    {node_id(rel(path))}["ipc/{path.stem}"]')
        w("  end")
        w('  subgraph CPP["ui/src"]')
        for path in sorted(chain.cpp_files | chain.dispatch_files):
            w(f'    {node_id(rel(path))}["{path.name}"]')
        w("  end")
        w('  subgraph IPC["JSON-RPC"]')
        for prefix, count in sorted(prefixes.items()):
            w(f'    {node_id("ipc_" + prefix)}(["{prefix}.* · {count}"])')
        w("  end")
        w('  subgraph CORE["crates/kinein-core"]')
        for path in sorted(chain.handlers):
            w(f'    {node_id(rel(path))}["{rel(path).replace("crates/kinein-core/src/", "")}"]')
        for module in sorted(chain.modules):
            w(f'    {node_id("core_" + module)}[{module}]')
        w("  end")
        w(f'  {node_id("tools_" + context.key)}[/"{" · ".join(context.tools)}"/]')
        for source, target, back in sorted(chain.links):
            if target.startswith("ipc_") and target.removeprefix("ipc_") not in prefixes:
                continue
            w(f"  {node_id(source)} {'-.->' if back else '-->'} {node_id(target)}")
        for path, methods in sorted(chain.handlers.items()):
            for prefix in sorted({m.split('.')[0] for m in methods}):
                w(f"  {node_id('ipc_' + prefix)} --> {node_id(rel(path))}")
            for module in sorted(crate_refs(read(path), modules)):
                w(f"  {node_id(rel(path))} --> {node_id('core_' + module)}")
        # O core executa as ferramentas; qual modulo roda qual nao e' medido aqui.
        w(f"  CORE -.->|processos| {node_id('tools_' + context.key)}")
        w("```")
        w("")
        w("| Etapa | Arquivo | O que ele diz de si |")
        w("| --- | --- | --- |")
        for label, paths in (("controller", sorted(chain.controllers)), ("roteador", chain.routers),
                             ("ponte C++", sorted(chain.cpp_files | chain.dispatch_files)),
                             ("handler Rust", sorted(chain.handlers))):
            for path in paths:
                w(f"| {label} | `{rel(path)}` | {header_comment(path)} |")
        w("")
        listed = ", ".join(f"`{m}`" for m in sorted(chain.methods & all_methods))
        w(f"Métodos IPC ({len(chain.methods & all_methods)}): {listed or '—'}.")
    w("")
    return "\n".join(out), errors


def main() -> int:
    text, errors = render()
    if "--check" in sys.argv:
        current = read(DOC) if DOC.exists() else ""
        if current != text:
            errors.append(f"{rel(DOC)} diverge do codigo: rode python3 scripts/module_map.py")
        if errors:
            print("✗ mapa de modulos:", file=sys.stderr)
            for error in errors:
                print(f"  - {error}", file=sys.stderr)
            return 1
        print(f"mapa de modulos: {rel(DOC)} confere com o codigo ({len(CONTEXTS)} contextos).")
        return 0
    if errors:
        for error in errors:
            print(f"aviso: {error}", file=sys.stderr)
    DOC.write_text(text, encoding="utf-8")
    print(f"{rel(DOC)} escrito ({len(CONTEXTS)} contextos).")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
