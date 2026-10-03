#!/usr/bin/env python3
"""gerar_degrade_boas_vindas.py [--check] — o degrade da tela de boas-vindas.

Pedido do autor (2026-10-03): um degrade "suave, nao o sRGB" da paleta do
icone. O Qt so' interpola em sRGB entre paradas vizinhas, e o meio de um
degrade sRGB entre a tinta e o ambar escuro sai "sujo"; misturar em OKLab
(Ottosson, 2020), que e' perceptual, deixa o passo de cor uniforme.

As pontas sao tokens do Theme (`brandInk` em cima, `brandEmber` embaixo). Este
script mistura em OKLab e grava SETE paradas prontas no proprio Theme.qml,
entre os marcadores `// BEGIN welcomeGradient (gerado)` e `// END ...`. A
conta roda aqui, uma vez, e nao no programa — o QML so' le cores.

--check: so' confere (o gate `verificar-qml-tokens.sh` o chama); sai 1 se o
bloco do Theme nao bate com as pontas de agora.
"""
import re
import sys
from pathlib import Path

THEME = Path(__file__).resolve().parent.parent / "ui" / "qml" / "Theme.qml"
BEGIN = "    // BEGIN welcomeGradient (gerado: scripts/gerar_degrade_boas_vindas.py)"
END = "    // END welcomeGradient"
STOPS = [0.0, 0.16, 0.33, 0.5, 0.67, 0.84, 1.0]


def token(text: str, name: str) -> str:
    m = re.search(r'readonly property color %s: "(#[0-9a-fA-F]{6})"' % name, text)
    if not m:
        sys.exit(f"erro: Theme.qml sem o token {name}")
    return m.group(1)


def to_linear(c: float) -> float:
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def to_gamma(v: float) -> int:
    v = 12.92 * v if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055
    return round(max(0.0, min(1.0, v)) * 255)


def to_oklab(hex_color: str) -> tuple[float, float, float]:
    r, g, b = (to_linear(int(hex_color[i:i + 2], 16)) for i in (1, 3, 5))
    l = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b) ** (1 / 3)
    m = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b) ** (1 / 3)
    s = (0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b) ** (1 / 3)
    return (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s)


def from_oklab(lab: tuple[float, float, float]) -> str:
    big_l, a, b = lab
    l = (big_l + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (big_l - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (big_l - 0.0894841775 * a - 1.2914855480 * b) ** 3
    rgb = (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
           -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
           -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)
    return "#" + "".join(f"{to_gamma(v):02x}" for v in rgb)


def block(top: str, bottom: str) -> str:
    a, b = to_oklab(top), to_oklab(bottom)
    colors = [from_oklab(tuple(a[i] + (b[i] - a[i]) * t for i in range(3))) for t in STOPS]
    body = ", ".join(f'"{c}"' for c in colors)
    return (f"{BEGIN}\n    // De `brandInk` a `brandEmber` em OKLab, nas posicoes {STOPS}.\n"
            f"    readonly property var welcomeGradient: [{body}]\n{END}")


def main() -> int:
    text = THEME.read_text()
    wanted = block(token(text, "brandInk"), token(text, "brandEmber"))
    pattern = re.compile(re.escape(BEGIN) + r".*?" + re.escape(END), re.S)
    current = pattern.search(text)
    if "--check" in sys.argv:
        if current is None or current.group(0) != wanted:
            print("erro: o degrade da tela de boas-vindas no Theme.qml nao bate com as pontas; "
                  "rode python3 scripts/gerar_degrade_boas_vindas.py")
            return 1
        print("degrade de boas-vindas: confere com brandInk/brandEmber (OKLab, 7 paradas).")
        return 0
    if current is None:
        sys.exit("erro: Theme.qml sem os marcadores do bloco welcomeGradient")
    THEME.write_text(text[:current.start()] + wanted + text[current.end():])
    print("Theme.qml: degrade de boas-vindas regravado.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
