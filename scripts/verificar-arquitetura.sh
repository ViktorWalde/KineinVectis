#!/usr/bin/env bash
# Catraca da regra de split (ARCHITECTURE.md §6 e §4 do core).
#
# POR QUE EXISTE. A regra de split ja existia e era boa; ela so nao era
# verificada por ninguem. Resultado medido em 2026-07-16: 20 arquivos acima do
# limite, e o `Main.qml` saiu de 336 linhas ("estado validado" escrito na
# ARCHITECTURE.md em 2026-07-06) para 700 em dez dias. Regra que mora so em .md
# nao segura arquitetura — ela apodrece em silencio enquanto o gate fica verde.
#
# POR QUE CATRACA E NAO LIMITE DURO. Falhar nos 20 hoje so ensinaria a
# desligar o script. A catraca e pragmatica: o debito existente fica congelado
# na baseline e so pode DIMINUIR. Arquivo novo acima do limite reprova; arquivo
# em debito que CRESCE reprova. Encolher e sempre aceito, e a baseline deve ser
# atualizada junto (o proprio script diz como).
#
# Uso: bash scripts/verificar-arquitetura.sh
#      bash scripts/verificar-arquitetura.sh --atualizar-baseline
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

BASELINE="scripts/arquitetura-baseline.txt"

# Limites: UI pela ARCHITECTURE.md §6 (QML visual 300, controller/host 400,
# C++ 500); CORE pela ARCHITECTURE.md §4 (~400-500 linhas de codigo fora dos
# testes) — ver DocsPublic/arquitetura/27-modulos-por-dominio.md.
#
# O core entrou em 2026-07-16. Antes disso a catraca varria SO a UI, e a §4 —
# que existe desde sempre e e boa — nunca foi verificada por ninguem: 7 arquivos
# ja a violavam, dois deles (`lib.rs`, `handlers/lsp.rs`) violando a
# responsabilidade que ela lhes atribui por escrito, nao so o tamanho. Mesmo
# diagnostico da UI: regra sem dente apodrece calada.
listar() {
    python3 - <<'PY'
import pathlib, re, subprocess

LIMITE_RUST = 500  # ARCHITECTURE.md §4.

# Um corpo de funcao dentro de um header: `) {`, `) const {`, `) noexcept {`.
# Onde ha corpo ha LOGICA, e logica em header e' outra categoria.
CORPO_DE_FUNCAO = re.compile(r"\)\s*(?:const\s*)?(?:noexcept\s*)?\{")
# O que um header de PONTE declara: metodo chamavel do QML, sinal e propriedade.
ITEM_DECLARADO = re.compile(r"Q_INVOKABLE|Q_PROPERTY|^\s*void\s+\w+\(", re.M)


def limite_ui(caminho: str) -> int:
    if caminho.startswith("ui/src/"):
        # HEADER QUE SO' DECLARA e' outra categoria, e o limite dele e' funcao
        # do CONTRATO que ele espelha — nao da qualidade do codigo.
        #
        # Registrado porque a §4 regra 8 exige que mexer em limite seja
        # explicito. DECISAO DO AUTOR em 2026-09-10, e ela vem depois de a
        # medicao descartar a outra saida: o `core_client.h` tem 422 linhas de
        # DECLARACAO PURA contra 63 de comentario, e mover comentario para o
        # .cpp compraria ~33 linhas contra a convencao da propria linguagem —
        # corte por TAMANHO, que a §4 regra 9 recusa.
        #
        # O que ele carrega, medido em 2026-09-10 com o contador abaixo: 289
        # itens declarados (129 `Q_INVOKABLE`, 18 `Q_PROPERTY`, 142 `void`) em
        # 423 linhas de declaracao, ou **1,46 linha por item**. E' o mesmo
        # argumento que o `Main.qml` ja' tem na §5 da `LEITURA_TECNICA`: o
        # tamanho e' funcao do numero de dominios, nao da qualidade.
        #
        # POR QUE O LIMITE NAO E' UM NUMERO FIXO. Escolher 700 porque 500 nao
        # coube seria levantar limite para caber, e isso e' o que este projeto
        # recusa. O limite e' 2x o que o arquivo DECLARA — ele so' cresce
        # DECLARANDO. Hoje da' 578 contra 500, e a folga de 78 linhas e' o
        # espaco entre a densidade medida (1,46) e o teto (2,00).
        #
        # As duas maneiras de ele reprovar, e as duas sao as certas: comentario
        # que entra sem declaracao come a folga; e logica que entra derruba o
        # arquivo para 500 NA HORA, porque ele deixa de ser desta categoria.
        if caminho.endswith(".h"):
            texto = pathlib.Path(caminho).read_text()
            if not CORPO_DE_FUNCAO.search(texto):
                return max(500, 2 * len(ITEM_DECLARADO.findall(texto)))
        return 500
    # `ui/qml/app/` e COMPOSICAO: instancia dominios e liga fiacao, zero pixel.
    # Ganha 400 pelo mesmo motivo que controller/host ganham — o limite de 300 e
    # de QML VISUAL, e classificar composicao como visual e erro de categoria.
    #
    # Registrado porque a §4 regra 8 exige que mexer em limite seja explicito:
    # aqui NAO se subiu o limite de ninguem. Corrigiu-se a categoria de um
    # arquivo que nunca foi visual. O gatilho foi concreto — espremer o
    # AppDomains em 300 gerou um segundo arquivo com 13 propriedades de
    # pass-through, cerimonia pura para satisfazer um numero. Catraca que
    # empurra para desacoplamento inutil esta errada tanto quanto a que deixa
    # monolito crescer: o que se evita sao os DOIS extremos.
    if caminho.startswith("ui/qml/app/"):
        return 400
    # Controller/store/host e o composition root sao "logica": 400.
    nome = caminho.rsplit("/", 1)[-1]
    if ("Controller" in nome or "Host" in nome or nome == "Main.qml"):
        return 400
    return 300  # QML visual


def linhas_de_codigo(caminho: str) -> int:
    """Linhas FORA dos testes, como a §4 manda contar.

    Em Rust o teste unitario e co-localizado (`#[cfg(test)] mod tests`): contar
    o arquivo inteiro puniria justamente quem testa bem, e a catraca passaria a
    empurrar na direcao errada. Na UI nao ha teste inline (os harnesses vivem em
    scripts/qml-harness), entao o corte nao acha nada e o numero e o arquivo
    todo — a mesma regra serve aos dois sem virar excecao.
    """
    texto = pathlib.Path(caminho).read_text()
    corte = texto.find("#[cfg(test)]")
    if corte >= 0:
        texto = texto[:corte]
    return len(texto.splitlines())


achados = []

for caminho in subprocess.check_output(
        ["find", "ui/qml", "ui/src", "-name", "*.qml", "-o", "-name", "*.cpp",
         "-o", "-name", "*.h"], text=True).split():
    linhas = linhas_de_codigo(caminho)
    lim = limite_ui(caminho)
    if linhas > lim:
        achados.append((caminho, linhas, lim))

for caminho in subprocess.check_output(
        ["find", "crates", "-name", "*.rs", "-not", "-path", "*/target/*"],
        text=True).split():
    # `tests/<dominio>.rs` e teste de integracao: a §4 organiza o core assim de
    # proposito e nao mira a regra de split neles.
    if "/tests/" in caminho or caminho.endswith("/tests.rs"):
        continue
    linhas = linhas_de_codigo(caminho)
    if linhas > LIMITE_RUST:
        achados.append((caminho, linhas, LIMITE_RUST))

for caminho, linhas, lim in sorted(achados):
    print(f"{caminho} {linhas} {lim}")
PY
}

