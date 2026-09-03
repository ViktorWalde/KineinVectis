#!/usr/bin/env python3
"""Verifica que todo binding QML aponta para propriedade/sinal que EXISTE.

Ver scripts/verificar-qml-propriedades.sh para o porque (falha silenciosa
medida por mutacao em 2026-09-03).
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
QML = RAIZ / "ui" / "qml"

# Propriedades que todo Item/QtObject do QtQuick ja' tem. Nao sao declaradas
# nos arquivos deste repositorio, e por isso precisam de allowlist.
BASE = {
    "id", "objectName", "x", "y", "z", "width", "height", "implicitWidth",
    "implicitHeight", "visible", "opacity", "enabled", "clip", "parent",
    "rotation", "scale", "transformOrigin", "smooth", "antialiasing",
    "focus", "activeFocusOnTab", "state", "states", "transitions",
    "transform", "children", "data", "resources", "baselineOffset",
    "containmentMask", "layer", "anchors", "childrenRect",
    # sinais/handlers de base usados com frequencia
    "onVisibleChanged", "onWidthChanged", "onHeightChanged", "onXChanged",
    "onYChanged", "onEnabledChanged", "onOpacityChanged", "onFocusChanged",
    "onActiveFocusChanged", "onParentChanged", "onStateChanged",
    "onImplicitWidthChanged", "onImplicitHeightChanged",
}

# Propriedades dos tipos-raiz do QtQuick usados neste repositorio. Um componente
# daqui herda a interface do seu tipo raiz: `KvIconButton` e' um `Rectangle`, e
# por isso aceita `radius`. Sem isto o gate acusa heranca legitima.
RAIZ_QTQUICK = {
    "Item": set(),
    "QtObject": set(),
    "Rectangle": {"color", "radius", "border", "gradient",
                  "topLeftRadius", "topRightRadius",
                  "bottomLeftRadius", "bottomRightRadius"},
    "MouseArea": {"acceptedButtons", "containsMouse", "cursorShape", "drag",
                  "hoverEnabled", "pressed", "propagateComposedEvents",
                  "preventStealing", "scrollGestureEnabled", "pressAndHoldInterval",
                  "onClicked", "onPressed", "onReleased", "onEntered", "onExited",
                  "onPositionChanged", "onWheel", "onDoubleClicked",
                  "onPressAndHold", "onCanceled", "onContainsMouseChanged"},
    "Row": {"spacing", "layoutDirection", "effectiveLayoutDirection",
            "add", "move", "populate", "padding", "topPadding",
            "leftPadding", "rightPadding", "bottomPadding"},
    "Column": {"spacing", "add", "move", "populate", "padding", "topPadding",
               "leftPadding", "rightPadding", "bottomPadding"},
    "ListView": {"model", "delegate", "currentIndex", "currentItem", "count",
                 "orientation", "spacing", "header", "footer", "section",
                 "highlight", "highlightItem", "highlightFollowsCurrentItem",
                 "highlightMoveDuration", "highlightMoveVelocity",
                 "highlightResizeDuration", "highlightRangeMode",
                 "preferredHighlightBegin", "preferredHighlightEnd",
                 "snapMode", "cacheBuffer", "displayMarginBeginning",
                 "displayMarginEnd", "boundsBehavior", "boundsMovement",
                 "flickDeceleration", "maximumFlickVelocity", "interactive",
                 "contentX", "contentY", "contentWidth", "contentHeight",
                 "contentItem", "originX", "originY", "moving", "flicking",
                 "dragging", "atYBeginning", "atYEnd", "atXBeginning", "atXEnd",
                 "verticalLayoutDirection", "layoutDirection", "keyNavigationEnabled",
                 "keyNavigationWraps", "reuseItems", "pixelAligned", "synchronousDrag",
                 "onCurrentIndexChanged", "onCountChanged", "onModelChanged",
                 "onContentYChanged", "onContentXChanged", "onMovementEnded",
                 "onFlickEnded", "onAtYEndChanged", "onDraggingChanged"},
    "Window": {"color", "title", "flags", "modality", "visibility", "screen",
               "minimumWidth", "minimumHeight", "maximumWidth", "maximumHeight",
               "onClosing", "onVisibilityChanged", "onScreenChanged"},
}

# Prefixos agrupados/anexados: `anchors.fill`, `Layout.row`, `Keys.onPressed`.
PREFIXO_OK = re.compile(r"^[A-Za-z_]\w*\.")

RE_PROP = re.compile(
    r"^\s*(?:readonly\s+|required\s+|default\s+)*property\s+"
    r"(?:alias\s+)?[\w<>.]+\s+(\w+)", re.M)
RE_ALIAS = re.compile(r"^\s*(?:readonly\s+)?property\s+alias\s+(\w+)\s*:", re.M)
RE_SIGNAL = re.compile(r"^\s*signal\s+(\w+)", re.M)
RE_FUNC = re.compile(r"^\s*function\s+(\w+)", re.M)
RE_TIPO_RAIZ = re.compile(r"^([A-Z]\w*)\s*\{", re.M)


def interface(texto: str) -> set[str]:
    """Nomes que um componente aceita: propriedades, aliases e handlers."""
    nomes: set[str] = set()
    props = set(RE_PROP.findall(texto)) | set(RE_ALIAS.findall(texto))
    nomes |= props
    for p in props:
        nomes.add(f"on{p[0].upper()}{p[1:]}Changed")
    for s in RE_SIGNAL.findall(texto):
        nomes.add(f"on{s[0].upper()}{s[1:]}")
    nomes |= set(RE_FUNC.findall(texto))
    return nomes


def tipo_raiz(texto: str) -> str | None:
    """O tipo do objeto RAIZ do arquivo — de quem o componente herda."""
    m = RE_TIPO_RAIZ.search(texto)
    return m.group(1) if m else None


def componentes() -> dict[str, Path]:
    return {p.stem: p for p in QML.rglob("*.qml") if p.stem[:1].isupper()}


def indent(linha: str) -> int:
    return len(linha) - len(linha.lstrip())


def checar(caminho: Path, tipos: dict[str, set[str]]) -> list[str]:
    linhas = caminho.read_text(encoding="utf-8").split("\n")
    erros: list[str] = []
    # pilha de blocos abertos: (indentacao_do_cabecalho, tipo)
    pilha: list[tuple[int, str | None]] = []

    abre = re.compile(r"^(\s*)(?:\w+\s*:\s*)?([A-Z]\w*)\s*\{\s*$")
    atrib = re.compile(r"^(\s*)([A-Za-z_][\w.]*)\s*:")

    for n, linha in enumerate(linhas, 1):
        crua = linha.rstrip()
        if not crua.strip() or crua.strip().startswith("//"):
            continue

        m = abre.match(crua)
        if m:
            pilha.append((len(m.group(1)), m.group(2)))
            continue

        if crua.strip() == "}" and pilha:
            ind = indent(crua)
            while pilha and pilha[-1][0] >= ind:
                pilha.pop()
            continue

        a = atrib.match(crua)
        if a and pilha:
            ind_cab, tipo = pilha[-1]
            # so' o nivel imediatamente dentro do bloco
            if indent(crua) != ind_cab + 4:
                continue
            if tipo not in tipos:
                continue
            nome = a.group(2)
            if PREFIXO_OK.match(nome) or nome in BASE:
                continue
            if nome in tipos[tipo]:
                continue
            erros.append(
                f"{caminho.relative_to(RAIZ)}:{n}: `{nome}:` nao existe em "
                f"{tipo} (nem propriedade, nem alias, nem on<Sinal>)")
    return erros


def main() -> int:
    mapa = componentes()
    textos = {n: p.read_text(encoding="utf-8") for n, p in mapa.items()}
    proprias = {n: interface(t) for n, t in textos.items()}
    raizes = {n: tipo_raiz(t) for n, t in textos.items()}

    def resolver(nome: str, vistos: frozenset[str] = frozenset()) -> set[str]:
        """Interface do componente MAIS a que ele herda do tipo raiz."""
        if nome in vistos:
            return set()
        herdado: set[str] = set()
        raiz = raizes.get(nome)
        if raiz in RAIZ_QTQUICK:
            herdado |= RAIZ_QTQUICK[raiz]
        elif raiz in proprias:
            herdado |= resolver(raiz, vistos | {nome})
        return proprias[nome] | herdado

    tipos = {n: resolver(n) for n in mapa}

    erros: list[str] = []
    for p in sorted(QML.rglob("*.qml")):
        erros += checar(p, tipos)

    if erros:
        print("propriedades QML: binding para nome INEXISTENTE:", file=sys.stderr)
        for e in erros:
            print(f"  {e}", file=sys.stderr)
        print(
            "\nO QML so' reclamaria disso ao INSTANCIAR o componente — e os "
            "hosts do shell nao sao instanciados por nenhum teste.",
            file=sys.stderr)
        return 1

    print(f"propriedades QML: {len(mapa)} componentes, nenhum binding orfao.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
