"""Mede um gate do clang-tidy contra o corpus dele.

Metodo do docs/engenharia/gates-e-medida-de-eficacia.md: N defeitos plantados
que o gate tem de reprovar e M formas corretas que ele tem de aceitar.

- No arquivo de defeitos, cada linha marcada com "// reprova:" tem de receber
  o diagnostico da checagem, e nenhuma outra linha pode receber.
- No arquivo de formas corretas, nenhuma linha pode receber. As marcadas com
  "// aceita:" sao contadas como formas corretas.
- Qualquer outro diagnostico (o corpus nao compila, por exemplo) reprova: um
  corpus que nao compila nao mede nada.

Roda so a checagem do gate (--checks=-*,<checagem>), com as opcoes do
.clang-tidy do core, para medir o gate e nada mais.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

DIAGNOSTIC = re.compile(
    r"^(?P<path>.+?):(?P<line>\d+):\d+: (?:warning|error): .*\[(?P<tags>[^\]]+)\]$"
)
REJECT_MARK = "// reprova:"
ACCEPT_MARK = "// aceita:"


def marked_lines(source: Path, mark: str) -> set[int]:
    lines = source.read_text(encoding="utf-8").splitlines()
    return {number for number, text in enumerate(lines, start=1) if mark in text}


def diagnostics(args: argparse.Namespace, source: Path) -> dict[int, set[str]]:
    command = [
        args.clang_tidy,
        f"--config-file={args.config}",
        f"--checks=-*,{args.check}",
        str(source),
        "--",
        f"-std=c++{args.std}",
    ]
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    found: dict[int, set[str]] = {}
    for line in result.stdout.splitlines():
        match = DIAGNOSTIC.match(line)
        if match and Path(match["path"]).resolve() == source.resolve():
            tags = {tag for tag in match["tags"].split(",") if not tag.startswith("-")}
            found.setdefault(int(match["line"]), set()).update(tags)
    return found


def foreign(found: dict[int, set[str]], check: str) -> list[str]:
    return [
        f"linha {line}: {', '.join(sorted(tags - {check}))}"
        for line, tags in sorted(found.items())
        if tags - {check}
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--clang-tidy", required=True)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--check", required=True)
    parser.add_argument("--std", required=True)
    parser.add_argument("--reject", required=True, type=Path)
    parser.add_argument("--accept", required=True, type=Path)
    args = parser.parse_args()

    problems: list[str] = []

    planted = marked_lines(args.reject, REJECT_MARK)
    rejected = diagnostics(args, args.reject)
    problems += [f"{args.reject.name}: diagnostico estranho na {item}" for item in foreign(rejected, args.check)]
    flagged = {line for line, tags in rejected.items() if args.check in tags}
    problems += [f"{args.reject.name}:{line}: defeito nao reprovado" for line in sorted(planted - flagged)]
    problems += [f"{args.reject.name}:{line}: acusada sem marca de defeito" for line in sorted(flagged - planted)]

    legitimate = marked_lines(args.accept, ACCEPT_MARK)
    accepted = diagnostics(args, args.accept)
    problems += [f"{args.accept.name}: diagnostico estranho na {item}" for item in foreign(accepted, args.check)]
    problems += [f"{args.accept.name}:{line}: forma correta reprovada" for line in sorted(accepted)]

    rejected_count = len(planted & flagged)
    accepted_count = len(legitimate - set(accepted))
    print(f"gate {args.check}: {rejected_count} de {len(planted)} defeitos reprovados; "
          f"{accepted_count} de {len(legitimate)} formas corretas aceitas")
    for problem in problems:
        print(problem)
    if not planted or not legitimate:
        print("corpus vazio: sem defeito ou sem forma correta, a medida nao vale")
        return 1
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
