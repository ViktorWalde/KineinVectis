#!/usr/bin/env python3
"""Prova a ferramenta de mutação num repositório de mentira (roadmaps/63, ENG-01).

A ferramenta julga os defeitos plantados; aqui os defeitos são dela. O
repositório de cada teste tem um arquivo guardado (`limit.conf`), um build
que confia na hora do arquivo, como o make e o cargo, e o teste da guarda
(`check.py`). Cada caso monta um catálogo e roda a ferramenta de fora, como o
gate roda: o que vale é o código de saída e o que ela imprime.

Só "reprovado" serve: o defeito que sobrevive, o que não compila, o que
reprova por outro motivo e a forma legítima que reprova têm de reprovar a
rodada. E o que ela promete do disco: o checkout intocado, a cópia que segue
a árvore de trabalho, a rodada interrompida consertada pela seguinte, e o
build velho que nunca julga a entrada seguinte. A rodada encerrada não deixa
processo, e nenhum começa depois.
"""
import concurrent.futures
import fcntl
import hashlib
import importlib.util
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path

from unproven import record

TOOL = Path(__file__).resolve().parent / "mutantes.py"
spec = importlib.util.spec_from_file_location("planted_defects", TOOL)
tool = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = tool  # as classes de dados procuram o módulo pelo nome
spec.loader.exec_module(tool)

# Nem a configuração do git da pessoa nem a pasta de cópias dela (R-TEST-04).
ENVIRONMENT = {**os.environ, "GIT_CONFIG_GLOBAL": os.devnull, "GIT_CONFIG_NOSYSTEM": "1"}
ENVIRONMENT.pop("KINEIN_MUTANTS_DIR", None)

BUILD = '''\
import os, re, shutil, sys
source, built = "limit.conf", "out/limit.conf"
for line in open(source, encoding="utf-8"):
    if not line.startswith("#") and not re.fullmatch(r"[a-z]+ = [0-9]+\\n", line):
        sys.exit(f"build: {source} has a line that is not `name = number`")
os.makedirs("out", exist_ok=True)
if not os.path.exists(built) or os.stat(source).st_mtime_ns > os.stat(built).st_mtime_ns:
    shutil.copyfile(source, built)
'''
CHECK = '''\
import sys
values = dict(line.split(" = ") for line in open("out/limit.conf", encoding="utf-8") if " = " in line)
if int(values["limit"]) != 10:
    sys.exit(f"check: the limit is {int(values['limit'])}, and the guard holds 10")
print("check: the guard holds the limit")
'''
PENDING = sum(tool.PENDING_OWNERS.values())
FILES = {
    ".gitignore": "out/\n*.log\n",
    "limit.conf": "# the limit of the guard\nlimit = 10\nspare = 3\n",
    "notes.txt": "the guard holds the limit\n",
    "trace.log": "tracked, though the ignore file names it\n",
    "pending.conf": "".join(f"pending{index:02} = 1\n" for index in range(1, PENDING + 1)),
    "build.py": BUILD,
    "check.py": CHECK,
}
HEAD = 'item = "FIX"\ncompila = "python3 -B build.py"\nroda = "python3 -B check.py"\n'


def entry(name: str, text: str, swap: str, *, file: str = "limit.conf",
          message: str | None = None, extra: str = "") -> str:
    lines = ["[[entrada]]", f'id = "{name}"', 'regra = "R-FIX-01"', extra]
    if message is not None:
        lines.append(f"mensagem = {json.dumps(message)}")
    lines += ["[[entrada.edicao]]", f'arquivo = "{file}"',
              f"texto = {json.dumps(text)}", f"troca = {json.dumps(swap)}", ""]
    return "\n".join(line for line in lines if line is not None)


