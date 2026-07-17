#!/usr/bin/env python3
"""Resolve o binaryDir REAL de cada configure preset USAVEL.

POR QUE ISTO EXISTE, e nao e' conveniencia. Um preset sem `binaryDir` proprio
herda o do pai — e essa heranca e' invisivel a olho nu no JSON. Foi assim que
`dev-local-release` passou a gravar por cima de `build/linux-clang-release-hardened`,
que e' o diretorio do binario executado por `scripts/kinein-vectis`: o atalho de
Desenvolvimento passou a rodar uma build que ninguem pediu, sem um aviso sequer.

Dois consumidores precisam da MESMA resposta, e se divergirem a mina volta:
  - scripts/verificar-presets.sh  -> dois presets no mesmo diretorio = colisao;
  - scripts/atualizar-tudo.sh     -> diretorio que ninguem reivindica = orfao.

Emite apenas presets USAVEIS (sem `hidden: true`), um por linha:

    <nome>\t<binaryDir absoluto>

Preset oculto existe so para heranca e nunca e' configurado direto, entao o
binaryDir dele so vira diretorio de verdade atraves de um filho: incluí-lo
inventaria colisao e faria o gate gritar falso — que e' pior que gate nenhum,
porque ensina a ignorar.
"""

import json
import sys
from pathlib import Path

# Macros que sabemos expandir com certeza. Qualquer outra sobrevive na string e
# e' denunciada: comparar caminho com macro por expandir compararia texto, nao
# diretorio, e um gate que compara a coisa errada mente com confianca.
MACROS_CONHECIDAS = ("${sourceDir}", "${presetName}")


def carregar(raiz: Path) -> dict:
    presets: dict[str, dict] = {}
    for arquivo in ("CMakePresets.json", "CMakeUserPresets.json"):
        caminho = raiz / arquivo
        if not caminho.is_file():
            continue
        with caminho.open(encoding="utf-8") as origem:
            dados = json.load(origem)
        for preset in dados.get("configurePresets", []):
            presets[preset["name"]] = preset
    return presets


def binary_dir_cru(presets: dict, nome: str, vistos: frozenset = frozenset()) -> str | None:
    """binaryDir declarado no preset ou herdado do primeiro pai que o declara."""
    if nome in vistos:            # `inherits` circular: o CMake rejeita, nos nao travamos
        return None
    preset = presets.get(nome, {})
    if "binaryDir" in preset:
        return preset["binaryDir"]
    herdados = preset.get("inherits") or []
    if isinstance(herdados, str):
        herdados = [herdados]
    for pai in herdados:
        achado = binary_dir_cru(presets, pai, vistos | {nome})
        if achado:
            return achado
    return None


def resolver(caminho: str, raiz: Path, nome: str) -> str:
    expandido = caminho.replace("${sourceDir}", str(raiz)).replace("${presetName}", nome)
    return str(Path(expandido))


def main() -> int:
    raiz = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    presets = carregar(raiz)

    saida: list[str] = []
    for nome, preset in presets.items():
        if preset.get("hidden"):
            continue
        cru = binary_dir_cru(presets, nome)
        if not cru:
            continue
        resolvido = resolver(cru, raiz, nome)
        if "${" in resolvido or "$env{" in resolvido:
            print(
                f"erro: preset '{nome}' usa macro que este resolvedor nao expande: {cru}\n"
                f"       Ensine a macro a {__file__} — nao compare caminho por expandir.",
                file=sys.stderr,
            )
            return 2
        saida.append(f"{nome}\t{resolvido}")

    print("\n".join(saida))
    return 0


if __name__ == "__main__":
    sys.exit(main())