if [ "${1:-}" = "--atualizar-baseline" ]; then
    listar > "$BASELINE"
    echo "baseline atualizada: $(wc -l < "$BASELINE") arquivos em debito"
    echo "Commite junto com a mudanca que a alterou."
    exit 0
fi

if [ ! -f "$BASELINE" ]; then
    echo "erro: $BASELINE ausente. Gere com --atualizar-baseline." >&2
    exit 1
fi

atual="$(listar)"
falhou=0

# Cada camada tem a sua secao da ARCHITECTURE.md. Apontar a errada faz o dev
# procurar a regra no lugar onde ela nao esta.
regra() {
    case "$1" in
        crates/*) echo "ARCHITECTURE.md §4 (core: pasta <dominio>/ com mod.rs + um submodulo por responsabilidade)" ;;
        *) echo "ARCHITECTURE.md §6" ;;
    esac
}

# 1. Arquivo NOVO acima do limite: reprova. Nao se paga debito com debito.
while read -r caminho linhas lim; do
    [ -z "$caminho" ] && continue
    if ! grep -q "^$caminho " "$BASELINE"; then
        echo "✗ NOVO acima do limite: $caminho ($linhas linhas, limite $lim)" >&2
        echo "  Quebre antes de crescer em cima ($(regra "$caminho"))." >&2
        echo "  ATENCAO: o criterio e RESPONSABILIDADE, nao tamanho. Este numero" >&2
        echo "  so manda VOCE OLHAR. Pergunte 'o que esta misturado aqui?' — se o" >&2
        echo "  arquivo faz UMA coisa so, o errado pode ser a categoria, nao ele." >&2
        echo "  Split que nao deixa mais claro nao e split, e cerimonia (§4 regra 9)." >&2
        falhou=1
    fi
done <<< "$atual"

# 2. Arquivo em debito que CRESCEU: reprova. A catraca so gira para um lado.
while read -r caminho linhas lim; do
    [ -z "$caminho" ] && continue
    antes="$(awk -v c="$caminho" '$1==c {print $2}' "$BASELINE")"
    if [ -n "$antes" ] && [ "$linhas" -gt "$antes" ]; then
        echo "✗ CRESCEU em debito: $caminho ($antes -> $linhas, limite $lim)" >&2
        echo "  Este arquivo ja passa do limite; nao pode engordar." >&2
        echo "  Regra: $(regra "$caminho")" >&2
        echo "  NAO corte uma linha qualquer para caber, e NAO suba o baseline:" >&2
        echo "  os dois sao trapaca. Olhe primeiro a SUA MUDANCA — se ela poe" >&2
        echo "  neste arquivo responsabilidade que e de outro dono, devolva-a e o" >&2
        echo "  arquivo encolhe sozinho. Se a sua mudanca esta certa e o arquivo e" >&2
        echo "  que e gordo, o debito dele e fatia PROPRIA (§4 regra 9)." >&2
        falhou=1
    fi
done <<< "$atual"

if [ "$falhou" -ne 0 ]; then
    echo >&2
    echo "✗ catraca de arquitetura FALHOU" >&2
    echo "  Encolher e sempre aceito; depois rode --atualizar-baseline." >&2
    exit 1
fi

restantes="$(printf '%s' "$atual" | grep -c . || true)"
echo "✓ arquitetura: catraca ok ($restantes arquivos em debito, nenhum novo/crescido)"