def pending(reproved: bool = False) -> str:
    """O `pendentes.toml` do repositório de mentira; com `reproved`, o primeiro já reprova."""
    owners = [owner for owner, count in sorted(tool.PENDING_OWNERS.items()) for _ in range(count)]
    entries = [entry(f"fix-pending-{index:02}", f"pending{index:02} = 1", f"pending{index:02} = 2",
                     file="pending.conf", extra=f'pendente = "{owner}"')
               for index, owner in enumerate(owners, 1)]
    if reproved:
        entries[0] = entry("fix-pending-01", "limit = 10", "limit = 11",
                           extra=f'pendente = "{owners[0]}"')
    return HEAD + "".join(entries)


DEFECT = entry("fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11")
LEGITIMATE = entry("fix-notes", "holds the limit", "holds a limit", file="notes.txt",
                   extra='espera = "passa"')


class Fixture:
    """O repositório de mentira, com um commit, e a pasta das cópias ao lado."""

    def __init__(self, folder: str):
        self.root = Path(folder) / "repo"
        self.copies = Path(folder) / "copias"
        self.tree = self.copies / "copia-1" / "tree"
        for name, body in FILES.items():
            self.write(name, body)
        self.catalogue("pendentes", pending())
        self.git("init", "-q")
        self.git("add", "-A")
        self.git("add", "--force", "trace.log")
        self.git("-c", "user.name=fixture", "-c", "user.email=fixture@localhost",
                 "commit", "-q", "-m", "fixture")

    def write(self, name: str, body: str) -> None:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8")

    def catalogue(self, name: str, body: str) -> None:
        self.write(f"scripts/mutantes/{name}.toml", body)

    def git(self, *arguments: str) -> str:
        return subprocess.run(["git", *arguments], cwd=self.root, env=ENVIRONMENT,
                              capture_output=True, text=True, check=True).stdout

    def command(self, *arguments: str) -> list[str]:
        return [sys.executable, str(TOOL), "--raiz", str(self.root),
                "--copias", str(self.copies), *arguments]

    def tool(self, *arguments: str) -> subprocess.CompletedProcess:
        return subprocess.run(self.command(*arguments), env=ENVIRONMENT,
                              capture_output=True, text=True, check=False)

    def contents(self) -> dict[str, str]:
        """Cada arquivo do repositório, fora o `.git`, com o resumo do conteúdo."""
        return {path.relative_to(self.root).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                for path in sorted(self.root.rglob("*"))
                if path.is_file() and ".git" not in path.relative_to(self.root).parts}


class Case(unittest.TestCase):
    def setUp(self) -> None:
        scratch = tempfile.TemporaryDirectory(prefix="kinein-mutantes-")
        self.addCleanup(scratch.cleanup)
        self.fixture = Fixture(scratch.name)

    def run_round(self, *entries: str, head: str = HEAD, arguments: tuple[str, ...] = ()):
        self.fixture.catalogue("fix", head + "".join(entries))
        return self.fixture.tool("--rodar", "fix", *arguments)

    def held(self) -> tuple[str, Path, Path, Path]:
        """Um `roda` que, com o defeito, avisa no primeiro arquivo, espera o segundo e
        só então reprova, deixando o terceiro."""
        waiting, release, released = (self.fixture.root.parent / name
                                      for name in ("waiting", "release", "released"))
        command = (f"python3 -B check.py || {{ touch '{waiting}'; while [ ! -e '{release}' ]; "
                   f"do sleep 0.02; done; touch '{released}'; exit 1; }}")
        return command, waiting, release, released

    def wait_for(self, condition) -> None:
        deadline = time.monotonic() + 30
        while not condition():
            self.assertLess(time.monotonic(), deadline, "a condição não chegou em 30 s")
            time.sleep(0.02)

    def assert_done(self, done: subprocess.CompletedProcess, code: int, *fragments: str) -> None:
        shown = done.stdout + done.stderr
        self.assertEqual(done.returncode, code, shown)
        for fragment in fragments:
            self.assertIn(fragment, shown)


