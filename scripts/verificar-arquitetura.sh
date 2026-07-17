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
cd "$(dirname "$0")/.."

BASELINE="scripts/arquitetura-baseline.txt"

# Limites: UI pela ARCHITECTURE.md §6 (QML visual 300, controller/host 400,
# C++ 500); CORE pela ARCHITECTURE.md §4 (~400-500 linhas de codigo fora dos
# testes) — ver docs/arquitetura/27-modulos-por-dominio.md.
#
# O core entrou em 2026-07-16. Antes disso a catraca varria SO a UI, e a §4 —
# que existe desde sempre e e boa — nunca foi verificada por ninguem: 7 arquivos
# ja a violavam, dois deles (`lib.rs`, `handlers/lsp.rs`) violando a
# responsabilidade que ela lhes atribui por escrito, nao so o tamanho. Mesmo
# diagnostico da UI: regra sem dente apodrece calada.
listar() {
    python3 - <<'PY'
import pathlib, subprocess

LIMITE_RUST = 500  # ARCHITECTURE.md §4.


def limite_ui(caminho: str) -> int:
    if caminho.startswith("ui/src/"):
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
    """Linhas de CODIGO: fora dos testes, fora dos comentarios, fora do branco.

    A §4 sempre disse "linhas de CODIGO fora dos testes". Ate 2026-07-17 esta
    funcao contava linhas de TEXTO, e o gate contradizia a regra que ele mesmo
    cita — a classe de mentira que o verificar-docs.sh existe para pegar.

    O argumento ja estava aqui, aplicado so' aos testes: contar o arquivo
    inteiro "puniria justamente quem testa bem, e a catraca passaria a empurrar
    na direcao errada". Vale igual para comentario. Um invariante medido, escrito
    onde o proximo leitor tropeca nele, e' o que impede a fatia inutil; cobrar o
    arquivo por explicar-se empurra na direcao de apagar justamente isso — e a
    propria catraca chama de trapaca ("NAO corte uma linha qualquer para caber").
    Comentario nao e' volume: e' o que faz 400 linhas serem legiveis.

    MEDIDO ANTES E DEPOIS, porque a mudanca podia ser autoengano — e o "depois"
    corrigiu o "antes". Nos tres god-files de C++ a diferenca e' pequena (1% a 8%:
    editor_highlighter.cpp 910 -> 818; core_client_dispatch.cpp 804 -> 757;
    core_client_requests.cpp 660 -> 551) e nenhum sai do debito: seguem acima de
    500 so' de codigo, a divida deles e' real. Mas no REPOSITORIO o efeito foi
    outro: 23 -> 18 arquivos em debito. Cinco estavam la' apenas por se
    explicarem — lib.rs, EditorFindBar, SearchPanel, TerminalViewport e
    SearchController. E `terminal.rs` caiu 955 -> 657: era cobrado por 298 linhas
    de comentario e branco, quase um terco do arquivo.

    Ou seja: a catraca vinha mandando cinco arquivos "refatorarem" o que ja estava
    certo, e cobrando do melhor documentado do core como se fosse o pior. A regra
    corrige a DIRECAO do incentivo; para quem esta gordo de codigo mesmo, nao muda
    nada — que e' exatamente o que se quer dos dois lados.

    Nao ha parser aqui de proposito: `//`, `/*...*/` e `#` cobrem Rust, C++, QML e
    JS, que e' tudo que este repositorio tem. Uma linha de codigo com comentario
    ao lado conta como CODIGO (o corte e' pelo inicio da linha) — cortar pelo meio
    exigiria entender string e regex, e um contador que erra em silencio seria
    pior que o texto cru que ele substitui.
    """
    texto = pathlib.Path(caminho).read_text()
    corte = texto.find("#[cfg(test)]")
    if corte >= 0:
        texto = texto[:corte]

    codigo = 0
    em_bloco = False
    for linha in texto.splitlines():
        s = linha.strip()
        if not s:
            continue
        if em_bloco:
            if "*/" in s:
                em_bloco = False
            continue
        if s.startswith("/*"):
            if "*/" not in s[2:]:
                em_bloco = True
            continue
        if s.startswith("//") or s.startswith("#"):
            continue
        codigo += 1
    return codigo


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
        echo "  os dois sao trapaca. Tres suspeitos, NESTA ordem (§4 regra 9):" >&2
        echo "  1) a SUA MUDANCA — se ela poe aqui responsabilidade de outro dono," >&2
        echo "     devolva-a e o arquivo encolhe sozinho;" >&2
        echo "  2) a CATEGORIA — arquivo que faz UMA coisa pode estar no limite" >&2
        echo "     errado (composicao nao e visual; ja aconteceu, ver regra 9);" >&2
        echo "  3) o ARQUIVO — so entao o split e fatia PROPRIA, por" >&2
        echo "     responsabilidade, nunca por linhas. Nos casos ja medidos, o" >&2
        echo "     arquivo NAO era o culpado em 2 de 3." >&2
        echo "  Comentario e linha em branco nao contam: explicar-se e gratis." >&2
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
