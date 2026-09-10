//! O PROGRAMA que roda do outro lado.
//!
//! Ele e' pequeno de proposito: tudo que pode ser decidido em Rust e' decidido
//! em Rust. O que so' o `SymPy` sabe fazer — `sympify`, `nsimplify`, `dsolve`,
//! `sstr` e a decomposicao dimensional — e' o que esta aqui.
//!
//! **Arquivo proprio desde 2026-09-10.** Ele e' `Python` dentro de uma string
//! Rust: quem le' o transporte nao quer atravessar 180 linhas de outra
//! linguagem, e quem le' o `Python` nao quer procura-lo no meio de um
//! `Command::spawn`.

/// O texto do programa.
///
/// Ele e' pequeno de proposito: tudo que pode ser decidido em Rust e' decidido
/// em Rust. O que so' o `SymPy` sabe fazer e' `sympify`, `nsimplify`, `dsolve` e
/// `sstr` — e e' exatamente isso que esta aqui.
pub(super) const PROGRAMA: &str = r#"
import json, sys

def linha(objeto):
    """Escreve UMA resposta e descarrega o buffer.

    Duas linhas, e a BARATA primeiro. O `dsolve` pode nao voltar — o pendulo
    nao linearizado nao volta em 20 s —, e quando o teto mata o processo tudo
    que ja' foi descarregado continua valendo. Antes desta separacao, uma
    equacao cuja EDO trava perdia junto o veredito de unidade, que estava
    pronto em 3 ms. Medido em 2026-09-10.
    """
    sys.stdout.write(json.dumps(objeto) + "\n")
    sys.stdout.flush()

def fatal(motivo):
    linha({"kind": "fatal", "reason": motivo})
    raise SystemExit(0)

try:
    import sympy as sp
    import sympy.physics.units as unidades
    from sympy.physics.units.systems import SI
except Exception:
    fatal("noSympy")

try:
    pedido = json.load(sys.stdin)
except Exception:
    fatal("badRequest")

t = sp.Symbol("t")
y = sp.Function("y")

# --- a EDO -----------------------------------------------------------------

def resolver(edo):
    try:
        # `nsimplify(..., rational=True)` troca todo literal float por Rational.
        # Sem isso o `dsolve` cai em RecursionError depois de 4 segundos.
        corpo = sp.nsimplify(sp.sympify(edo["formula"]), rational=True)
        troca = {}
        if edo.get("estado"):
            troca[sp.Symbol(edo["estado"])] = y(t)
        if edo.get("derivada"):
            troca[sp.Symbol(edo["derivada"])] = y(t).diff(t)
        if edo.get("tempo"):
            troca[sp.Symbol(edo["tempo"])] = t
        for nome, valor in edo.get("parametros", []):
            troca[sp.Symbol(nome)] = sp.Rational(valor)
        # `simultaneous=True` impede que a troca de `x` por `y(t)` seja
        # reprocessada pela troca seguinte — cascata daria OUTRA equacao.
        lado = corpo.subs(troca, simultaneous=True)
        ordem = int(edo["ordem"])
        equacao = sp.Eq(y(t).diff(t, ordem), lado)
        inicio = {y(0): sp.Rational(edo["y0"])}
        if ordem == 2:
            inicio[y(t).diff(t).subs(t, 0)] = sp.Rational(edo["dy0"])
        solucao = sp.dsolve(equacao, y(t), ics=inicio)
    except (NotImplementedError, RecursionError):
        return {"ok": False, "reason": "cannotSolve"}
    except SystemExit:
        raise
    except Exception:
        return {"ok": False, "reason": "failed"}
    if isinstance(solucao, (list, tuple)):
        return {"ok": False, "reason": "cannotSolve"}
    return {"ok": True, "expression": sp.sstr(solucao.rhs)}

# --- as DIMENSOES ----------------------------------------------------------

# O vocabulario de unidades. Uma unidade que nao esta aqui NAO vira simbolo
# livre em silencio: `traduzir` recusa, porque simbolo livre e' ADIMENSIONAL
# para o SymPy e o veredito sairia errado sem nada reclamar.
VOCABULARIO = {
    "m": unidades.meter, "s": unidades.second, "kg": unidades.kilogram,
    "N": unidades.newton, "J": unidades.joule, "C": unidades.coulomb,
    "K": unidades.kelvin, "mol": unidades.mole, "A": unidades.ampere,
    "rad": unidades.radian, "Hz": unidades.hertz, "ohm": unidades.ohm,
    "V": unidades.volt, "W": unidades.watt, "Pa": unidades.pascal,
}

# Funcoes cujo argumento tem de ser ADIMENSIONAL. `sin(x)` com `x` em metros
# nao e' fisica: e' erro de unidade que produz numero.
TRANSCENDENTES = (sp.sin, sp.cos, sp.tan, sp.exp, sp.log, sp.asin, sp.acos,
                  sp.atan, sp.sinh, sp.cosh, sp.tanh)