class TheStaticCheck(Case):
    """O `--conferir`: a forma de cada entrada e o texto que ainda se aplica uma vez."""

    def check(self, *entries: str, head: str = HEAD) -> subprocess.CompletedProcess:
        self.fixture.catalogue("fix", head + "".join(entries))
        return self.fixture.tool("--conferir")

    def test_a_sound_catalogue_passes(self) -> None:
        done = self.check(DEFECT, LEGITIMATE)
        self.assert_done(done, 0, f"{PENDING + 2} entradas em 2 catálogo(s)",
                         f"{PENDING} delas pendentes")
        owner = sorted(tool.PENDING_OWNERS)[0]
        self.assert_done(self.fixture.tool("--listar"), 0, "fix: 2 entradas",
                         "  fix-limit  [R-FIX-01]  reprova", "  fix-notes  [R-FIX-01]  passa",
                         f"  fix-pending-01  [R-FIX-01]  pendente do {owner}")

    def test_a_text_that_occurs_twice_or_never_fails(self) -> None:
        twice = entry("fix-twice", "limit", "bound", message="check")
        self.assert_done(self.check(twice), 1, "entrada `fix-twice`",
                         "o texto de `limit.conf` ocorre 2 vezes")
        gone = entry("fix-gone", "limit = 99", "limit = 11", message="check")
        self.assert_done(self.check(gone), 1, "o texto de `limit.conf` ocorre 0 vezes")
        absent = entry("fix-absent", "limit = 10", "limit = 11", file="other.conf", message="check")
        self.assert_done(self.check(absent), 1, "`other.conf` não existe")

    def test_the_second_edit_sees_the_first(self) -> None:
        chained = DEFECT + '[[entrada.edicao]]\narquivo = "limit.conf"\ntexto = "limit = 11"\ntroca = "limit = 12"\n'
        self.assert_done(self.check(chained), 0)

    def test_a_new_file_has_to_be_new(self) -> None:
        new = ('[[entrada]]\nid = "fix-new"\nregra = "R-FIX-01"\nmensagem = "check"\n'
               '[[entrada.edicao]]\narquivo = "{}"\nnovo = "limit = 11"\n')
        self.assert_done(self.check(new.format("fresh.conf")), 0)
        self.assert_done(self.check(new.format("limit.conf")), 1, "`limit.conf` já existe")

    def test_the_shape_of_an_entry_is_checked(self) -> None:
        for body, problem in (
            (entry("fix-limit", "limit = 10", "limit = 11"), "quem reprova diz em `mensagem`"),
            (DEFECT.replace("mensagem", "mensagen"), "`mensagen` não é campo do catálogo"),
            (DEFECT.replace('"R-FIX-01"', '""'), "falta `regra`"),
            (DEFECT.replace("troca", "trocas"), "`trocas` não é campo do catálogo"),
            (DEFECT.replace('troca = "limit = 11"', 'troca = "limit = 10"'), "igual à `troca`"),
            (DEFECT.replace('"limit.conf"', '"../limit.conf"'), "caminho dentro do repositório"),
            (DEFECT.split("[[entrada.edicao]]")[0], "sem edição não há defeito plantado"),
            (DEFECT + 'espera = "talvez"\n', "não é campo do catálogo"),
            (entry("fix-limit", "limit = 10", "limit = 11", message="x", extra='espera = "talvez"'),
             "`espera` é `reprova` ou `passa`"),
            (LEGITIMATE.replace("[[entrada.edicao]]", 'mensagem = "check"\n[[entrada.edicao]]'),
             "a forma legítima passa, e não tem `mensagem`"),
            (DEFECT + DEFECT, "o id já existe em `fix.toml`"),
            ("[[entrada]\n", "não é TOML válido"),
        ):
            with self.subTest(problem=problem):
                self.assert_done(self.check(body), 1, problem)

    def test_an_entry_without_a_runner_fails(self) -> None:
        self.assert_done(self.check(DEFECT, head='item = "FIX"\n'), 1, "falta `roda`")

    def test_pending_lives_in_its_file_and_its_count_is_exact(self) -> None:
        owner = sorted(tool.PENDING_OWNERS)[0]
        count = tool.PENDING_OWNERS[owner]
        elsewhere = entry("fix-spare", "spare = 3", "spare = 4", extra=f'pendente = "{owner}"')
        self.assert_done(self.check(elsewhere), 1, "pendente só mora em `pendentes.toml`",
                         f"{count + 1} pendentes do {owner}, e o registro diz {count}")
        self.fixture.catalogue("pendentes", pending().replace(
            f'pendente = "{owner}"', 'pendente = "ENG-99"', 1))
        self.assert_done(self.check(DEFECT), 1, "`ENG-99` não é dono de pendente",
                         f"{count - 1} pendentes do {owner}, e o registro diz {count}")
        self.fixture.catalogue("pendentes", pending().replace(
            f'pendente = "{owner}"', f'pendente = "{owner}"\nespera = "passa"', 1))
        self.assert_done(self.check(DEFECT), 1, "pendente é defeito que ainda sobrevive")


