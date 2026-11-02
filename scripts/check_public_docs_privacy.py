#!/usr/bin/env python3
"""Verifica a separação entre documentação técnica pública e notas locais."""

from pathlib import Path
import re
import subprocess
import sys


LOCAL_GUIDES = frozenset({
    "AGENTS.md", "AGENTS.override.md", "CLAUDE.md", "05-com-um-agente-de-ia.md",
})
LOCAL_DIRECTORIES = frozenset({"DocsPrivate", ".claude", ".codex", ".agents"})
PRIVATE_REFERENCE = re.compile(
    r"DocsPrivate|AGENTS(?:\.override)?\.md|CLAUDE\.md|05-com-um-agente-de-ia\.md",
    re.I,
)
HOME_ACCOUNT = re.compile(
    r"(?:/(?:home|Users)/|(?i:[A-Z]:[\\/]+Users[\\/]+))([^/\\\s`\"'<>),:;]+)",
)
EXAMPLE_ACCOUNTS = frozenset({
    "user", "usuario", "username", "developer", "dev", "example", "pi",
    "kinein", "runner", "root", "...",
})
SESSION_MARKER = re.compile(
    r"\bHANDOFF\b|prompt de (?:entrada|retomada)|objetivo (?:contínuo )?autorizado"
    r"|retomada local|\*\*Checkout:\*\*",
    # Sessões de terminal/depuração são técnicas; permissões de agentes são locais.
    re.I,
)
SESSION_REQUEST = re.compile(
    r"\b(?:autor|mantenedor) (?:pediu|disse|instalou|autorizou|fará|fara)\b"
    r"|\b(?:autor|mantenedor)\b.{0,50}\b(?:pediu|disse|autorizou|mandou remover)\b"
    r"|\bmodo aut[oô]nomo\b|prompt (?:de trabalho|para IA CLI)"
    r"|\b(?:minha|meu) (?:máquina|notebook|placa)\b",
    re.I,
)
AGENT_SESSION = re.compile(
    r"orçamento de tokens|renov\w*.{0,40}\bcota\b|modelo/esforço"
    r"|agentes adicionais|sem novas delegações|continuidade (?:automática|imediata)"
    r"|encerrar a sessão após commit|limite semanal|sessão do agente, que rodava"
    r"|fornecer prompt para outra sessão|\bagente (?:passo \d+|anterior)\b",
    re.I,
)
DEVICE_ADDRESS = re.compile(r"\b(?:[0-9a-f]{2}:){5}[0-9a-f]{2}\b", re.I)
EXAMPLE_DEVICE_ADDRESS = "02:00:00:00:00:01"


def inspect_text(name: str, content: str) -> list[str]:
    """Retorna diagnósticos sem reproduzir o conteúdo potencialmente privado."""
    issues = []
    for number, line in enumerate(content.splitlines(), 1):
        location = f"{name}:{number}"
        if PRIVATE_REFERENCE.search(line):
            issues.append(f"{location}: referência a orientação ou nota local")
        if any(match.group(1).lower() not in EXAMPLE_ACCOUNTS
               for match in HOME_ACCOUNT.finditer(line)):
            issues.append(f"{location}: caminho de conta pessoal; use exemplo neutro")
        if any(pattern.search(line) for pattern in (SESSION_MARKER, SESSION_REQUEST, AGENT_SESSION)):
            issues.append(f"{location}: metadado de sessão em documentação pública")
        if any(match.group().lower() != EXAMPLE_DEVICE_ADDRESS
               for match in DEVICE_ADDRESS.finditer(line)):
            issues.append(f"{location}: identificador de dispositivo; use exemplo neutro")
    return issues


def inspect_files(root: Path, names: list[str]) -> list[str]:
    """Examina a árvore publicável, incluindo novos Markdown não ignorados."""
    issues = []
    for name in names:
        path = Path(name)
        if path.name in LOCAL_GUIDES or any(part in LOCAL_DIRECTORIES for part in path.parts):
            issues.append(f"{name}: orientação local não pode integrar a árvore publicável")
            continue
        if path.suffix.lower() != ".md" or not (root / path).is_file():
            continue
        issues.extend(inspect_text(name, (root / path).read_text(encoding="utf-8")))
    return issues


def main() -> int:
    """Reprova vazamentos na seleção de arquivos do Git sem alterar o índice."""
    root = Path(__file__).resolve().parent.parent
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root, capture_output=True, check=True,
    )
    names = sorted(set(result.stdout.decode().rstrip("\0").split("\0")))
    issues = inspect_files(root, names)
    for issue in issues:
        print(f"✗ {issue}", file=sys.stderr)
    if issues:
        print(f"privacidade documental: {len(issues)} ocorrência(s)", file=sys.stderr)
        return 1
    print("privacidade documental: somente documentação pública, sem contas pessoais ou notas locais")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
