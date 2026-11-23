"""Detector de codigo duplicado sobre a arvore sintatica da libclang (P20).

Candidato ao gate de duplicacao, medido pelo metodo do
docs/engenharia/gates-e-medida-de-eficacia.md contra o corpus desta pasta.

O mecanismo:
- Cada definicao de funcao do projeto vira uma sequencia de unidades, uma por
  no da arvore, em pre-ordem: o tipo do no e o que ele carrega (operador,
  literal, declaracao referida). Comentario, formatacao e o nome da propria
  funcao nao entram.
- Variavel local e parametro perdem o nome: so importa onde a mesma variavel
  aparece de novo. Declaracao de fora da funcao (funcao chamada, tipo,
  membro, enumerador) entra pelo USR, entao chamar outra coisa muda a
  sequencia.
- Detector de janela: K unidades seguidas iguais em dois lugares. A variavel
  local vira a distancia ate o uso anterior dela (Baker), com a distancia que
  sai da janela zerada; assim a janela casa em qualquer posicao. Pega a copia
  (T1), a copia com nomes trocados (T2) e o trecho copiado dentro de funcao
  maior.
- Detector de funcao inteira: semelhanca do difflib entre duas funcoes de
  pelo menos MIN unidades. Pega a copia com comandos a mais, a menos ou
  trocados (T3) e a copia com literais ou tipos trocados.
- Funcao gerada por macro fica de fora: a fonte dela e uma so.

Mesma logica escrita de outro jeito (T4) nenhuma ferramenta decide (teorema de
Rice); este detector nao tenta.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import difflib
import shutil
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

import clang.cindex as ci

K = ci.CursorKind
FUNCTION_KINDS = {
    K.FUNCTION_DECL,
    K.CXX_METHOD,
    K.CONSTRUCTOR,
    K.DESTRUCTOR,
    K.FUNCTION_TEMPLATE,
    K.CONVERSION_FUNCTION,
}
SCOPE_KINDS = {
    K.NAMESPACE,
    K.CLASS_DECL,
    K.STRUCT_DECL,
    K.CLASS_TEMPLATE,
    K.CLASS_TEMPLATE_PARTIAL_SPECIALIZATION,
    K.UNEXPOSED_DECL,
}
LITERAL_KINDS = {
    K.INTEGER_LITERAL,
    K.FLOATING_LITERAL,
    K.IMAGINARY_LITERAL,
    K.CHARACTER_LITERAL,
    K.STRING_LITERAL,
    K.CXX_BOOL_LITERAL_EXPR,
    K.CXX_NULL_PTR_LITERAL_EXPR,
}
OPERATOR_KINDS = {K.BINARY_OPERATOR, K.COMPOUND_ASSIGNMENT_OPERATOR}
NAMED_EXPRESSION_KINDS = {K.DECL_REF_EXPR, K.MEMBER_REF_EXPR, K.CALL_EXPR}
SOURCE_SUFFIXES = {".cpp", ".cc", ".cxx"}


BINARY_SYMBOLS = {
    "PtrMemD": ".*", "PtrMemI": "->*", "Mul": "*", "Div": "/", "Rem": "%",
    "Add": "+", "Sub": "-", "Shl": "<<", "Shr": ">>", "Cmp": "<=>", "LT": "<",
    "GT": ">", "LE": "<=", "GE": ">=", "EQ": "==", "NE": "!=", "And": "&",
    "Xor": "^", "Or": "|", "LAnd": "&&", "LOr": "||", "Assign": "=",
    "MulAssign": "*=", "DivAssign": "/=", "RemAssign": "%=", "AddAssign": "+=",
    "SubAssign": "-=", "ShlAssign": "<<=", "ShrAssign": ">>=", "AndAssign": "&=",
    "XorAssign": "^=", "OrAssign": "|=", "Comma": ",",
}  # fmt: skip


@dataclass(frozen=True)
class Unit:
    """Um no da arvore, em dois rotulos.

    - exact: o no com a declaracao referida pelo USR. A janela compara este.
    - written: o que esta escrito, com a declaracao pelo nome; no implicito
      (conversao, por exemplo) fica None e nao entra. A semelhanca compara
      este, para que a mesma logica com outro tipo, ou dentro de template,
      continue parecida.
    """

    exact: str
    written: str | None
    local: tuple[str, int] | None
    line: int


@dataclass(frozen=True)
class Function:
    name: str
    short_name: str
    path: Path
    first_line: int
    last_line: int
    units: tuple[Unit, ...]


@dataclass(frozen=True)
class Clone:
    detector: str
    score: float
    first: Function
    first_lines: tuple[int, int]
    second: Function
    second_lines: tuple[int, int]

    def describe(self, root: Path) -> str:
        def side(function: Function, lines: tuple[int, int]) -> str:
            path = function.path.relative_to(root)
            return f"{path}:{lines[0]}-{lines[1]} ({function.name})"

        return (
            f"{side(self.first, self.first_lines)} ~ "
            f"{side(self.second, self.second_lines)} "
            f"[{self.detector} {self.score:.2f}]"
        )


@dataclass(frozen=True)
class Settings:
    window: int
    min_units: int
    ratio: float


def inside(location: ci.SourceLocation, extent: ci.SourceRange) -> bool:
    if location.file is None or extent.start.file is None:
        return False
    if location.file.name != extent.start.file.name:
        return False
    return extent.start.offset <= location.offset <= extent.end.offset


def qualified_name(cursor: ci.Cursor) -> str:
    parts = [cursor.displayname]
    parent = cursor.semantic_parent
    while parent is not None and parent.kind != K.TRANSLATION_UNIT:
        parts.append(parent.spelling or "(anonimo)")
        parent = parent.semantic_parent
    return "::".join(reversed(parts))


def unary_label(cursor: ci.Cursor) -> str:
    tokens = list(cursor.get_tokens())
    if not tokens:
        return "OP:unario"
    if tokens[0].kind == ci.TokenKind.PUNCTUATION:
        return f"OP:pre{tokens[0].spelling}"
    return f"OP:pos{tokens[-1].spelling}"


def operator_symbol(declaration: ci.Cursor) -> str | None:
    """O simbolo de um operador sobrecarregado ('==' de 'operator=='), ou None."""
    name = declaration.spelling
    if not name.startswith("operator"):
        return None
    symbol = name.removeprefix("operator").strip()
    if not symbol or symbol[0].isalnum() or symbol[0] == "_":
        return None
    return symbol


def written_name(cursor: ci.Cursor) -> str:
    """O nome como esta escrito. No template, a libclang da o nome dependente
    (o 'size' de 'text.size()') vazio; ele e' o ultimo identificador do no."""
    if cursor.spelling:
        return cursor.spelling
    names = [token.spelling for token in cursor.get_tokens() if token.kind == ci.TokenKind.IDENTIFIER]
    return names[-1] if names else ""


