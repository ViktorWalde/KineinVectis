"""Measure a clang-tidy gate against its corpus.

Documented in: docs/engenharia/gates-e-medida-de-eficacia.md

N planted defects the gate must reject and M correct forms it must accept:

- In the defects file, each line marked with "// reject:" must receive the
  check's diagnostic, and no other line may receive it.
- In the correct-forms file, no line may receive it. The lines marked with
  "// accept:" are counted as correct forms.
- Any other diagnostic (the corpus does not compile, for example) fails: a
  corpus that does not compile measures nothing.

Runs only the gate's check (--checks=-*,<check>), with the options of the
core's .clang-tidy, to measure the gate and nothing else.
"""

import argparse
import re
import subprocess
import sys
from pathlib import Path

DIAGNOSTIC = re.compile(
    r"^(?P<path>.+?):(?P<line>\d+):\d+: (?:warning|error): .*\[(?P<tags>[^\]]+)\]$"
)
REJECT_MARK = "// reject:"
ACCEPT_MARK = "// accept:"


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
        f"line {line}: {', '.join(sorted(tags - {check}))}"
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
    problems += [f"{args.reject.name}: unexpected diagnostic on {item}" for item in foreign(rejected, args.check)]
    flagged = {line for line, tags in rejected.items() if args.check in tags}
    problems += [f"{args.reject.name}:{line}: defect not rejected" for line in sorted(planted - flagged)]
    problems += [f"{args.reject.name}:{line}: flagged without a defect mark" for line in sorted(flagged - planted)]

    legitimate = marked_lines(args.accept, ACCEPT_MARK)
    accepted = diagnostics(args, args.accept)
    problems += [f"{args.accept.name}: unexpected diagnostic on {item}" for item in foreign(accepted, args.check)]
    problems += [f"{args.accept.name}:{line}: correct form rejected" for line in sorted(accepted)]

    rejected_count = len(planted & flagged)
    accepted_count = len(legitimate - set(accepted))
    print(f"gate {args.check}: {rejected_count} of {len(planted)} defects rejected; "
          f"{accepted_count} of {len(legitimate)} correct forms accepted")
    for problem in problems:
        print(problem)
    if not planted or not legitimate:
        print("empty corpus: no defect or no correct form, the measurement is void")
        return 1
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