SISTEMA = SI.get_dimension_system()

def traduzir(texto):
    if not texto or not texto.strip():
        return sp.Integer(1)
    expressao = sp.sympify(texto.replace(".", "*").replace("^", "**"),
                           locals=VOCABULARIO)
    desconhecidas = [str(x) for x in expressao.free_symbols]
    if desconhecidas:
        raise ValueError("unidade desconhecida: " + ", ".join(sorted(desconhecidas)))
    return expressao

def em_base(expressao):
    dimensao = unidades.Dimension(SI.get_dimensional_expr(expressao))
    cru = SISTEMA.get_dimensional_dependencies(dimensao)
    # O EXPOENTE VOLTA COMO FLOAT quando a formula tem potencia nao inteira, e
    # e' comum: `(x^2+y^2)^1.5` da' `length^1.00000000000000`, que NAO e' igual
    # a `length^1` num dicionario. Medido em 2026-09-10 contra a orbita, cujas
    # equacoes CERTAS foram reprovadas por isso. A chave vira nome e o expoente
    # vira float; a comparacao e' numerica, com folga.
    return {nome_da_dimensao(d): float(e) for d, e in cru.items() if abs(float(e)) > 1e-9}

def nome_da_dimensao(d):
    return str(d).replace("Dimension(", "").split(",")[0].rstrip(")")

def mesma_dimensao(a, b):
    if set(a) != set(b):
        return False
    return all(abs(a[chave] - b[chave]) < 1e-9 for chave in a)

def dimensionar(pedido):
    try:
        expressao = sp.sympify(pedido["formula"])
        troca = {}
        for indice, (variavel, unidade) in enumerate(sorted(pedido["unidades"])):
            # Cada variavel ganha um simbolo POSITIVO proprio. Substituir pela
            # unidade crua faz termos iguais se CANCELAREM (`a*x - b*v` vira
            # `u - u = 0`), e a dimensao de zero e' 1 — medido em 2026-09-10.
            troca[sp.Symbol(variavel)] = (sp.Symbol("kv%d" % indice, positive=True)
                                          * traduzir(unidade))
        direita = sp.expand(expressao.subs(troca, simultaneous=True))
        esquerda = em_base(traduzir(pedido["esquerda"]))
    except ValueError as erro:
        return {"label": pedido["label"], "verdict": "unknownUnit", "detail": str(erro)[:120]}
    except SystemExit:
        raise
    except Exception:
        return {"label": pedido["label"], "verdict": "unreadable", "detail": ""}

    for no in sp.preorder_traversal(direita):
        if isinstance(no, TRANSCENDENTES):
            for argumento in no.args:
                if em_base(argumento):
                    return {"label": pedido["label"], "verdict": "dimensionalArgument",
                            "detail": type(no).__name__.lower()}

    # TODO addend tem de ter a mesma dimensao, e a conferencia e' FEITA AQUI.
    # O `check_dimensions` do SymPy fica CEGO quando ha' simbolo livre na soma:
    # medido em 2026-09-10, ele ACEITA `length**3/time**2 + length/time**2`.
    termos = sp.Add.make_args(direita)
    dimensoes = [em_base(termo) for termo in termos]
    if any(not mesma_dimensao(d, dimensoes[0]) for d in dimensoes[1:]):
        return {"label": pedido["label"], "verdict": "incoherent",
                "detail": " != ".join(sorted({nome_da_base(d) for d in dimensoes}))[:120]}
    if not mesma_dimensao(dimensoes[0], esquerda):
        return {"label": pedido["label"], "verdict": "wrongSide",
                "detail": "%s != %s" % (nome_da_base(esquerda), nome_da_base(dimensoes[0]))}
    return {"label": pedido["label"], "verdict": "coherent", "detail": ""}

def nome_da_base(base):
    if not base:
        return "adimensional"
    partes = []
    for nome, expoente in sorted(base.items()):
        # O expoente saiu como float; escreve-se inteiro quando ele e' inteiro,
        # porque `length^1.0` na tela e' ruido.
        arredondado = round(expoente)
        texto = str(arredondado) if abs(expoente - arredondado) < 1e-9 else ("%g" % expoente)
        partes.append(nome if texto == "1" else "%s^%s" % (nome, texto))
    return "*".join(partes)

# A BARATA primeiro: 3 ms por equacao, contra ate' 5 s do `dsolve`.
linha({"kind": "dimensoes", "sympy": sp.__version__,
       "itens": [dimensionar(item) for item in pedido.get("dimensoes", [])]})
if pedido.get("edo"):
    linha({"kind": "edo", "sympy": sp.__version__, **resolver(pedido["edo"])})
"#;