def called(kind: ci.CursorKind, label: str) -> str:
    """A chamada, escrita, e' so a chamada: o nome do chamado vem no no filho,
    que e' o que existe tambem no template, onde a chamada nao tem nome."""
    return kind.name if kind == K.CALL_EXPR else label


def local_key(location: ci.SourceLocation) -> tuple[str, int]:
    return (location.file.name, location.offset)


def describe_unit(cursor: ci.Cursor, function: ci.Cursor) -> Unit:
    kind = cursor.kind
    line = cursor.location.line
    if kind in LITERAL_KINDS:
        spelling = "".join(token.spelling for token in cursor.get_tokens())
        label = f"{kind.name}:{spelling}"
        return Unit(exact=label, written=label, local=None, line=line)
    if kind in OPERATOR_KINDS:
        label = "OP:" + BINARY_SYMBOLS.get(cursor.binary_operator.name, "?")
        return Unit(exact=label, written=label, local=None, line=line)
    if kind == K.UNARY_OPERATOR:
        label = unary_label(cursor)
        return Unit(exact=label, written=label, local=None, line=line)
    if kind == K.ARRAY_SUBSCRIPT_EXPR:
        return Unit(exact="OP:[]", written="OP:[]", local=None, line=line)
    written: str | None = None if kind == K.UNEXPOSED_EXPR else kind.name
    if cursor.location.file is not None and kind.is_declaration():
        if inside(cursor.location, function.extent):
            local = local_key(cursor.location)
            return Unit(exact=kind.name, written=written, local=local, line=line)
    if not (kind.is_reference() or kind.is_expression()):
        return Unit(exact=kind.name, written=written, local=None, line=line)
    referenced = cursor.referenced
    if referenced is None or referenced.kind.is_invalid():
        # Nome dependente de template: so a grafia diz o que e chamado.
        # Outro no sem declaracao (conversao implicita, por exemplo) nao leva
        # a grafia, que pode ser o nome de uma variavel local.
        if kind.is_reference() or kind in NAMED_EXPRESSION_KINDS:
            label = f"{kind.name}:{written_name(cursor)}"
            return Unit(exact=label, written=written and called(kind, label), local=None, line=line)
        return Unit(exact=kind.name, written=written, local=None, line=line)
    if referenced == function:
        label = f"{kind.name}:$self"
        return Unit(exact=label, written=written and label, local=None, line=line)
    if inside(referenced.location, function.extent):
        local = local_key(referenced.location)
        return Unit(exact=kind.name, written=written and f"{kind.name}:L", local=local, line=line)
    exact = f"{kind.name}:{referenced.get_usr() or referenced.spelling}"
    symbol = operator_symbol(referenced)
    if symbol is not None:
        # Operador sobrecarregado: escrito, e' o operador. A referencia ao
        # operador dentro da chamada e' implicita.
        written = f"OP:{symbol}" if kind == K.CALL_EXPR else None
    elif written is not None:
        written = called(kind, f"{kind.name}:{referenced.spelling}")
    return Unit(exact=exact, written=written, local=None, line=line)