class TheRound(Case):
    """O `--rodar`: o veredito de cada entrada, e só "reprovado" serve."""

    def test_a_reproved_defect_and_a_legitimate_form_pass_the_round(self) -> None:
        done = self.run_round(DEFECT, LEGITIMATE)
        self.assert_done(done, 0, "✓ fix-limit: reprovado", "✓ fix-notes: passou",
                         "2 como o esperado")
        row = next(line for line in done.stdout.splitlines() if line.startswith("R-FIX-01"))
        self.assertEqual(row.split(), ["R-FIX-01", "1", "1", "0", "1", "1"])

    def test_a_survivor_fails_the_round(self) -> None:
        survivor = entry("fix-spare", "spare = 3", "spare = 4", message="check: the limit")
        done = self.run_round(DEFECT, survivor)
        self.assert_done(done, 1, "✓ fix-limit: reprovado", "✗ fix-spare: sobreviveu",
                         "1 como o esperado")

    def test_a_defect_that_does_not_build_fails_the_round(self) -> None:
        broken = entry("fix-broken", "limit = 10", "limit = ten", message="build: limit.conf")
        self.assert_done(self.run_round(broken), 1, "✗ fix-broken: nao-compila")

    def test_a_defect_reproved_for_another_reason_fails_the_round(self) -> None:
        other = entry("fix-limit", "limit = 10", "limit = 11", message="check: the limit is 12")
        self.assert_done(self.run_round(other), 1, "✗ fix-limit: outro-motivo",
                         "reprovou sem o trecho esperado (`check: the limit is 12`)")
        both = DEFECT.replace('"check: the limit is 11"', '["check: the limit is 11", "holds 12"]')
        self.assert_done(self.run_round(both), 1, "✗ fix-limit: outro-motivo")

    def test_a_legitimate_form_that_is_reproved_fails_the_round(self) -> None:
        refused = entry("fix-limit", "limit = 10", "limit = 11", extra='espera = "passa"')
        self.assert_done(self.run_round(refused, LEGITIMATE), 1, "✗ fix-limit: falso-positivo",
                         "✓ fix-notes: passou")

    def test_a_runner_that_is_missing_or_fails_without_the_defect_fails_the_round(self) -> None:
        for runner, reason in (("no-such-runner --check", "saiu com 127"), ("false", "saiu com 1")):
            with self.subTest(runner=runner):
                missing = entry("fix-limit", "limit = 10", "limit = 11",
                                message="check: the limit is 11",
                                extra=f'roda = "{runner}"')
                self.assert_done(self.run_round(missing), 1, "✗ fix-limit: base-falhou",
                                 f"sem defeito nenhum, `{runner}` {reason}")
        self.assert_done(self.run_round(DEFECT, head=HEAD.replace("build.py", "missing.py")), 1,
                         "✗ fix-limit: base-falhou", "sem defeito nenhum, `python3 -B missing.py` saiu com")

    def test_a_fragment_that_the_passing_output_has_fails_the_round(self) -> None:
        loose = entry("fix-limit", "limit = 10", "limit = 11", message="the guard holds")
        self.assert_done(self.run_round(loose), 1, "✗ fix-limit: trecho-na-base",
                         "o trecho `the guard holds` aparece na saída sem defeito")

    def test_a_command_past_its_deadline_is_killed_and_fails_the_round(self) -> None:
        sleeper = self.fixture.root.parent / "sleeper"
        slow = entry("fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
                     extra="prazo = 3\nroda = " + json.dumps(
                         f"python3 -B check.py || {{ echo $$ > '{sleeper}'; exec sleep 12; }}"))
        self.assert_done(self.run_round(slow), 1, "✗ fix-limit: prazo")
        with self.assertRaises(ProcessLookupError):
            os.kill(int(sleeper.read_text(encoding="utf-8")), 0)

    def test_a_stopped_round_takes_its_processes_with_it(self) -> None:
        sleeper = self.fixture.root.parent / "sleeper"
        self.fixture.catalogue("fix", HEAD + entry(
            "fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
            extra="roda = " + json.dumps(
                f"python3 -B check.py || {{ echo $$ > '{sleeper}'; exec sleep 60; }}")))
        # Sem `subTest`: a rodada que não sai com o primeiro sinal já reprova o caso.
        for ending in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            sleeper.unlink(missing_ok=True)
            running = subprocess.Popen(self.fixture.command("--rodar", "fix"), env=ENVIRONMENT,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            try:
                self.wait_for(lambda: sleeper.exists() and sleeper.read_text(encoding="utf-8").strip())
                running.send_signal(ending)
                shown, _ = running.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                self.fail(f"{ending.name}: a rodada não saiu em 10 s")
            finally:
                running.kill()
                running.wait()
            self.assertEqual(running.returncode, 130, f"{ending.name}: {shown}")
            self.assertIn("rodada interrompida", shown, ending.name)
            # O que a interrupção matou não aparece como veredito.
            self.assertNotIn("fix-limit:", shown, ending.name)
            with self.assertRaises(ProcessLookupError, msg=ending.name):
                os.kill(int(sleeper.read_text(encoding="utf-8")), 0)

    def test_each_copy_has_its_share_of_the_cores(self) -> None:
        mine = os.sched_getaffinity(0)
        if len(mine) < 2:
            record("test_mutantes", "a parte dos núcleos de cada cópia do --rodar",
                   "esta rodada tem um núcleo só, e não há o que dividir")
        folder = self.fixture.root.parent
        shown = ("import os, sys; open(sys.argv[1], 'w').write("
                 "' '.join(map(str, sorted(os.sched_getaffinity(0)))))")
        records = (f"python3 -B check.py || {{ python3 -B -c \"{shown}\" "
                   f"'{folder}'/share-$(basename \"$(dirname \"$PWD\")\"); exit 1; }}")
        entries = [entry(f"fix-limit-{index}", "limit = 10", "limit = 11",
                         message="check: the limit is 11", extra=f"roda = {json.dumps(records)}")
                   for index in (1, 2)]
        self.assert_done(self.run_round(*entries, arguments=("--jobs", "2")), 0)
        shares = [set(map(int, path.read_text(encoding="utf-8").split()))
                  for path in sorted(folder.glob("share-copia-*"))]
        self.assertTrue(shares)
        for share in shares:
            self.assertLessEqual(share, mine)
            self.assertIn(len(share), {max(1, len(mine) // 2), (len(mine) + 1) // 2})
        if len(shares) == 2 and len(mine) > 1:
            self.assertFalse(shares[0] & shares[1])
        # Uma cópia só fica com a máquina que a rodada tem.
        self.assert_done(self.run_round(entries[0], arguments=("--jobs", "1")), 0)
        alone = set(map(int, (folder / "share-copia-1").read_text(encoding="utf-8").split()))
        self.assertEqual(alone, mine)

    def test_a_command_that_writes_in_the_tree_fails_the_round(self) -> None:
        writes = entry("fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
                       extra='roda = "python3 -B check.py && touch written.txt"')
        self.assert_done(self.run_round(writes), 1, "✗ fix-limit: base-falhou",
                         "grava na árvore: ?? written.txt")
        # Só com o defeito: a entrada reprova a rodada, e a cópia é refeita
        # antes da entrada seguinte.
        leaks = entry("fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
                      extra='roda = "python3 -B check.py || { touch leaked.txt; exit 1; }"')
        done = self.run_round(leaks, LEGITIMATE, arguments=("--jobs", "1"))
        self.assert_done(done, 1, "✗ fix-limit: copia-suja", "grava na árvore: ?? leaked.txt",
                         "✓ fix-notes: passou")
        self.assertFalse((self.fixture.tree / "leaked.txt").exists())
        self.assertFalse((self.fixture.tree / "written.txt").exists())

    def test_every_entry_runs_or_the_round_fails(self) -> None:
        ready = HEAD + 'prepara = "mkdir -p out"\n'
        self.assert_done(self.run_round(DEFECT, head=ready), 0, "✓ fix-limit: reprovado")
        stuck = HEAD + 'prepara = "false"\n'
        self.assert_done(self.run_round(DEFECT, LEGITIMATE, head=stuck), 1, "`false` falhou",
                         "✗ fix-limit: nao-rodou", "✗ fix-notes: nao-rodou", "0 como o esperado")

    def test_only_the_named_entries_run(self) -> None:
        done = self.run_round(DEFECT, LEGITIMATE, arguments=("--so", "fix-notes"))
        self.assert_done(done, 0, "✓ fix-notes: passou", "1 como o esperado")
        self.assertNotIn("fix-limit", done.stdout)
        self.assert_done(self.run_round(DEFECT, arguments=("--so", "fix-other")), 2,
                         "`fix` não tem a entrada fix-other")
        self.assert_done(self.fixture.tool("--rodar", "other"), 2, "não há catálogo `other`")

    def test_a_catalogue_out_of_shape_does_not_run(self) -> None:
        twice = entry("fix-twice", "limit", "bound", message="check")
        self.assert_done(self.run_round(twice), 1, "ocorre 2 vezes")
        self.assertFalse(self.fixture.copies.exists())

    def test_a_pending_survivor_is_expected_and_a_reproved_one_leaves(self) -> None:
        self.assert_done(self.fixture.tool("--rodar", "pendentes"), 0,
                         f"{PENDING} como o esperado, {PENDING} delas pendentes")
        self.fixture.catalogue("pendentes", pending(reproved=True))
        self.assert_done(self.fixture.tool("--rodar", "pendentes"), 1,
                         "✗ fix-pending-01: reprovado: se quem reprova é o defeito, a entrada sai de "
                         "`pendentes.toml`", "se é um teste que não o vê, o teste é instável",
                         f"{PENDING - 1} como o esperado")


class TheProcesses(unittest.TestCase):
    """O que a rodada encerrada promete dos processos, por dentro: a corrida entre o sinal e o
    processo que nasce não se reproduz de fora."""

    def test_a_stopped_round_starts_no_process(self) -> None:
        with tempfile.TemporaryDirectory(prefix="kinein-mutantes-") as scratch:
            folder = Path(scratch)
            marker = folder / "marker"
            processes = tool.Processes()

            def touch() -> tuple[int | None, str]:
                return processes.run(f"touch '{marker}'", folder, dict(os.environ), folder / "log", 30)

            self.assertEqual(touch(), (0, ""))
            self.assertTrue(marker.exists())
            marker.unlink()
            processes.stop()
            self.assertEqual(touch(), (None, ""))
            self.assertFalse(marker.exists())


class TheDisk(Case):
    """O que o `--rodar` promete do disco: o checkout, as cópias e a hora dos arquivos."""

    def test_the_checkout_is_untouched(self) -> None:
        self.fixture.catalogue("fix", HEAD + DEFECT + LEGITIMATE
                               + entry("fix-spare", "spare = 3", "spare = 4",
                                       message="check: the limit is"))
        self.fixture.write("untracked.txt", "new, and not ignored\n")
        before = (self.fixture.contents(), self.fixture.git("status", "--porcelain"))
        self.assert_done(self.fixture.tool("--rodar", "fix"), 1, "✗ fix-spare: sobreviveu")
        self.assertEqual((self.fixture.contents(), self.fixture.git("status", "--porcelain")), before)
        self.assertFalse((self.fixture.root / "out").exists())
        self.assertEqual((self.fixture.tree / "untracked.txt").read_text(encoding="utf-8"),
                         "new, and not ignored\n")

    def test_the_copies_stay_outside_the_repository(self) -> None:
        self.fixture.catalogue("fix", HEAD + DEFECT)
        inside = subprocess.run(
            [sys.executable, str(TOOL), "--raiz", str(self.fixture.root),
             "--copias", str(self.fixture.root / "copias"), "--rodar", "fix"],
            env=ENVIRONMENT, capture_output=True, text=True, check=False)
        self.assert_done(inside, 2, "as cópias ficam fora do repositório")
        self.assertFalse((self.fixture.root / "copias").exists())

    def test_two_rounds_do_not_share_the_copies(self) -> None:
        self.fixture.catalogue("fix", HEAD + DEFECT)
        self.fixture.copies.mkdir()
        with (self.fixture.copies / "em-uso").open("w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            self.assert_done(self.fixture.tool("--rodar", "fix"), 2, "outra rodada usa as cópias")
        self.assert_done(self.fixture.tool("--rodar", "fix"), 0)

    def test_a_stale_build_never_judges_the_next_entry(self) -> None:
        # O arquivo da origem é mais antigo que qualquer build.
        os.utime(self.fixture.root / "limit.conf", (978307200, 978307200))
        # Dentro da rodada: a forma legítima vem depois do defeito e não mexe
        # no arquivo dele, e só passa se a volta do defeito refez o build.
        done = self.run_round(DEFECT, LEGITIMATE, arguments=("--jobs", "1"))
        self.assert_done(done, 0, "✓ fix-limit: reprovado", "✓ fix-notes: passou")
        # Entre as rodadas: a anterior acaba no build do defeito, e a base da
        # seguinte só passa se o arquivo que volta da origem é de agora.
        for _ in range(2):
            done = self.run_round(LEGITIMATE, DEFECT, arguments=("--jobs", "1"))
            self.assert_done(done, 0, "✓ fix-notes: passou", "✓ fix-limit: reprovado")

    def test_the_copy_follows_the_working_tree_and_keeps_what_did_not_change(self) -> None:
        self.assert_done(self.run_round(DEFECT), 0)
        self.assertEqual((self.fixture.tree / "trace.log").read_text(encoding="utf-8"),
                         FILES["trace.log"])
        kept = (self.fixture.tree / "build.py").stat().st_mtime_ns
        self.fixture.write("limit.conf", FILES["limit.conf"].replace("spare = 3", "spare = 5"))
        self.fixture.write("fresh.txt", "new\n")
        (self.fixture.root / "notes.txt").unlink()
        (self.fixture.root / "trace.log").unlink()
        os.chmod(self.fixture.root / "check.py", 0o755)
        moved = entry("fix-spare", "spare = 5", "spare = 6", extra='espera = "passa"')
        self.assert_done(self.run_round(DEFECT, moved), 0, "✓ fix-spare: passou")
        self.assertEqual((self.fixture.tree / "fresh.txt").read_text(encoding="utf-8"), "new\n")
        self.assertFalse((self.fixture.tree / "notes.txt").exists())
        # O rastreado que o `.gitignore` ignora também sai: o git da cópia não o mostra.
        self.assertFalse((self.fixture.tree / "trace.log").exists())
        self.assertTrue(os.access(self.fixture.tree / "check.py", os.X_OK))
        self.assertEqual((self.fixture.tree / "build.py").stat().st_mtime_ns, kept)

    def test_a_copy_changed_from_outside_is_rewritten(self) -> None:
        self.assert_done(self.run_round(DEFECT), 0)
        # O `notes.txt` nunca recebeu defeito: o manifesto diz que ele está certo.
        (self.fixture.tree / "notes.txt").write_text("changed\n", encoding="utf-8")
        (self.fixture.tree / "stray.txt").write_text("left behind\n", encoding="utf-8")
        self.assert_done(self.run_round(DEFECT, LEGITIMATE), 0)
        self.assertEqual((self.fixture.tree / "notes.txt").read_text(encoding="utf-8"),
                         FILES["notes.txt"])
        self.assertFalse((self.fixture.tree / "stray.txt").exists())

    def test_an_interrupted_round_is_repaired_by_the_next_one(self) -> None:
        self.assert_done(self.run_round(DEFECT), 0)
        kept = (self.fixture.tree / "build.py").stat().st_mtime_ns
        held, waiting, release, released = self.held()
        self.fixture.catalogue("fix", HEAD + entry(
            "fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
            extra=f"roda = {json.dumps(held)}"))
        # A rodada para com o defeito dentro da cópia, como num Ctrl-C sem volta.
        running = subprocess.Popen(self.fixture.command("--rodar", "fix"), env=ENVIRONMENT,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            self.wait_for(waiting.exists)
        finally:
            running.send_signal(signal.SIGKILL)
            running.wait()
            release.touch()
        self.wait_for(released.exists)
        self.assertIn("limit = 11", (self.fixture.tree / "limit.conf").read_text(encoding="utf-8"))
        self.assert_done(self.run_round(DEFECT, LEGITIMATE), 0, "✓ fix-limit: reprovado",
                         "✓ fix-notes: passou")
        self.assertEqual((self.fixture.tree / "limit.conf").read_text(encoding="utf-8"),
                         FILES["limit.conf"])
        # Só o arquivo do defeito foi gravado de novo: o resto do build serve.
        self.assertEqual((self.fixture.tree / "build.py").stat().st_mtime_ns, kept)

    def test_a_working_tree_that_changes_during_the_round_fails_it(self) -> None:
        held, waiting, release, _ = self.held()
        self.fixture.catalogue("fix", HEAD + entry(
            "fix-limit", "limit = 10", "limit = 11", message="check: the limit is 11",
            extra=f"roda = {json.dumps(held)}"))
        running = subprocess.Popen(self.fixture.command("--rodar", "fix"), env=ENVIRONMENT,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        try:
            self.wait_for(waiting.exists)
            self.fixture.write("notes.txt", "changed while the round ran\n")
        finally:
            release.touch()
        shown, _ = running.communicate(timeout=60)
        self.assertEqual(running.returncode, 1, shown)
        self.assertIn("✓ fix-limit: reprovado", shown)
        self.assertIn("a árvore de trabalho mudou durante a rodada", shown)


def flatten(suite: unittest.TestSuite):
    for test in suite:
        if isinstance(test, unittest.TestSuite):
            yield from flatten(test)
        else:
            yield test


def main() -> int:
    """Os casos em paralelo: cada um é um repositório e alguns processos, e não divide nada."""
    tests = list(flatten(unittest.defaultTestLoader.loadTestsFromModule(sys.modules[__name__])))

    def run_one(test: unittest.TestCase) -> unittest.TestResult:
        result = unittest.TestResult()
        test.run(result)
        return result

    started = time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(len(os.sched_getaffinity(0))) as pool:
        results = list(pool.map(run_one, tests))
    failed = 0
    for test, result in zip(tests, results):
        for _, trace in result.failures + result.errors:
            failed += 1
            print(f"FALHOU {test.id()}\n{trace}", file=sys.stderr)
    ran = sum(result.testsRun for result in results)
    if failed or ran != len(tests):
        print(f"ferramenta de mutação FALHOU: {failed} de {len(tests)} casos.", file=sys.stderr)
        return 1
    print(f"ferramenta de mutação: {ran} casos em {time.monotonic() - started:.1f} s.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
