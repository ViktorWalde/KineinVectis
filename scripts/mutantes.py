#!/usr/bin/env python3
"""Catálogo versionado de defeitos plantados: mutantes de guarda e defeitos de gate.

POR QUE (2026-11-13, roadmaps/63, ENG-01). Até esta data o mutante era um
script avulso sobre a árvore de trabalho: nada dizia quais existiam, a troca
envelhecia calada quando o código mudava, e uma rodada interrompida deixava o
checkout com o defeito dentro. E "reprovou" não queria dizer "viu o defeito":
na medida de 2026-11-13, uma rodada inteira de um gate reprovou porque o passo
quebrou.

O CATÁLOGO. Um arquivo TOML por item em scripts/mutantes/:

    item = "INF-15"
    compila = "cargo test -p kinein-core --lib --all-features --locked --no-run"
    roda = "cargo test -p kinein-core --lib --all-features --locked kinein_dir"

    [[entrada]]
    id = "inf-15-guarda-segue-o-link"
    regra = "R-DADOS-02"
    mensagem = "test kinein_dir::tests::the_read_checks_each_component_on_the_descriptor ... FAILED"
    [[entrada.edicao]]
    arquivo = "crates/kinein-core/src/platform/unix.rs"
    texto = "OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC"
    troca = "OFlags::RDONLY | OFlags::NONBLOCK | OFlags::CLOEXEC"

A entrada planta um defeito (uma lista de edições: um texto exato trocado uma
vez, ou `novo`, o conteúdo de um arquivo que não existe) e diz quem o reprova
(`roda`, dela ou do catálogo) e o trecho que a reprovação traz (`mensagem`,
um texto ou uma lista; todos têm de aparecer na saída do `roda`, e nenhum na
saída dele sem o defeito). Com `espera = "passa"` a entrada é uma forma
legítima, que não pode reprovar. `compila`, quando o catálogo tem, roda antes:
quebrar nele é "não compila", que não prova nada. `prazo` é em segundos, e
`nucleos` é quantos núcleos cada cópia usa (4 com `compila`, 1 sem).

USO.

    python3 scripts/mutantes.py --conferir        # estático: o gate rápido
    python3 scripts/mutantes.py --rodar inf-15    # nas cópias, em paralelo
    python3 scripts/mutantes.py --rodar inf-15 --so inf-15-guarda-segue-o-link
    python3 scripts/mutantes.py --listar

O `--rodar` nunca grava no checkout. Ele espelha a árvore de trabalho (os
arquivos rastreados e os novos que o git não ignora) em cópias fora dele, cada
uma um repositório git com a sua pasta de build, e as guarda entre as rodadas:
só o que mudou é copiado e recompilado. Cada cópia roda o `roda` sem defeito
(tem de passar) e depois uma entrada por vez; entre uma e outra o defeito sai
e o `git status` da cópia tem de ser o de antes. O arquivo que a ferramenta
grava numa cópia sempre recebe a hora de agora: o build que confia na hora do
arquivo não pode guardar a saída de um defeito. A rodada encerrada (Ctrl-C,
SIGTERM, SIGHUP) mata os processos que criou, e a seguinte conserta as cópias.

SÓ "REPROVADO" SERVE. O defeito que sobrevive, que não compila ou que reprova
sem o trecho esperado reprova a rodada, e a forma legítima que reprova também
(R-TEST-02, R-GATE-03).

PENDENTES. Os mutantes que um dia ficaram registrados como equivalentes moram
em scripts/mutantes/pendentes.toml, cada um com `pendente = "<item>"`, o item
que tem de matá-lo (63 §0, D-21). Enquanto o dono está aberto, sobreviver é o
esperado; quando ele fecha, a entrada vai para o catálogo dele e tem de
reprovar. É catraca de migração: PENDING_OWNERS só desce, e o arquivo é
apagado no zero. Nenhuma entrada nova nasce pendente.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import contextlib
import dataclasses
import hashlib
import json
import os
import queue
import shutil
import signal
import subprocess
import sys
import threading
import time
import tomllib
import traceback
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOGUE_DIR = "scripts/mutantes"
PENDING_FILE = "pendentes.toml"
# Quantos pendentes cada dono ainda tem de matar. A ENG-40 leva estes números
# para a catraca comum.
PENDING_OWNERS = {"ENG-02": 4, "ENG-47": 6}
VERDICTS = ("reprova", "passa")
DEADLINE = 900
BASE_DEADLINE = 3600
PLANTED = "plantado"
TOP_KEYS = {"item", "compila", "prepara", "roda", "prazo", "nucleos", "entrada"}
ENTRY_KEYS = {"id", "regra", "espera", "roda", "mensagem", "pendente", "prazo", "edicao"}
EDIT_KEYS = {"arquivo", "texto", "troca", "novo"}
COPY_GIT = ("git", "-c", "user.name=mutantes", "-c", "user.email=mutantes@localhost",
            "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null", "-c", "gc.auto=0")
STATUS = ("status", "--porcelain", "--untracked-files=all", "-z")


@dataclasses.dataclass(frozen=True)
class Edit:
    """Uma mudança da entrada: `text` trocado uma vez por `swap`, ou o arquivo `new`."""
    file: str
    text: str | None
    swap: str | None
    new: str | None


@dataclasses.dataclass(frozen=True)
class Entry:
    name: str
    rule: str
    expects: str
    command: str
    fragments: tuple[str, ...]
    pending: str | None
    deadline: int
    edits: tuple[Edit, ...]


@dataclasses.dataclass(frozen=True)
class Catalogue:
    name: str
    builds: str | None
    prepares: str | None
    cores: int
    entries: tuple[Entry, ...]


@dataclasses.dataclass(frozen=True)
class Outcome:
    entry: Entry
    state: str
    seconds: float
    log: Path | None

    @property
    def expected(self) -> bool:
        if self.entry.pending:
            return self.state == "sobreviveu"
        return self.state == ("reprovado" if self.entry.expects == "reprova" else "passou")


class Shape:
    """Lê os campos de uma tabela do catálogo e junta o que está fora da forma."""

    def __init__(self, where: str, table: dict, allowed: set[str], problems: list[str]):
        self.where = where
        self.table = table
        self.problems = problems
        for key in sorted(table.keys() - allowed):
            problems.append(f"{where}: `{key}` não é campo do catálogo")

    def text(self, key: str, default: str | None = None) -> str | None:
        value = self.table.get(key, default)
        if value is not None and not isinstance(value, str):
            self.problems.append(f"{self.where}: `{key}` é um texto")
            return default
        return value

    def count(self, key: str, default: int) -> int:
        value = self.table.get(key, default)
        if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
            self.problems.append(f"{self.where}: `{key}` é um número inteiro maior que zero")
            return default
        return value

    def fragments(self, key: str) -> tuple[str, ...]:
        value = self.table.get(key, [])
        found = [value] if isinstance(value, str) else value
        if not isinstance(found, list) or not all(isinstance(item, str) and item.strip() for item in found):
            self.problems.append(f"{self.where}: `{key}` é um texto ou uma lista de textos, nenhum vazio")
            return ()
        return tuple(found)

    def tables(self, key: str) -> list[dict]:
        value = self.table.get(key, [])
        if not isinstance(value, list) or not all(isinstance(item, dict) for item in value):
            self.problems.append(f"{self.where}: `{key}` é uma lista de tabelas (`[[{key}]]`)")
            return []
        return value


def load(root: Path) -> tuple[list[Catalogue], list[str]]:
    """Os catálogos de `root` e o que está fora da forma neles."""
    catalogues, problems = [], []
    for path in sorted((root / CATALOGUE_DIR).glob("*.toml")):
        shown = f"{CATALOGUE_DIR}/{path.name}"
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
            problems.append(f"{shown}: não é TOML válido ({error})")
            continue
        top = Shape(shown, data, TOP_KEYS, problems)
        command = top.text("roda")
        deadline = top.count("prazo", DEADLINE)
        entries = []
        for index, raw in enumerate(top.tables("entrada"), 1):
            named = isinstance(raw.get("id"), str) and raw["id"].strip()
            where = f"{shown}: entrada `{raw['id']}`" if named else f"{shown}: entrada {index}"
            shape = Shape(where, raw, ENTRY_KEYS, problems)
            name = shape.text("id") or ""
            rule = shape.text("regra") or ""
            run_with = shape.text("roda", command) or ""
            for key, value in (("id", name), ("regra", rule), ("roda", run_with)):
                if not value.strip():
                    problems.append(f"{where}: falta `{key}`")
            expects = shape.text("espera", "reprova")
            if expects not in VERDICTS:
                problems.append(f"{where}: `espera` é `reprova` ou `passa`")
            fragments = shape.fragments("mensagem")
            pending = shape.text("pendente")
            if expects == "reprova" and pending is None and not fragments:
                problems.append(f"{where}: quem reprova diz em `mensagem` o trecho que a reprovação traz")
            if expects == "passa" and fragments:
                problems.append(f"{where}: a forma legítima passa, e não tem `mensagem`")
            edits = []
            for change in shape.tables("edicao"):
                part = Shape(where, change, EDIT_KEYS, problems)
                file = part.text("arquivo") or ""
                text, swap, new = part.text("texto"), part.text("troca"), part.text("novo")
                swaps = text is not None and swap is not None and new is None
                creates = new is not None and text is None and swap is None
                if not file or Path(file).is_absolute() or ".." in Path(file).parts:
                    problems.append(f"{where}: `arquivo` é um caminho dentro do repositório")
                elif not swaps and not creates:
                    problems.append(f"{where}: a edição de `{file}` é `texto` com `troca`, ou `novo`")
                elif swaps and (not text or text == swap):
                    problems.append(f"{where}: o `texto` de `{file}` é vazio ou igual à `troca`")
                else:
                    edits.append(Edit(file, text, swap, new))
            if not raw.get("edicao"):
                problems.append(f"{where}: sem edição não há defeito plantado")
            entries.append(Entry(name, rule, expects, run_with, fragments, pending,
                                 shape.count("prazo", deadline), tuple(edits)))
        builds = top.text("compila")
        # Quem compila usa vários núcleos por cópia; o passo estático, um.
        catalogues.append(Catalogue(path.stem, builds, top.text("prepara"),
                                    top.count("nucleos", 4 if builds else 1), tuple(entries)))
    return catalogues, problems


def planted(root: Path, entry: Entry) -> tuple[dict[str, bytes], list[str]]:
    """Os arquivos de `entry` com o defeito dentro, e por que ele não se planta em `root`."""
    contents: dict[str, bytes] = {}
    problems = []
    for edit in entry.edits:
        path = root / edit.file
        if edit.new is not None:
            if os.path.lexists(path) or edit.file in contents:
                problems.append(f"`{edit.file}` já existe, e o arquivo novo tem de ser novo")
            contents[edit.file] = edit.new.encode()
            continue
        if edit.file not in contents:
            if path.is_symlink() or not path.is_file():
                problems.append(f"`{edit.file}` não existe")
                continue
            contents[edit.file] = path.read_bytes()
        found = contents[edit.file].count(edit.text.encode())
        if found != 1:
            problems.append(f"o texto de `{edit.file}` ocorre {found} vezes, e tem de ocorrer uma")
            continue
        contents[edit.file] = contents[edit.file].replace(edit.text.encode(), edit.swap.encode())
    return contents, problems


def check(root: Path) -> tuple[list[Catalogue], list[str]]:
    """A conferência estática: toda entrada tem forma e ainda se aplica uma vez."""
    catalogues, problems = load(root)
    seen: dict[str, str] = {}
    pending: dict[str, int] = {}
    for catalogue in catalogues:
        file = f"{catalogue.name}.toml"
        for entry in catalogue.entries:
            where = f"{CATALOGUE_DIR}/{file}: entrada `{entry.name}`"
            if entry.name in seen:
                problems.append(f"{where}: o id já existe em `{seen[entry.name]}`")
            seen[entry.name] = file
            problems.extend(f"{where}: {problem}" for problem in planted(root, entry)[1])
            if entry.pending is None:
                continue
            pending[entry.pending] = pending.get(entry.pending, 0) + 1
            if file != PENDING_FILE:
                problems.append(f"{where}: pendente só mora em `{PENDING_FILE}`, e nenhuma entrada nova nasce pendente")
            elif entry.pending not in PENDING_OWNERS:
                problems.append(f"{where}: `{entry.pending}` não é dono de pendente")
            elif entry.expects != "reprova":
                problems.append(f"{where}: pendente é defeito que ainda sobrevive, não forma legítima")
    for owner, limit in sorted(PENDING_OWNERS.items()):
        count = pending.get(owner, 0)
        if count != limit:
            problems.append(
                f"{CATALOGUE_DIR}/{PENDING_FILE}: {count} pendentes do {owner}, e o registro diz {limit}: "
                "a entrada que sai desce o número em `PENDING_OWNERS`, e nenhuma entra")
    return catalogues, problems


def digest(path: Path) -> str | None:
    """O conteúdo e o modo de `path`; `None` quando não é arquivo nem link."""
    if path.is_symlink():
        return "link:" + os.readlink(path)
    if not path.is_file():
        return None
    with path.open("rb") as file:
        return f"{path.stat().st_mode & 0o777:o}:{hashlib.file_digest(file, 'sha256').hexdigest()}"


@dataclasses.dataclass(frozen=True)
class Snapshot:
    """A árvore de trabalho num instante: cada arquivo com o seu `digest`."""
    files: dict[str, str]
    tracked: tuple[str, ...]
    others: tuple[str, ...]

    @classmethod
    def of(cls, root: Path) -> Snapshot:
        files = {}
        kinds = []
        for options in (("--cached",), ("--others", "--exclude-standard")):
            out = subprocess.run(["git", "ls-files", "-z", *options], cwd=root,
                                 capture_output=True, check=True).stdout
            present = []
            for name in filter(None, out.decode().split("\0")):
                mark = digest(root / name)
                if mark is not None:  # o apagado na árvore, que o índice ainda tem, fica de fora
                    files[name] = mark
                    present.append(name)
            kinds.append(tuple(present))
        return cls(files, kinds[0], kinds[1])


def remove(path: Path) -> None:
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink(missing_ok=True)


class Copy:
    """Uma cópia da árvore de trabalho, guardada entre as rodadas."""

    def __init__(self, source: Path, folder: Path):
        self.source = source
        self.name = folder.name
        self.tree = folder / "tree"
        self.manifest = folder / "manifest.json"
        self.status: list[str] = []
        # O `roda` que já rodou sem defeito nesta cópia: a saída de quem
        # passou e o problema de quem não.
        self.passed: dict[str, str] = {}
        self.failed: dict[str, str] = {}

    def git(self, *arguments: str, stdin: bytes | None = None) -> str:
        done = subprocess.run([*COPY_GIT, *arguments], cwd=self.tree, input=stdin,
                              capture_output=True, check=False)
        if done.returncode != 0:
            raise RuntimeError(f"`git {' '.join(arguments)}` falhou em {self.tree}:\n"
                               f"{done.stderr.decode(errors='replace')}")
        return done.stdout.decode(errors="replace")

    def state(self) -> list[str]:
        """O `git status` da cópia, uma linha por arquivo."""
        return sorted(line for line in self.git(*STATUS).split("\0") if line)

    def sync(self, snapshot: Snapshot) -> None:
        """Deixa a cópia igual a `snapshot`, gravando só o que o manifesto não garante."""
        try:
            known = json.loads(self.manifest.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            known = {}
        # Sem o manifesto, a sincronização interrompida recomeça do zero.
        self.manifest.unlink(missing_ok=True)
        expected = sorted(f"?? {name}" for name in snapshot.others)
        # O manifesto que não diz a verdade (a cópia mexida por fora) perde a
        # vez: a segunda volta grava tudo.
        for trusted in (known, {}):
            self.mirror(snapshot, trusted)
            self.status = self.state()
            if self.status == expected:
                break
        else:
            raise RuntimeError(f"a {self.name} não ficou igual à árvore de trabalho: "
                               + ", ".join(sorted(set(self.status) ^ set(expected))))
        self.manifest.write_text(json.dumps(snapshot.files), encoding="utf-8")

    def mirror(self, snapshot: Snapshot, known: dict[str, str]) -> None:
        self.tree.mkdir(parents=True, exist_ok=True)
        if not (self.tree / ".git").exists():
            self.git("init", "-q")
        for name in known.keys() - snapshot.files.keys():
            remove(self.tree / name)
        written = set()
        for name, mark in snapshot.files.items():
            target = self.tree / name
            if known.get(name) == mark and os.path.lexists(target):
                continue
            origin = self.source / name
            target.parent.mkdir(parents=True, exist_ok=True)
            remove(target)
            if origin.is_symlink():
                os.symlink(os.readlink(origin), target)
            else:
                # Nunca a hora da origem: o arquivo gravado é de agora.
                shutil.copyfile(origin, target)
                shutil.copymode(origin, target)
            written.add(name)
        # O índice da cópia só aprende o que acabou de vir da origem: o
        # arquivo mexido por fora continua diferente dele, e o `state` o vê.
        indexed = set(filter(None, self.git("ls-files", "-z", "--cached").split("\0")))
        tracked = set(snapshot.tracked)
        for option, names in (("--force-remove", indexed - tracked),
                              ("--add", (tracked & written) | (tracked - indexed))):
            if names:
                self.git("update-index", option, "-z", "--stdin",
                         stdin="\0".join(sorted(names)).encode())
        self.git("commit", "-q", "--allow-empty", "-m", "espelho")
        # O que uma rodada anterior deixou e a árvore de trabalho não tem.
        for name in self.git("ls-files", "-z", "--others", "--exclude-standard").split("\0"):
            if name and name not in snapshot.files:
                remove(self.tree / name)

    def dirty(self) -> str | None:
        """O que mudou na árvore da cópia desde a sincronização; `None` quando nada."""
        found = self.state()
        return None if found == self.status else ", ".join(sorted(set(found) ^ set(self.status)))

    def plant(self, contents: dict[str, bytes]) -> dict[str, bytes | None]:
        """Grava o defeito e devolve o que havia (`None`: nada)."""
        # O manifesto esquece o arquivo antes: a rodada interrompida com o
        # defeito dentro é consertada pela sincronização seguinte.
        known = json.loads(self.manifest.read_text(encoding="utf-8"))
        known.update(dict.fromkeys(contents, PLANTED))
        self.manifest.write_text(json.dumps(known), encoding="utf-8")
        before: dict[str, bytes | None] = {}
        for name, data in contents.items():
            target = self.tree / name
            before[name] = target.read_bytes() if target.is_file() else None
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        return before

    def restore(self, before: dict[str, bytes | None]) -> str | None:
        """Desfaz o `plant`; o problema quando a árvore não volta a ser a de antes."""
        for name, data in before.items():
            target = self.tree / name
            if data is None:
                target.unlink(missing_ok=True)
            else:
                target.write_bytes(data)
        return self.dirty()


def kill(process: subprocess.Popen) -> None:
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGKILL)
    process.wait()


class Processes:
    """Os processos de uma rodada: depois do `stop`, nenhum começa, e os que rodam morrem."""

    def __init__(self) -> None:
        self.running: set[subprocess.Popen] = set()
        self.lock = threading.Lock()
        self.stopped = False

    def run(self, command: str, cwd: Path, environment: dict[str, str], log: Path,
            deadline: int) -> tuple[int | None, str]:
        """Roda `command` com a saída no fim de `log`; o código e a saída.

        O código é `None` quando o comando passou do prazo, ou quando a rodada
        já foi encerrada e ele nem começou.
        """
        with log.open("ab") as output:
            output.write(f"$ {command}\n".encode())
            output.flush()
            start = output.tell()
            # Começar e entrar na conta é um gesto só, sob a trava do `stop`:
            # ele nunca perde o processo que nasce junto com ele.
            with self.lock:
                if self.stopped:
                    return None, ""
                process = subprocess.Popen(["bash", "-c", command], cwd=cwd, env=environment,
                                           stdin=subprocess.DEVNULL, stdout=output,
                                           stderr=subprocess.STDOUT, start_new_session=True)
                self.running.add(process)
            try:
                code = process.wait(timeout=deadline)
            except subprocess.TimeoutExpired:
                kill(process)
                code = None
            finally:
                with self.lock:
                    self.running.discard(process)
        with log.open("rb") as written:
            written.seek(start)
            return code, written.read().decode(errors="replace")

    def stop(self) -> None:
        with self.lock:
            self.stopped = True
            running = list(self.running)
        for process in running:
            kill(process)


class Round:
    """Um `--rodar`: as cópias, a fila das entradas e o que deu cada uma."""

    def __init__(self, root: Path, base: Path, catalogue: Catalogue, entries: list[Entry],
                 jobs: int):
        self.root = root
        self.base = base
        self.catalogue = catalogue
        self.entries = entries
        self.jobs = jobs
        self.logs = base / "logs" / catalogue.name
        self.waiting: queue.SimpleQueue[Entry] = queue.SimpleQueue()
        self.outcomes: list[Outcome] = []
        self.failures: list[str] = []
        self.processes = Processes()
        self.environment = dict(os.environ)
        # Com mais de uma cópia, cada uma fica com a sua parte dos núcleos:
        # quem ela roda só enxerga esses, e as cópias não disputam os mesmos.
        # Os nomes são os que o cargo, o CMake, o teste do Rust e o
        # orquestrador leem, para quem os herdou com a máquina inteira.
        self.shares: list[list[int]] = []
        if jobs > 1 and hasattr(os, "sched_getaffinity"):
            cores = sorted(os.sched_getaffinity(0))
            self.shares = [cores[index::jobs] or cores for index in range(jobs)]
            for name in ("CARGO_BUILD_JOBS", "CMAKE_BUILD_PARALLEL_LEVEL", "RUST_TEST_THREADS",
                         "KINEIN_TIDY_JOBS", "KINEIN_QML_JOBS"):
                self.environment[name] = str(max(1, len(cores) // jobs))

    def environment_of(self, copy: Copy) -> dict[str, str]:
        # A pasta de build é da cópia, mesmo que a pessoa tenha uma só para tudo.
        return {**self.environment, "CARGO_TARGET_DIR": str(copy.tree / "target")}

    def baseline(self, copy: Copy, entry: Entry) -> tuple[str, str] | None:
        """O `roda` da entrada passa sem o defeito, uma vez por cópia; o estado e o problema quando não."""
        command = entry.command
        if command not in copy.passed and command not in copy.failed:
            log = self.logs / f"base-{copy.name}-{len(copy.passed) + len(copy.failed) + 1}.log"
            log.unlink(missing_ok=True)
            started = time.monotonic()
            problem, output = None, ""
            for step in filter(None, (self.catalogue.builds, command)):
                code, output = self.processes.run(step, copy.tree, self.environment_of(copy),
                                                  log, BASE_DEADLINE)
                if code != 0:
                    reason = "passou do prazo" if code is None else f"saiu com {code}"
                    problem = f"sem defeito nenhum, `{step}` {reason}; log: {log}"
                    break
            written = None if problem else copy.dirty()
            if written:
                problem = f"sem defeito nenhum, `{command}` grava na árvore: {written}"
            if problem:
                copy.failed[command] = problem
            else:
                copy.passed[command] = output
                print(f"  · {copy.name}: a base passa em `{command}` "
                      f"({time.monotonic() - started:.1f} s)", flush=True)
        if command in copy.failed:
            return "base-falhou", copy.failed[command]
        for fragment in entry.fragments:
            if fragment in copy.passed[command]:
                return "trecho-na-base", (f"o trecho `{fragment}` aparece na saída sem defeito, "
                                          "e não distingue a reprovação")
        return None

    def verdict(self, copy: Copy, entry: Entry, log: Path) -> str:
        environment = self.environment_of(copy)
        if self.catalogue.builds:
            code, _ = self.processes.run(self.catalogue.builds, copy.tree, environment, log,
                                         entry.deadline)
            if code != 0:
                return "prazo" if code is None else "nao-compila"
        code, output = self.processes.run(entry.command, copy.tree, environment, log,
                                          entry.deadline)
        if code is None:
            return "prazo"
        if code == 0:
            return "passou" if entry.expects == "passa" else "sobreviveu"
        if entry.expects == "passa":
            return "falso-positivo"
        return "reprovado" if all(fragment in output for fragment in entry.fragments) else "outro-motivo"

    def judge(self, copy: Copy, entry: Entry) -> Outcome:
        started = time.monotonic()
        log = self.logs / f"{entry.name}.log"
        log.unlink(missing_ok=True)

        def done(state: str, problem: str | None = None) -> Outcome:
            if problem:
                self.failures.append(f"{entry.name}: {problem}")
            return Outcome(entry, state, time.monotonic() - started, log if log.exists() else None)

        refused = self.baseline(copy, entry)
        if refused:
            return done(*refused)
        started = time.monotonic()  # o tempo da entrada não conta a base
        contents, problems = planted(copy.tree, entry)
        if problems:
            return done("nao-aplica", "; ".join(problems))
        before = copy.plant(contents)
        try:
            state = self.verdict(copy, entry, log)
        finally:
            written = copy.restore(before)
        if written:
            return done("copia-suja", f"com o defeito, `{entry.command}` grava na árvore: {written}")
        return done(state)

    def work(self, index: int, snapshot: Snapshot) -> None:
        copy = Copy(self.root, self.base / f"copia-{index}")
        try:
            if self.shares:
                # A afinidade é da thread, e o processo que ela cria a herda.
                os.sched_setaffinity(0, self.shares[index - 1])
            copy.sync(snapshot)
            if self.catalogue.prepares:
                log = self.logs / f"prepara-{copy.name}.log"
                log.unlink(missing_ok=True)
                code, _ = self.processes.run(self.catalogue.prepares, copy.tree,
                                             self.environment_of(copy), log, BASE_DEADLINE)
                if code != 0:
                    self.failures.append(f"{copy.name}: `{self.catalogue.prepares}` falhou; log: {log}")
                    return
            while not self.processes.stopped:
                try:
                    entry = self.waiting.get_nowait()
                except queue.Empty:
                    return
                outcome = self.judge(copy, entry)
                if self.processes.stopped:
                    return  # o que a interrupção matou não é veredito
                self.outcomes.append(outcome)
                print(f"  {'✓' if outcome.expected else '✗'} {entry.name}: {outcome.state} "
                      f"({outcome.seconds:.1f} s)", flush=True)
                if outcome.state == "copia-suja":
                    # O manifesto não vale mais para esta cópia: grava tudo.
                    copy.manifest.unlink(missing_ok=True)
                    copy.sync(snapshot)
        except Exception:  # noqa: BLE001 - a cópia que para reprova a rodada, com o motivo
            self.failures.append(f"a {copy.name} parou:\n{traceback.format_exc()}")

    def go(self) -> list[Outcome]:
        self.logs.mkdir(parents=True, exist_ok=True)
        snapshot = Snapshot.of(self.root)
        for entry in self.entries:
            self.waiting.put(entry)
        with concurrent.futures.ThreadPoolExecutor(self.jobs) as pool:
            futures = [pool.submit(self.work, index, snapshot) for index in range(1, self.jobs + 1)]
            try:
                for future in futures:
                    future.result()
            except KeyboardInterrupt:
                self.processes.stop()
                raise
        # Toda entrada roda: a que ficou sem cópia reprova a rodada.
        ran = {outcome.entry.name for outcome in self.outcomes}
        self.outcomes.extend(Outcome(entry, "nao-rodou", 0.0, None)
                             for entry in self.entries if entry.name not in ran)
        if Snapshot.of(self.root).files != snapshot.files:
            self.failures.append("a árvore de trabalho mudou durante a rodada, que julgou a de antes: rode de novo")
        return self.outcomes


def report(outcomes: list[Outcome], failures: list[str], seconds: float, processor: float,
           jobs: int) -> bool:
    """Imprime a rodada por regra; `True` quando toda entrada deu o esperado."""
    rules: dict[str, list[Outcome]] = {}
    for outcome in outcomes:
        rules.setdefault(outcome.entry.rule, []).append(outcome)
    width = max([len(rule) for rule in rules] + [len("regra")])
    print(f"\n{'regra':{width}}  plantados  reprovados  pendentes  legítimas  passaram")
    for rule, group in sorted(rules.items()):
        defects = [o for o in group if o.entry.expects == "reprova" and not o.entry.pending]
        waiting = [o for o in group if o.entry.pending]
        forms = [o for o in group if o.entry.expects == "passa" and not o.entry.pending]
        print(f"{rule:{width}}  {len(defects):9}  {sum(o.state == 'reprovado' for o in defects):10}"
              f"  {len(waiting):9}  {len(forms):9}  {sum(o.state == 'passou' for o in forms):8}")
    wrong = sorted((o for o in outcomes if not o.expected), key=lambda o: o.entry.name)
    for outcome in wrong:
        entry = outcome.entry
        hint = ""
        if entry.pending and outcome.state in ("reprovado", "outro-motivo"):
            # O pendente não tem trecho: a ferramenta não sabe quem reprovou.
            hint = (f": se quem reprova é o defeito, a entrada sai de `{PENDING_FILE}` para o catálogo "
                    f"do {entry.pending}; se é um teste que não o vê, o teste é instável")
        elif outcome.state == "outro-motivo":
            hint = ": reprovou sem o trecho esperado (" + "; ".join(f"`{f}`" for f in entry.fragments) + ")"
        print(f"✗ {entry.name}: {outcome.state}{hint}" + (f"; log: {outcome.log}" if outcome.log else ""))
    for failure in failures:
        print(f"✗ {failure}")
    held = sum(1 for o in outcomes if o.entry.pending and o.expected)
    print(f"mutantes: {len(outcomes)} entradas em {seconds:.0f} s ({processor:.0f} s de processador, "
          f"{processor / max(seconds, 0.001):.1f} núcleos em média), {jobs} cópia(s); "
          f"{len(outcomes) - len(wrong)} como o esperado" + (f", {held} delas pendentes" if held else ""))
    return not wrong and not failures


def copies_folder(root: Path, chosen: str | None) -> Path:
    if chosen:
        return Path(chosen).resolve()
    if os.environ.get("KINEIN_MUTANTS_DIR"):
        return Path(os.environ["KINEIN_MUTANTS_DIR"]).resolve()
    cache = Path(os.environ.get("XDG_CACHE_HOME") or Path.home() / ".cache")
    # Um checkout, uma pasta: dois clones não dividem as cópias.
    return cache / "kinein-vectis" / "mutantes" / f"{root.name}-{hashlib.sha256(str(root).encode()).hexdigest()[:8]}"


def run_round(root: Path, catalogue: Catalogue, entries: list[Entry], jobs: int, base: Path) -> int:
    import fcntl  # só o `--rodar` precisa dele, e ele só existe no Unix

    # Quem encerra a rodada (o prazo de quem a chamou, o terminal fechado)
    # vale como o Ctrl-C: os processos dela, cada um na sua sessão, morrem
    # com ela. O do Ctrl-C é posto de novo para valer também em segundo plano.
    for ending in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        signal.signal(ending, signal.default_int_handler)
    base.mkdir(parents=True, exist_ok=True)
    with (base / "em-uso").open("w") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print(f"erro: outra rodada usa as cópias de {base}", file=sys.stderr)
            return 2
        print(f"== mutantes: {catalogue.name} ({len(entries)} entradas, {jobs} cópia(s) em {base}) ==",
              flush=True)
        started = time.monotonic()
        before = os.times()
        current = Round(root, base, catalogue, entries, jobs)
        try:
            outcomes = current.go()
        except KeyboardInterrupt:
            print("\nmutantes: rodada interrompida; a próxima conserta as cópias.", file=sys.stderr)
            return 130
        after = os.times()
        processor = (after.children_user + after.children_system
                     - before.children_user - before.children_system)
        return 0 if report(outcomes, current.failures, time.monotonic() - started, processor, jobs) else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n", 1)[0])
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--conferir", action="store_true", dest="check",
                        help="confere, sem rodar, que toda entrada tem forma e ainda se aplica uma vez")
    action.add_argument("--rodar", metavar="CATALOGO", dest="run",
                        help="planta cada defeito do catálogo numa cópia e roda quem o reprova")
    action.add_argument("--listar", action="store_true", dest="list",
                        help="lista os catálogos e as entradas")
    parser.add_argument("--so", metavar="ID", action="append", dest="only", default=[],
                        help="com --rodar: só esta entrada (pode repetir)")
    parser.add_argument("--jobs", type=int, help="quantas cópias rodam ao mesmo tempo")
    parser.add_argument("--copias", metavar="PASTA", dest="copies",
                        help="onde ficam as cópias (padrão: KINEIN_MUTANTS_DIR, ou o cache da pessoa)")
    parser.add_argument("--raiz", metavar="PASTA", dest="root", default=str(ROOT),
                        help="o repositório (padrão: este)")
    options = parser.parse_args()
    root = Path(options.root).resolve()

    catalogues, problems = check(root)
    if options.list:
        for catalogue in catalogues:
            print(f"{catalogue.name}: {len(catalogue.entries)} entradas")
            for entry in catalogue.entries:
                kind = f"pendente do {entry.pending}" if entry.pending else entry.expects
                print(f"  {entry.name}  [{entry.rule}]  {kind}")
    if problems:
        print("catálogo de mutantes FALHOU:", file=sys.stderr)
        for problem in problems:
            print(f"  ✗ {problem}", file=sys.stderr)
        print("\nA entrada que não se aplica mais não prova nada: acerte o texto ao código de hoje,"
              "\nou apague a entrada junto com a guarda que ela provava.", file=sys.stderr)
        return 1
    if options.list:
        return 0
    if options.check:
        total = sum(len(catalogue.entries) for catalogue in catalogues)
        waiting = sum(1 for catalogue in catalogues for entry in catalogue.entries if entry.pending)
        print(f"mutantes: {total} entradas em {len(catalogues)} catálogo(s), cada uma se aplica uma vez"
              + (f"; {waiting} delas pendentes." if waiting else "."))
        return 0

    chosen = next((catalogue for catalogue in catalogues if catalogue.name == options.run), None)
    if chosen is None:
        print(f"erro: não há catálogo `{options.run}` em {CATALOGUE_DIR}/ "
              f"(há: {', '.join(catalogue.name for catalogue in catalogues) or 'nenhum'})", file=sys.stderr)
        return 2
    entries = [entry for entry in chosen.entries if not options.only or entry.name in options.only]
    missing = sorted(set(options.only) - {entry.name for entry in entries})
    if missing or not entries:
        print(f"erro: `{chosen.name}` não tem " + (f"a entrada {', '.join(missing)}" if missing else "entradas"),
              file=sys.stderr)
        return 2
    base = copies_folder(root, options.copies)
    if base == root or root in base.parents:
        print(f"erro: as cópias ficam fora do repositório, e {base} está dentro dele", file=sys.stderr)
        return 2
    cores = len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else os.cpu_count() or 1
    jobs = options.jobs or max(1, cores // chosen.cores)
    return run_round(root, chosen, entries, max(1, min(jobs, len(entries))), base)


if __name__ == "__main__":
    sys.exit(main())