def function_units(function: ci.Cursor) -> tuple[Unit, ...]:
    units: list[Unit] = []
    stack = list(reversed(list(function.get_children())))
    while stack:
        cursor = stack.pop()
        units.append(describe_unit(cursor, function))
        stack.extend(reversed(list(cursor.get_children())))
    return tuple(units)


def generated_by_macro(body: ci.Cursor, sources: dict[str, bytes]) -> bool:
    """O corpo que, no arquivo, nao comeca por '{' veio da expansao de uma macro.

    A posicao vem da arvore: no corpo expandido de macro ela e' a da chamada
    da macro. O get_tokens nao serve: devolve os tokens da definicao dela.
    """
    start = body.extent.start
    if start.file is None:
        return True
    name = start.file.name
    if name not in sources:
        sources[name] = Path(name).read_bytes()
    return sources[name][start.offset : start.offset + 1] != b"{"


def collect(
    cursor: ci.Cursor,
    root: Path,
    found: dict[tuple[str, int], Function],
    sources: dict[str, bytes],
) -> None:
    for child in cursor.get_children():
        location = child.location
        if location.file is None:
            continue
        path = Path(location.file.name).resolve()
        if not path.is_relative_to(root):
            continue
        if child.kind in SCOPE_KINDS:
            collect(child, root, found, sources)
            continue
        if child.kind not in FUNCTION_KINDS or not child.is_definition():
            continue
        bodies = [part for part in child.get_children() if part.kind == K.COMPOUND_STMT]
        if not bodies or generated_by_macro(bodies[-1], sources):
            continue
        key = (str(path), child.extent.start.offset)
        if key in found:
            continue
        found[key] = Function(
            name=qualified_name(child),
            short_name=child.spelling,
            path=path,
            first_line=child.extent.start.line,
            last_line=child.extent.end.line,
            units=function_units(child),
        )


def parse_tree(root: Path, arguments: list[str]) -> list[Function]:
    index = ci.Index.create()
    found: dict[tuple[str, int], Function] = {}
    sources: dict[str, bytes] = {}
    for source in sorted(root.rglob("*")):
        if source.suffix not in SOURCE_SUFFIXES or not source.is_file():
            continue
        unit = index.parse(str(source), args=arguments)
        errors = [
            diagnostic
            for diagnostic in unit.diagnostics
            if diagnostic.severity >= ci.Diagnostic.Error
        ]
        if errors:
            details = "\n".join(str(diagnostic) for diagnostic in errors)
            raise RuntimeError(f"{source} nao compila; o corpus nao mede nada:\n{details}")
        collect(unit.cursor, root, found, sources)
    return list(found.values())


def distance_codes(function: Function) -> list[object]:
    """A local vira a distancia ate o uso anterior dela; a primeira vez, zero."""
    previous: dict[tuple[str, int], int] = {}
    codes: list[object] = []
    for position, unit in enumerate(function.units):
        if unit.local is None:
            codes.append(unit.exact)
            continue
        distance = position - previous.get(unit.local, position)
        previous[unit.local] = position
        codes.append((unit.exact, distance))
    return codes


def window_key(codes: list[object], start: int, size: int) -> tuple[object, ...]:
    key: list[object] = []
    for offset in range(size):
        code = codes[start + offset]
        if isinstance(code, tuple) and code[1] > offset:
            code = (code[0], 0)
        key.append(code)
    return tuple(key)


