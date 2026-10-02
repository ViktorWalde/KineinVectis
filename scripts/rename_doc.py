#!/usr/bin/env python3
"""Renomeia um documento do DocsPublic e reescreve toda referencia a ele.

POR QUE EXISTE (2026-10-01). `git mv` nao atualiza referencia nenhuma, e este
repositorio ja' pagou por isso: em 2026-07-16, 410 referencias reescritas a
mao; num checkout de 2026-10-01, 12 citacoes mortas de renomeacoes antigas,
inclusive em Rust e C++. Um documento e' citado de tres jeitos, e os tres sao
reescritos aqui:

  1. link Markdown relativo    [texto](../arquitetura/03-protocolo-ipc.md#x)
     recalculado a partir da pasta de CADA arquivo que o contem;
  2. citacao de caminho        DocsPublic/arquitetura/03-protocolo-ipc.md
     em qualquer arquivo rastreado (codigo, scripts, comentarios);
  3. o nome do arquivo         `03-protocolo-ipc.md`, `03-protocolo-ipc`
     (o nome sem extensao so' entre crases, onde e' citacao e nao prosa).

Quem confere o resultado e' o scripts/verificar-links-docs.sh: rode-o depois.

Uso:
    python3 scripts/rename_doc.py ANTIGO NOVO [ANTIGO NOVO ...]
    python3 scripts/rename_doc.py --dry-run ANTIGO NOVO
Os caminhos sao relativos a raiz do repositorio (DocsPublic/...).
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"(\]\()([^)\s#]+\.md)((?:#[^)\s]*)?(?:\s+\"[^\"]*\")?\))")
SKIPPED_PREFIXES = ("KV0.3/", "build/", "target/")


def tracked_text_files() -> list[Path]:
    names = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.rstrip("\0").split("\0")
    files = []
    for name in names:
        if not name or name.startswith(SKIPPED_PREFIXES):
            continue
        path = ROOT / name
        if path.is_file():
            files.append(path)
    return files


def rewrite_links(source: Path, text: str, old: Path, new: Path) -> str:
    def replace(match: re.Match[str]) -> str:
        target = match.group(2)
        if target.startswith(("http://", "https://", "mailto:")):
            return match.group(0)
        resolved = (source.parent / target).resolve()
        if resolved != old.resolve():
            return match.group(0)
        relative = os.path.relpath(new.resolve(), source.parent.resolve())
        return f"{match.group(1)}{relative}{match.group(3)}"

    return LINK.sub(replace, text)


def rename(pairs: list[tuple[Path, Path]], dry_run: bool) -> int:
    for old, new in pairs:
        if not old.is_file():
            print(f"erro: {old.relative_to(ROOT)} nao existe", file=sys.stderr)
            return 1
        if new.exists():
            print(f"erro: {new.relative_to(ROOT)} ja' existe", file=sys.stderr)
            return 1
    files = tracked_text_files()
    # Nome repetido no repositorio: o nome solto e' ambiguo, so' caminho e link
    # sao reescritos para ele.
    all_names = [path.name for path in files]
    unique = {old: all_names.count(old.name) == 1 for old, _ in pairs}
    changed: dict[Path, int] = {}
    for path in files:
        try:
            original = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        text = original
        for old, new in pairs:
            old_rel, new_rel = str(old.relative_to(ROOT)), str(new.relative_to(ROOT))
            if path.suffix == ".md":
                text = rewrite_links(path, text, old, new)
            text = text.replace(old_rel, new_rel)
            # A forma pasta/nome (texto de link, citacao curta) — a regra do nome
            # solto abaixo nao a pega, por causa da barra antes do nome.
            folder = old.parent.name
            text = text.replace(f"{folder}/{old.name}", f"{new.parent.name}/{new.name}")
            if unique[old]:
                text = re.sub(rf"(?<![\w./-]){re.escape(old.name)}", new.name, text)
                text = text.replace(f"`{old.stem}`", f"`{new.stem}`")
        if text != original:
            changed[path] = sum(1 for a, b in zip(original.splitlines(), text.splitlines()) if a != b)
            if not dry_run:
                path.write_text(text, encoding="utf-8")
    for old, new in pairs:
        print(f"{old.relative_to(ROOT)} -> {new.relative_to(ROOT)}")
        if not dry_run:
            subprocess.run(["git", "mv", str(old), str(new)], cwd=ROOT, check=True)
    for path, count in sorted(changed.items()):
        print(f"  {count:3d} linha(s)  {path.relative_to(ROOT)}")
    print(f"{len(changed)} arquivo(s) com referencia reescrita"
          + (" (dry-run: nada gravado)" if dry_run else ""))
    return 0


def main() -> int:
    args = sys.argv[1:]
    dry_run = "--dry-run" in args
    args = [a for a in args if a != "--dry-run"]
    if not args or len(args) % 2:
        print(__doc__, file=sys.stderr)
        return 2
    pairs = [(ROOT / args[i], ROOT / args[i + 1]) for i in range(0, len(args), 2)]
    return rename(pairs, dry_run)


if __name__ == "__main__":
    sys.exit(main())
