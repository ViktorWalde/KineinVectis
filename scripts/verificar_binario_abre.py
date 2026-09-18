#!/usr/bin/env python3
"""O binario que sai do `cmake --build` ABRE — e nenhum objeto do link e' de
antes da instalacao de um header que ele inclui.

Ver scripts/verificar-binario-abre.sh para o porque.

Duas perguntas, nesta ordem, porque a primeira EXPLICA a segunda:

  1. OBSOLETO   ha' objeto na arvore de build compilado ANTES de a instalacao
                de uma dependencia dele mudar? O ninja compara mtime, e o rpm
                (e o dpkg, e o pacman) instala header com o mtime de quando o
                pacote foi CONSTRUIDO — meses atras. O ctime nao se falsifica:
                e' a hora em que o inode entrou neste disco.
  2. ABRE       o binario chega ao primeiro frame, offscreen, e sai com 0.
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
PRESETS = [RAIZ / "CMakePresets.json", RAIZ / "CMakeUserPresets.json"]
BINARIO = Path("ui") / "kinein-vectis"
MARCADOR = "KINEIN_PERF first_frame_ms="


def _presets() -> tuple[dict[str, dict], dict[str, dict]]:
    """Presets de configure e de build, dos dois arquivos (o do usuario e'
    opcional e vence pelo nome, como no proprio cmake)."""
    configure: dict[str, dict] = {}
    build: dict[str, dict] = {}
    for arquivo in PRESETS:
        if not arquivo.is_file():
            continue
        dados = json.loads(arquivo.read_text(encoding="utf-8"))
        for p in dados.get("configurePresets", []):
            configure[p["name"]] = p
        for p in dados.get("buildPresets", []):
            build[p["name"]] = p
    return configure, build


def binary_dir_do_preset(nome: str) -> Path:
    """`cmake --build --preset NOME` escreve em qual pasta? O build preset
    aponta o configure preset; o `binaryDir` pode vir por `inherits`."""
    configure, build = _presets()
    if nome in build:
        nome = build[nome]["configurePreset"]
    visto: set[str] = set()
    atual = nome
    while atual in configure and atual not in visto:
        visto.add(atual)
        p = configure[atual]
        if "binaryDir" in p:
            return Path(p["binaryDir"].replace("${sourceDir}", str(RAIZ)))
        herda = p.get("inherits", [])
        atual = herda[0] if isinstance(herda, list) and herda else str(herda)
    raise SystemExit(f"erro: preset '{nome}' sem binaryDir nos CMake*Presets.json")


def objetos_obsoletos(build_dir: Path) -> list[tuple[Path, Path, int, int]]:
    """(objeto, dependencia mais nova, mtime do objeto, ctime da dependencia)
    para cada objeto compilado antes de uma dependencia dele ser instalada."""
    saida = subprocess.run(
        ["ninja", "-C", str(build_dir), "-t", "deps"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    # Para cada dependencia: o instante em que ela CHEGOU ao disco (ctime), mas
    # SO' quando esse instante e' invisivel ao ninja — ver `armadilha_de`.
    armadilha: dict[str, int | None] = {}

    def armadilha_de(caminho: str, obj_mtime: int) -> int | None:
        completo = build_dir / caminho
        # Dependencia DENTRO da arvore de build e' do ninja: ele a gera, a
        # reescreve e sabe quando ela mudou (restat). A armadilha deste gate e'
        # o que o ninja NAO gerencia — o header que o PACOTE instalou.
        if completo.resolve().is_relative_to(build_dir.resolve()):
            return None
        if caminho not in armadilha:
            try:
                st = os.stat(completo)
                armadilha[caminho] = (st.st_mtime_ns, st.st_ctime_ns)
            except FileNotFoundError:
                # Dependencia que sumiu: o ninja ja' recompila por conta propria.
                armadilha[caminho] = None
        par = armadilha[caminho]
        if par is None:
            return None
        dep_mtime, dep_ctime = par
        # O TRAP, e SO' ele: a dependencia parece velha para o ninja
        # (mtime <= objeto, entao ele nao recompila) mas CHEGOU depois de o
        # objeto ser compilado (ctime > objeto). E' a assinatura do pacote que
        # instala header com mtime de quando foi construido. Um arquivo do REPO
        # que o autor acabou de editar tem mtime > objeto — o ninja o recompila,
        # e nao e' este gate que cuida disso (medido em 2026-09-11: editar o
        # CMakeLists reprovava um mocs_compilation que o proximo build conserta).
        if dep_mtime <= obj_mtime < dep_ctime:
            return dep_ctime
        return None

    achados: list[tuple[Path, Path, int, int]] = []
    vistos: set[Path] = set()
    objeto: Path | None = None
    mtime = 0
    pior: tuple[int, str] | None = None

    def fecha() -> None:
        # O mesmo alvo pode aparecer mais de uma vez no `-t deps`; uma linha basta.
        if objeto is not None and pior is not None and pior[0] > mtime and objeto not in vistos:
            vistos.add(objeto)
            achados.append((objeto, build_dir / pior[1], mtime, pior[0]))

    for linha in saida.splitlines():
        if not linha.startswith(" "):
            fecha()
            objeto, pior = None, None
            if ": #deps" not in linha:
                continue
            candidato = build_dir / linha.split(": #deps", 1)[0]
            try:
                mtime = os.stat(candidato).st_mtime_ns
            except FileNotFoundError:
                continue
            objeto = candidato
            continue
        if objeto is None:
            continue
        dep = linha.strip()
        c = armadilha_de(dep, mtime)
        if c is not None and (pior is None or c > pior[0]):
            pior = (c, dep)
    fecha()
    return achados


def abre(build_dir: Path, timeout: float) -> tuple[int | None, str, int | None]:
    """(codigo de saida ou None se estourou o tempo, saida, ms ate o primeiro frame)."""
    ambiente = dict(os.environ)
    ambiente.update(
        {
            "QT_QPA_PLATFORM": "offscreen",
            # O Qt do Fedora e' compilado com journald: sem tty no stderr, o
            # assert vai para o journal e o gate veria um SIGABRT MUDO.
            "QT_FORCE_STDERR_LOGGING": "1",
            "KINEIN_PERF_MARKER": "1",
            "KINEIN_PERF_EXIT": "1",
        }
    )
    try:
        proc = subprocess.run(
            [str(build_dir / BINARIO)],
            cwd=RAIZ,
            env=ambiente,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=timeout,
        )
    except subprocess.TimeoutExpired as e:
        # No POSIX o TimeoutExpired carrega BYTES mesmo com text=True.
        return None, _texto(e.stdout) + _texto(e.stderr), None
    texto = proc.stdout + proc.stderr
    ms: int | None = None
    for linha in texto.splitlines():
        if MARCADOR in linha:
            ms = int(linha.split(MARCADOR, 1)[1].strip())
    return proc.returncode, texto, ms


# Avisos do motor QML que significam fiacao ou binding QUEBRADOS — o qmllint
# nao os ve, porque dependem do tipo real do `target` em tempo de execucao.
# Achado em 2026-09-18 (F7): `onIdentifyRequested` numa Connections cujo
# target nao tinha o sinal — o serial.identify nunca saia, e nenhum gate
# reprovava; a IDE so' avisava no stderr, que ninguem lia.
AVISOS_QML = (
    "no signal of the target matches",
    "Binding loop detected",
    "ReferenceError",
    "TypeError",
    "is not a function",
    "Cannot assign",
    "Unable to assign",
)


def avisos_qml(texto: str) -> list[str]:
    return [l for l in texto.splitlines() if any(a in l for a in AVISOS_QML)]


def _texto(saida: bytes | str | None) -> str:
    if isinstance(saida, bytes):
        return saida.decode(errors="replace")
    return saida or ""


def _quando(ns: int) -> str:
    return datetime.fromtimestamp(ns / 1e9).strftime("%Y-%m-%d %H:%M:%S")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    alvo = parser.add_mutually_exclusive_group(required=True)
    alvo.add_argument("--preset", help="build preset do cmake (ex.: dev-local)")
    alvo.add_argument("--build-dir", type=Path, help="pasta de build do ninja")
    parser.add_argument(
        "--listar-obsoletos",
        action="store_true",
        help="so' imprime os objetos obsoletos, um por linha (para `| xargs rm -f`)",
    )
    parser.add_argument("--timeout", type=float, default=30.0)
    args = parser.parse_args()

    build_dir = args.build_dir.resolve() if args.build_dir else binary_dir_do_preset(args.preset)
    if not (build_dir / "build.ninja").is_file():
        print(f"erro: {build_dir} nao e' uma arvore de build do ninja.", file=sys.stderr)
        return 2

    obsoletos = objetos_obsoletos(build_dir)
    if args.listar_obsoletos:
        for objeto, _, _, _ in obsoletos:
            print(objeto)
        return 0

    if obsoletos:
        print(f"✗ {len(obsoletos)} alvo(s) compilado(s) ANTES de uma dependencia instalada mudar no disco:", file=sys.stderr)
        for objeto, dep, mtime, c in obsoletos:
            print(
                f"    {objeto.relative_to(build_dir)}\n"
                f"        compilado em {_quando(mtime)}; {dep} mudou no disco em {_quando(c)}",
                file=sys.stderr,
            )
        rotulo = f"--preset {args.preset}" if args.preset else f"--build-dir {build_dir}"
        print(
            "\n  O ninja nao os recompila: ele compara MTIME, e o pacote instala o"
            " header com o mtime de quando foi construido. Misturar objetos de duas"
            " versoes do mesmo header e' violacao de ODR — o linker dobra o inline"
            " numa versao so', e o chamador da outra aborta.\n"
            "  Remova-os e recompile:\n"
            f"    python3 scripts/verificar_binario_abre.py {rotulo} --listar-obsoletos | xargs rm -f\n"
            f"    cmake --build --preset {args.preset or '<preset>'}",
            file=sys.stderr,
        )
        return 1

    binario = build_dir / BINARIO
    if not os.access(binario, os.X_OK):
        print(f"erro: {binario} nao existe ou nao e' executavel.", file=sys.stderr)
        return 2

    inicio = time.monotonic()
    codigo, texto, ms = abre(build_dir, args.timeout)
    duracao = time.monotonic() - inicio
    if codigo == 0 and ms is not None:
        ruido = avisos_qml(texto)
        if ruido:
            print(f"✗ {binario.relative_to(RAIZ)} abre, mas o QML avisou em tempo de execucao:", file=sys.stderr)
            print("\n".join("    " + l for l in ruido), file=sys.stderr)
            return 1
        print(f"{binario.relative_to(RAIZ)}: abre, primeiro frame em {ms} ms, sem aviso do QML")
        return 0

    cauda = texto.strip().splitlines()[-40:]
    if cauda:
        print("\n".join("    " + l for l in cauda), file=sys.stderr)
    if codigo is None:
        print(f"✗ {binario.relative_to(RAIZ)} nao confirmou o primeiro frame em {args.timeout:.0f} s.", file=sys.stderr)
    elif codigo < 0:
        print(
            f"✗ {binario.relative_to(RAIZ)} morreu com o sinal {-codigo} apos {duracao:.1f} s.",
            file=sys.stderr,
        )
    elif codigo != 0:
        print(f"✗ {binario.relative_to(RAIZ)} saiu com {codigo} apos {duracao:.1f} s.", file=sys.stderr)
    else:
        print(f"✗ {binario.relative_to(RAIZ)} saiu com 0 sem imprimir '{MARCADOR}'.", file=sys.stderr)
    print("  Reproduzir na mao (o stderr precisa do QT_FORCE_STDERR_LOGGING, senao o assert vai para o journal):", file=sys.stderr)
    print(
        f"    QT_QPA_PLATFORM=offscreen QT_FORCE_STDERR_LOGGING=1 KINEIN_PERF_MARKER=1 KINEIN_PERF_EXIT=1"
        f" {binario.relative_to(RAIZ)}",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