def span_lines(function: Function, start: int, end: int) -> tuple[int, int]:
    # No implicito pode vir sem posicao (linha 0).
    lines = [unit.line for unit in function.units[start:end] if unit.line > 0]
    return (min(lines), max(lines)) if lines else (function.first_line, function.last_line)


def window_clones(functions: list[Function], size: int) -> list[Clone]:
    seen: dict[tuple[object, ...], list[tuple[int, int]]] = {}
    for number, function in enumerate(functions):
        codes = distance_codes(function)
        for start in range(len(codes) - size + 1):
            seen.setdefault(window_key(codes, start, size), []).append((number, start))

    # Casamentos seguidos com o mesmo deslocamento viram uma regiao so.
    matches: dict[tuple[int, int, int], list[int]] = {}
    for places in seen.values():
        for left in range(len(places)):
            for right in range(left + 1, len(places)):
                (first, first_start), (second, second_start) = places[left], places[right]
                if first == second and abs(first_start - second_start) < size:
                    continue
                shift = second_start - first_start
                matches.setdefault((first, second, shift), []).append(first_start)

    clones: list[Clone] = []
    for (first, second, shift), starts in matches.items():
        starts.sort()
        region_start = previous = starts[0]
        for start in [*starts[1:], None]:
            if start is not None and start == previous + 1:
                previous = start
                continue
            end = previous + size
            clones.append(
                Clone(
                    detector="janela",
                    score=float(end - region_start),
                    first=functions[first],
                    first_lines=span_lines(functions[first], region_start, end),
                    second=functions[second],
                    second_lines=span_lines(
                        functions[second], region_start + shift, end + shift
                    ),
                )
            )
            if start is not None:
                region_start = previous = start
    return clones


def similarity_labels(function: Function) -> list[str]:
    return [unit.written for unit in function.units if unit.written is not None]


def whole_function_clones(functions: list[Function], settings: Settings) -> list[Clone]:
    labels = [similarity_labels(function) for function in functions]
    clones: list[Clone] = []
    for first in range(len(functions)):
        for second in range(first + 1, len(functions)):
            sizes = len(labels[first]), len(labels[second])
            if min(sizes) < settings.min_units:
                continue
            if 2 * min(sizes) / sum(sizes) < settings.ratio:
                continue
            matcher = difflib.SequenceMatcher(None, labels[first], labels[second], autojunk=False)
            if matcher.real_quick_ratio() < settings.ratio:
                continue
            if matcher.quick_ratio() < settings.ratio:
                continue
            score = matcher.ratio()
            if score < settings.ratio:
                continue
            one, other = functions[first], functions[second]
            clones.append(
                Clone(
                    detector="funcao",
                    score=score,
                    first=one,
                    first_lines=(one.first_line, one.last_line),
                    second=other,
                    second_lines=(other.first_line, other.last_line),
                )
            )
    return clones


def find_clones(functions: list[Function], settings: Settings) -> list[Clone]:
    return window_clones(functions, settings.window) + whole_function_clones(
        functions, settings
    )


def scan(root: Path, arguments: list[str], settings: Settings) -> list[Clone]:
    return find_clones(parse_tree(root.resolve(), arguments), settings)


# ---------------------------------------------------------------------------
# Medida contra o corpus.

REJECT_MARK = "// reprova:"


def corpus_arguments(tree: Path, standard: str) -> list[str]:
    return ["-x", "c++", f"-std=c++{standard}", "-I", str(tree / "include")]


def assemble(base: Path, case: Path, target: Path) -> None:
    shutil.copytree(base, target)
    for source in case.rglob("*"):
        if source.is_file():
            destination = target / source.relative_to(case)
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)


def expected_pairs(case: Path, tree: Path) -> list[tuple[Path, int, str]]:
    """Cada marca diz: a funcao desta linha e copia da funcao com este nome."""
    pairs = []
    for source in sorted(case.rglob("*")):
        if not source.is_file():
            continue
        lines = source.read_text(encoding="utf-8").splitlines()
        for number, text in enumerate(lines, start=1):
            if REJECT_MARK in text:
                original = text.split(REJECT_MARK, 1)[1].split()[0]
                pairs.append(((tree / source.relative_to(case)).resolve(), number, original))
    return pairs


def holds(function: Function, path: Path, line: int) -> bool:
    return function.path == path and function.first_line <= line <= function.last_line


