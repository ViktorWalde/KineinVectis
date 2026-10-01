"""Onde o `cmake --build --preset NOME` escreve, e qual e' o binario da UI.

Dono unico desta leitura (saiu do verificar_binario_abre.py em 2026-10-01,
quando o check_terminal_quiet.py tambem precisou dela): dois leitores de
preset divergiriam no primeiro `inherits` novo.
"""

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PRESET_FILES = [REPO_ROOT / "CMakePresets.json", REPO_ROOT / "CMakeUserPresets.json"]
UI_BINARY = Path("ui") / "kinein-vectis"


def _presets() -> tuple[dict[str, dict], dict[str, dict]]:
    """Presets de configure e de build, dos dois arquivos (o do usuario e'
    opcional e vence pelo nome, como no proprio cmake)."""
    configure: dict[str, dict] = {}
    build: dict[str, dict] = {}
    for preset_file in PRESET_FILES:
        if not preset_file.is_file():
            continue
        data = json.loads(preset_file.read_text(encoding="utf-8"))
        for preset in data.get("configurePresets", []):
            configure[preset["name"]] = preset
        for preset in data.get("buildPresets", []):
            build[preset["name"]] = preset
    return configure, build


def preset_binary_dir(name: str) -> Path:
    """`cmake --build --preset NOME` escreve em qual pasta? O build preset
    aponta o configure preset; o `binaryDir` pode vir por `inherits`."""
    configure, build = _presets()
    if name in build:
        name = build[name]["configurePreset"]
    seen: set[str] = set()
    current = name
    while current in configure and current not in seen:
        seen.add(current)
        preset = configure[current]
        if "binaryDir" in preset:
            return Path(preset["binaryDir"].replace("${sourceDir}", str(REPO_ROOT)))
        parents = preset.get("inherits", [])
        current = parents[0] if isinstance(parents, list) and parents else str(parents)
    raise SystemExit(f"erro: preset '{name}' sem binaryDir nos CMake*Presets.json")