def judge_defect(
    clones: list[Clone], pairs: list[tuple[Path, int, str]]
) -> list[str]:
    problems = []
    if not pairs:
        return ["o caso nao tem marca de defeito"]
    for path, line, original in pairs:
        found = any(
            (holds(clone.first, path, line) and clone.second.short_name == original)
            or (holds(clone.second, path, line) and clone.first.short_name == original)
            for clone in clones
        )
        if not found:
            problems.append(f"nao reprovou a copia de {original} na linha {line}")
    for clone in clones:
        touches = any(
            holds(clone.first, path, line) or holds(clone.second, path, line)
            for path, line, _ in pairs
        )
        if not touches:
            problems.append("falso positivo: " + clone.describe(Path("/")))
    return problems


def judge_case(
    kind: str,
    base: Path,
    case: Path,
    scratch: str,
    extra: list[str],
    standard: str,
    settings: Settings,
) -> tuple[list[str], list[str]]:
    """Monta a arvore do caso (a base mais o caso) e julga. Devolve os
    problemas e as copias achadas."""
    tree = Path(scratch) / case.name
    if kind == "base":
        shutil.copytree(base, tree)
    else:
        assemble(base, case, tree)
    root = tree.resolve()
    clones = scan(tree, corpus_arguments(tree, standard) + extra, settings)
    found = [clone.describe(root) for clone in clones]
    if kind == "defeito":
        return judge_defect(clones, expected_pairs(case, tree)), found
    return ["falso positivo: " + text for text in found], found


def measure(
    corpus: Path, extra: list[str], standard: str, settings: Settings, verbose: bool
) -> bool:
    base = corpus / "base"
    defects = sorted(path for path in (corpus / "defeitos").iterdir() if path.is_dir())
    correct = sorted(path for path in (corpus / "corretas").iterdir() if path.is_dir())
    cases = [("base", base)] + [("defeito", case) for case in defects]
    cases += [("correta", case) for case in correct]
    # Cada arvore e' independente: um processo por caso.
    with tempfile.TemporaryDirectory(prefix="kinein-duplicacao-") as scratch:
        with concurrent.futures.ProcessPoolExecutor() as pool:
            futures = [
                pool.submit(judge_case, kind, base, case, scratch, extra, standard, settings)
                for kind, case in cases
            ]
            results = [future.result() for future in futures]
    passed = {"base": 0, "defeito": 0, "correta": 0}
    for (kind, case), (problems, found) in zip(cases, results):
        passed[kind] += not problems
        for problem in problems:
            print(f"{case.name}: {problem}")
        if verbose and not problems:
            for text in found:
                print(f"{case.name}: ok: {text}")
    print(
        f"defeitos reprovados: {passed['defeito']} de {len(defects)}; "
        f"formas corretas aceitas: {passed['correta']} de {len(correct)}; "
        f"base limpa: {'sim' if passed['base'] else 'nao'}"
    )
    return (
        passed["base"] == 1
        and passed["defeito"] == len(defects)
        and passed["correta"] == len(correct)
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--libclang", help="caminho da libclang, se nao for a padrao")
    parser.add_argument("--resource-dir", help="o de `clang -print-resource-dir`")
    parser.add_argument("--window", type=int, default=44)
    parser.add_argument("--min-units", type=int, default=40)
    parser.add_argument("--ratio", type=float, default=0.85)
    parser.add_argument("--std", default="23")
    parser.add_argument("--verbose", action="store_true")
    commands = parser.add_subparsers(dest="command", required=True)
    corpus = commands.add_parser("measure", help="mede o detector contra o corpus")
    corpus.add_argument("corpus", type=Path)
    tree = commands.add_parser("scan", help="procura copias numa arvore")
    tree.add_argument("root", type=Path)
    tree.add_argument("arguments", nargs=argparse.REMAINDER)
    options = parser.parse_args()
    if options.libclang:
        ci.Config.set_library_file(options.libclang)
    extra = ["-resource-dir", options.resource_dir] if options.resource_dir else []
    settings = Settings(window=options.window, min_units=options.min_units, ratio=options.ratio)
    if options.command == "measure":
        passed = measure(options.corpus, extra, options.std, settings, options.verbose)
        return 0 if passed else 1
    clones = scan(options.root, options.arguments + extra, settings)
    for clone in clones:
        print(clone.describe(options.root.resolve()))
    return 1 if clones else 0


if __name__ == "__main__":
    sys.exit(main())
