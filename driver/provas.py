"""Os testes funcionais da prova do Neovim (65 §12.2 e §1).

Cada teste sobe a pilha real (UI offscreen -> supervisor -> nvim), age pelo
caminho de uma pessoa (teclas no Qt) e confere o efeito observável: o texto do
buffer, o disco, a grade, o popup, os diagnósticos, os processos.

Uso: python3 -I provas.py [nome do teste ...]
"""

import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import fixtures  # noqa: E402
from pilha import TMP, Pilha, descendentes, ler_cmdline, ler_stat, vivos  # noqa: E402
from util import esperar, resumo, rodar  # noqa: E402


def novo_dir(nome):
    os.makedirs(TMP, exist_ok=True)
    return tempfile.mkdtemp(prefix=f"{nome}-", dir=TMP)


def estado_nvim(p):
    return p.lua("return require('kinein').state()")


def texto(p, caminho):
    r = p.sup.pedir("texto", path=caminho)
    return r["lines"] if isinstance(r, dict) else None


def esperar_texto(p, caminho, esperado, timeout_s=5.0):
    return esperar(lambda: texto(p, caminho) == esperado, timeout_s,
                   descricao=f"buffer {os.path.basename(caminho)} == {esperado!r}")


def abrir(p, caminho):
    p.sup.pedir("abrir", path=caminho)
    esperar(lambda: estado_nvim(p)["name"] == caminho, 5, descricao="arquivo aberto")
    esperar(lambda: estado_nvim(p)["mode"].startswith("i"), 5, descricao="modo de inserção")


def preparar(p, caminho, linhas, linha, coluna):
    """Estado conhecido: buffer com `linhas`, cursor em (linha, coluna), inserção.
    Só quem ficou fora da inserção (um caso que falhou) recebe o <C-\\><C-n>; o
    autocmd do runtime devolve à inserção."""
    if estado_nvim(p)["mode"] != "i" or p.lua("return vim.fn.pumvisible()") == 1:
        p.lua("vim.api.nvim_input('<C-\\\\><C-n>')")
        time.sleep(0.1)
        esperar(lambda: estado_nvim(p)["mode"] == "i", 5, descricao="volta à inserção")
    p.lua(
        "local path, lines, row, col = ...\n"
        "local buf = vim.fn.bufnr(path)\n"
        "vim.api.nvim_set_current_buf(buf)\n"
        "vim.api.nvim_buf_set_lines(buf, 0, -1, true, lines)\n"
        "vim.api.nvim_win_set_cursor(0, {row, col})",
        caminho, linhas, linha, coluna)
    esperar_texto(p, caminho, linhas)
    p.ui.pedir("esperar_quieto", quieto_ms=60)


def cursor(p):
    st = estado_nvim(p)
    return st["row"], st["col16"]


def contexto_cpp(p, proj, preset="build-20"):
    return p.sup.pedir("contexto_cpp", compile_commands_dir=os.path.join(proj, preset), query_driver="/usr/bin/g++")


def esperar_morte(pids, timeout_s=10):
    esperar(lambda: not vivos(pids), timeout_s, descricao=f"processos {pids} terminaram")


# --- item 1 e R3 da configuração da pessoa ------------------------------------------

def t_inicializacao():
    """Item 1: o nvim sobe pelo supervisor com a configuração da Kinein e sem a
    da pessoa (init.lua, plugins, VIMINIT e VIMRUNTIME plantados)."""
    marcas = novo_dir("marcas-casa")
    casa = novo_dir("casa")
    env = fixtures.casa_plantada(casa, marcas)
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    rodadas = []
    for _ in range(5):
        p = Pilha(proj, nome="inicio", env=env)
        try:
            p.esperar_anexado()
            p.ui.pedir("esperar_quieto", quieto_ms=300)
            st = p.ui.pedir("estado")
            sup = p.sup.pedir("estado")
            pronto = [e for e in sup["eventos"] if e["evento"] == "nvim_pronto"][0]
            rodadas.append({
                "ui_primeiro_quadro_ms": st["primeiro_quadro_ms"],
                "ui_editor_quadro_ms": st["editor_quadro_ms"],
                "nvim_pronto_ms_no_supervisor": pronto["pronto_ms"],
                "sonda_ms": pronto["sonda_ms"],
                "contencao": pronto["contencao"],
                "runtimepath": p.lua("return vim.o.runtimepath"),
                "scripts_carregados": len(p.lua("return vim.fn.getscriptinfo()")),
            })
        finally:
            p.fechar()
    achadas = sorted(os.listdir(marcas))
    return {
        "veredito": "APROVADO" if not achadas else "REPROVADO",
        "marcas": achadas,
        "vetores_plantados": ["init.lua da pessoa", "plugin da pessoa", "pacote em site/pack",
                              "plugin em site", "VIMINIT", "VIMRUNTIME falso", "NVIM_APPNAME"],
        "rodadas": rodadas,
        "ui_editor_quadro_ms": resumo([r["ui_editor_quadro_ms"] for r in rodadas]),
        "nvim_pronto_ms": resumo([r["nvim_pronto_ms_no_supervisor"] for r in rodadas]),
        "nota": "o HOME falso da casa plantada vale para a UI também (cache de fontes frio); os tempos daqui "
                "não são o R2",
    }


# --- itens 3, 4, 5 e 2 ----------------------------------------------------------

def comparar_tinta(tinta, grade, cursor_rc):
    """Célula com texto sem tinta é glifo que faltou; célula vazia com tinta é
    sobra de desenho. O cursor fica fora; o '~' do fim do buffer também. A
    célula à esquerda do cursor também: a barra da inserção começa na borda da
    célula e o antialiasing do retângulo pinta 1 px da vizinha (medido em
    2026-11-14: pixel 49 com 154 de intensidade para a barra em x = 50,34)."""
    faltou, sobrou = [], []
    cr, cc = cursor_rc
    for r, (linha_t, linha_g) in enumerate(zip(tinta, grade)):
        celulas = list(linha_g)
        for c, marca in enumerate(linha_t):
            if r == cr and c in (cc, cc - 1):
                continue
            ch = celulas[c] if c < len(celulas) else " "
            if ch == "~":
                continue
            if ch.strip() and marca == ".":
                faltou.append((r, c, ch))
            if not ch.strip() and marca == "#":
                sobrou.append((r, c))
    return faltou, sobrou


def t_desenho():
    """Item 3: a grade desenhada bate com a grade do Neovim, também depois de
    rolar (grid_scroll), e a grade bate com o buffer."""
    d = novo_dir("desenho")
    caminho, linhas = fixtures.fixture_desempenho(d, 60)
    p = Pilha(d, nome="desenho")
    capturas, verificacoes = [], []
    try:
        p.esperar_anexado()
        abrir(p, caminho)
        passos = [("abertura", []), ("PageDown x3", ["PgDown"] * 3), ("PageUp x1", ["PgUp"]),
                  ("Ctrl+End", ["Ctrl+End"]), ("Ctrl+Home", ["Ctrl+Home"]), ("seta abaixo x45", ["Down"] * 45)]
        for nome, teclas in passos:
            if teclas:
                p.ui.pedir("teclas", seq=teclas)
            p.ui.pedir("esperar_quieto", quieto_ms=150)
            t = p.ui.pedir("tinta")
            st = t["estado"]
            faltou, sobrou = comparar_tinta(t["tinta"], st["grade"], st["cursor"])
            # A grade mostra as linhas do buffer na ordem, desde a do topo: o
            # número na coluna de números e o texto depois do textoff.
            topo, textoff, altura = p.lua("local w = vim.fn.getwininfo(vim.fn.win_getid())[1]\n"
                                          "return {w.topline, w.textoff, w.height}")
            buf = texto(p, caminho)
            fora = []
            for r, linha_g in enumerate(st["grade"][:altura]):
                n = topo + r
                if n > len(buf):
                    break
                esperado = buf[n - 1].replace("\t", "    ")
                if linha_g[:textoff].strip() != str(n) or \
                        linha_g[textoff:].rstrip() != esperado[: len(linha_g) - textoff].rstrip():
                    fora.append(r)
            arquivo_png = os.path.join(p.base, f"captura-{len(capturas)}.png")
            p.ui.pedir("captura", caminho=arquivo_png)
            capturas.append(arquivo_png)
            verificacoes.append({"passo": nome, "linha_do_topo": topo, "linhas_conferidas": min(altura, len(buf)),
                                 "n_faltando": len(faltou), "glifos_faltando": faltou[:10],
                                 "n_sobrando": len(sobrou), "sobrando": sobrou[:10],
                                 "linhas_diferentes_do_buffer": fora})
        erros = p.ui.pedir("estado")["erros_de_protocolo"]
    finally:
        p.fechar()
    ok = all(v["n_faltando"] == 0 and v["n_sobrando"] == 0 and not v["linhas_diferentes_do_buffer"]
             for v in verificacoes)
    return {"veredito": "APROVADO" if ok else "REPROVADO", "linhas_do_arquivo": linhas,
            "verificacoes": verificacoes, "capturas": capturas, "erros_de_protocolo_ui": erros}


def compilar(p, comando, cwd):
    return p.sup.pedir("build", cmd=comando, cwd=cwd, timeout_s=600)


def ler(caminho):
    with open(caminho) as f:
        return f.read()


def esperar_disco(caminho, esperado, timeout_s=5):
    try:
        return esperar(lambda: ler(caminho) == esperado and esperado, timeout_s, descricao="disco igual ao esperado")
    except TimeoutError:
        return ler(caminho)


def t_editar_cpp(gravacao="nvim"):
    """Item 4: abrir, buscar, digitar com recuo automático, Tab/Shift+Tab,
    salvar com Ctrl+S; o disco fica igual ao esperado e o programa compila e roda."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    main = os.path.join(proj, "src/main.cpp")
    p = Pilha(proj, nome=f"editar-cpp-{gravacao}", env={"KINEIN_PROVA_GRAVACAO": gravacao})
    try:
        p.esperar_anexado()
        abrir(p, main)
        # Ctrl+F leva ao main; Home; a função nova entra antes dele.
        p.ui.pedir("teclas", seq=["Ctrl+F"])
        p.ui.pedir("digitar", texto="int main\n")
        esperar(lambda: cursor(p)[0] == 6, 5, descricao="busca levou ao main")
        p.ui.pedir("teclas", seq=["Home"])
        p.ui.pedir("digitar", texto="int triplo(int v) {\nreturn v * 3;\n}\n\n")
        # Na linha do printf: soma(p) vira triplo(soma(p) * 2), digitando.
        p.ui.pedir("teclas", seq=["Ctrl+F"])
        p.ui.pedir("digitar", texto="soma(p)\n")
        p.ui.pedir("digitar", texto="triplo(")
        p.ui.pedir("teclas", seq=["Right"] * 7)
        p.ui.pedir("digitar", texto=" * 2)")
        # Tab e Shift+Tab numa linha nova no fim.
        p.ui.pedir("teclas", seq=["Ctrl+End", "End"])
        p.ui.pedir("digitar", texto="\n// fim")
        p.ui.pedir("teclas", seq=["Home", "Tab"])
        p.ui.pedir("teclas", seq=["Shift+Tab"])
        p.ui.pedir("teclas", seq=["Ctrl+S"])
        esperado = (
            "#include <cstdio>\n#include <vector>\n\n#include \"util.h\"\n\n"
            "int triplo(int v) {\n    return v * 3;\n}\n\n"
            "int main() {\n    const Ponto p{3, 4};\n    std::printf(\"%d\\n\", triplo(soma(p) * 2));\n"
            "    return 0;\n}\n// fim\n"
        )
        disco = esperar_disco(main, esperado)
        modificado = estado_nvim(p)["modified"]
        b = compilar(p, ["cmake", "--build", os.path.join(proj, "build-20")], proj)
        saida = subprocess.run([os.path.join(proj, "build-20", "app")], capture_output=True,
                               text=True).stdout if b["status"] == 0 else ""
        eventos = [e["evento"] for e in p.sup.pedir("estado")["eventos"]]
        erros = p.ui.pedir("estado")["erros_de_protocolo"] + p.sup.pedir("estado")["erros_de_protocolo"]
    finally:
        p.fechar()
    quem = "gravado_pelo_core" if gravacao == "core" else "gravado_pelo_nvim"
    ok = disco == esperado and b["status"] == 0 and saida.strip() == "42" and not modificado and quem in eventos
    return {"veredito": "APROVADO" if ok else "REPROVADO", "gravacao": gravacao, "disco": disco,
            "esperado": esperado, "buffer_modificado_depois_de_salvar": modificado,
            "build": {"status": b["status"], "ms": b["ms"], "stderr": b["stderr"][-1500:]},
            "saida_do_programa": saida.strip(), "evento_de_gravacao": quem in eventos, "erros_de_protocolo": erros}


def t_editar_rust():
    """Item 5: o mesmo em Rust, com cargo build e o programa rodando."""
    proj = fixtures.projeto_rust(novo_dir("rust"))
    main = os.path.join(proj, "src/main.rs")
    p = Pilha(proj, nome="editar-rust")
    try:
        p.esperar_anexado()
        abrir(p, main)
        p.ui.pedir("teclas", seq=["Ctrl+F"])
        p.ui.pedir("digitar", texto="fn main\n")
        esperar(lambda: cursor(p)[0] == 3, 5, descricao="busca levou ao main")
        p.ui.pedir("teclas", seq=["Home"])
        p.ui.pedir("digitar", texto="fn triplo(v: i32) -> i32 {\nv * 3\n}\n\n")
        p.ui.pedir("teclas", seq=["Ctrl+F"])
        p.ui.pedir("digitar", texto="util::dobro\n")
        p.ui.pedir("digitar", texto="triplo(")
        p.ui.pedir("teclas", seq=["Ctrl+F"])
        p.ui.pedir("digitar", texto="i32)\n")
        p.ui.pedir("teclas", seq=["Right"] * 5)
        p.ui.pedir("digitar", texto=")")
        p.ui.pedir("teclas", seq=["Ctrl+S"])
        esperado = (
            "mod util;\n\nfn triplo(v: i32) -> i32 {\n    v * 3\n}\n\nfn main() {\n"
            "    let valores: Vec<i32> = vec![1, 2, 3];\n"
            "    println!(\"{}\", triplo(util::dobro(valores.len() as i32)));\n}\n"
        )
        disco = esperar_disco(main, esperado)
        b = compilar(p, ["cargo", "build", "--quiet"], proj)
        saida = subprocess.run([os.path.join(proj, "target", "debug", "prova_rust")], capture_output=True,
                               text=True).stdout if b["status"] == 0 else ""
        erros = p.ui.pedir("estado")["erros_de_protocolo"] + p.sup.pedir("estado")["erros_de_protocolo"]
    finally:
        p.fechar()
    ok = disco == esperado and b["status"] == 0 and saida.strip() == "18"
    return {"veredito": "APROVADO" if ok else "REPROVADO", "disco": disco, "esperado": esperado,
            "build": {"status": b["status"], "ms": b["ms"], "stderr": b["stderr"][-1500:]},
            "saida_do_programa": saida.strip(), "erros_de_protocolo": erros}


# --- item 6 e R8: a tabela de atalhos -----------------------------------------

def t_atalhos():
    """Item 6 e R8: cada atalho do 65 §12.3 que não depende de LSP (os de LSP
    estão no clangd, no rust-analyzer e nas ações), sem saber o modo do Vim.
    Depois de cada ação, uma tecla comum ainda insere texto: ninguém ficou
    preso num modo."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    arq = fixtures.escrever(proj, "src/atalhos.cpp", "x\n")
    p = Pilha(proj, nome="atalhos")
    resultados = []

    def caso(nome, linhas, pos, acoes, esperado=None, checar=None, aceitos=None):
        r = {"acao": nome}
        try:
            preparar(p, arq, linhas, pos[0], pos[1])
            for tipo, valor in acoes:
                if tipo == "teclas":
                    p.ui.pedir("teclas", seq=valor)
                elif tipo == "digitar":
                    p.ui.pedir("digitar", texto=valor)
                elif tipo == "clip":
                    p.ui.pedir("area_transferencia", texto=valor)
                elif tipo == "quieto":
                    p.ui.pedir("esperar_quieto", quieto_ms=valor)
            opcoes = aceitos or ([esperado] if esperado is not None else None)
            if opcoes is not None:
                r["buffer"] = esperar(lambda: (texto(p, arq) in opcoes) and texto(p, arq), 4,
                                      descricao=f"{nome}: buffer em {opcoes!r}")
            if checar:
                r.update(checar())
            r["modo_depois"] = estado_nvim(p)["mode"]
            # A tecla comum depois da ação: insere ou substitui, nunca vira comando.
            antes = "".join(texto(p, arq)).count("Q")
            p.ui.pedir("digitar", texto="Q")
            esperar(lambda: "".join(texto(p, arq)).count("Q") == antes + 1, 3,
                    descricao=f"{nome}: a tecla seguinte inseriu texto")
            r["tecla_seguinte_insere"] = True
            r["ok"] = r.get("ok", True)
        except Exception as e:
            r["ok"] = False
            r["erro"] = f"{type(e).__name__}: {e}"[:600]
            try:
                r["buffer"] = texto(p, arq)
                r["modo_depois"] = estado_nvim(p)["mode"]
            except Exception:
                pass
        resultados.append(r)

    def clip_igual(valor):
        return lambda: {"ok": bool(esperar(lambda: p.ui.pedir("area_transferencia") == valor, 3,
                                           descricao=f"área de transferência == {valor!r}"))}

    try:
        p.esperar_anexado()
        abrir(p, arq)
        caso("digitar insere", ["abc"], (1, 1), [("digitar", "XY")], ["aXYbc"])
        caso("Shift+→ seleciona; digitar substitui (e não mexe na área de transferência)", ["abcdef"], (1, 1),
             [("clip", "CLIP"), ("teclas", ["Shift+Right"] * 3), ("digitar", "Z")], ["aZef"],
             checar=clip_igual("CLIP"))
        caso("Shift+← seleciona para trás", ["abcdef"], (1, 4),
             [("teclas", ["Shift+Left"] * 2), ("digitar", "Z")], ["abZef"])
        caso("Shift+↓ seleciona até a linha de baixo", ["abc", "def"], (1, 1),
             [("teclas", ["Shift+Down"]), ("digitar", "Z")], ["aZef"])
        caso("Shift+End seleciona até o fim", ["abc def"], (1, 4), [("teclas", ["Shift+End"]), ("digitar", "x")],
             ["abc x"])
        caso("Shift+Home seleciona até o início", ["abc def"], (1, 4),
             [("teclas", ["Shift+Home"]), ("digitar", "x")], ["xdef"])
        caso("Ctrl+Shift+→ seleciona palavra", ["alfa beta"], (1, 0),
             [("teclas", ["Ctrl+Shift+Right"]), ("digitar", "x")], aceitos=[["x beta"], ["xbeta"]])
        caso("Ctrl+C copia a seleção (e a mantém)", ["alfa beta"], (1, 0),
             [("teclas", ["Shift+Right"] * 4 + ["Ctrl+C"])], ["alfa beta"], checar=clip_igual("alfa"))
        caso("Ctrl+X recorta", ["alfa beta"], (1, 0), [("teclas", ["Shift+Right"] * 4 + ["Ctrl+X"])], [" beta"],
             checar=clip_igual("alfa"))
        caso("Ctrl+V cola", ["ab"], (1, 1), [("clip", "XYZ"), ("teclas", ["Ctrl+V"])], ["aXYZb"])
        caso("Ctrl+V substitui a seleção (e a área de transferência fica)", ["abcd"], (1, 1),
             [("clip", "XY"), ("teclas", ["Shift+Right", "Shift+Right", "Ctrl+V"])], ["aXYd"],
             checar=clip_igual("XY"))
        caso("Ctrl+V no fim da linha sobre a seleção", ["abcd"], (1, 2),
             [("clip", "XY"), ("teclas", ["Shift+End", "Ctrl+V"])], ["abXY"])
        caso("Ctrl+V duas vezes cola o mesmo texto", ["ab"], (1, 1),
             [("clip", "XY"), ("teclas", ["Shift+Right", "Ctrl+V", "Ctrl+V"])], ["aXYXY"])
        caso("Ctrl+Z desfaz a última palavra", ["base"], (1, 4),
             [("digitar", " um dois"), ("quieto", 100), ("teclas", ["Ctrl+Z"])], ["base um"])
        caso("Ctrl+Shift+Z refaz", ["base"], (1, 4),
             [("digitar", " um dois"), ("quieto", 100), ("teclas", ["Ctrl+Z"]), ("quieto", 100),
              ("teclas", ["Ctrl+Shift+Z"])], ["base um dois"])
        caso("Ctrl+A seleciona tudo", ["um", "dois"], (1, 0), [("teclas", ["Ctrl+A"]), ("digitar", "x")], ["x"])
        caso("Ctrl+S salva", ["salvo"], (1, 5), [("teclas", ["Ctrl+S"])], ["salvo"],
             checar=lambda: {"ok": bool(esperar(lambda: ler(arq) == "salvo\n", 3, descricao="disco salvo"))})
        caso("Ctrl+D duplica a linha", ["um", "dois"], (1, 1), [("teclas", ["Ctrl+D"])], ["um", "um", "dois"])
        caso("Ctrl+Y apaga a linha", ["um", "dois", "tres"], (2, 0), [("teclas", ["Ctrl+Y"])], ["um", "tres"])
        caso("Ctrl+/ comenta", ["int a;"], (1, 0), [("teclas", ["Ctrl+/"])], ["// int a;"])
        caso("Ctrl+/ descomenta", ["// int a;"], (1, 0), [("teclas", ["Ctrl+/"])], ["int a;"])
        caso("Alt+Shift+↑ sobe a linha", ["um", "dois", "tres"], (2, 0), [("teclas", ["Alt+Shift+Up"])],
             ["dois", "um", "tres"])
        caso("Alt+Shift+↓ desce a linha", ["um", "dois", "tres"], (2, 0), [("teclas", ["Alt+Shift+Down"])],
             ["um", "tres", "dois"])
        caso("Tab recua", ["x"], (1, 0), [("teclas", ["Tab"])], ["    x"])
        caso("Shift+Tab desfaz o recuo", ["    x"], (1, 4), [("teclas", ["Shift+Tab"])], ["x"])
        caso("Tab com seleção recua as linhas", ["a", "b"], (1, 0), [("teclas", ["Shift+Down", "Shift+Right", "Tab"])],
             ["    a", "    b"])
        caso("Ctrl+F busca", ["alfa", "beta", "gama alvo"], (1, 0),
             [("teclas", ["Ctrl+F"]), ("digitar", "alvo\n")], ["alfa", "beta", "gama alvo"],
             checar=lambda: {"ok": bool(esperar(lambda: cursor(p) == (3, 5), 3, descricao="cursor no alvo"))})
        caso("Esc não tira da edição", ["ab"], (1, 1), [("teclas", ["Escape"])], ["ab"])
        caso("Home e End", ["  abc"], (1, 3), [("teclas", ["End"]), ("digitar", "!"), ("teclas", ["Home"]),
                                                ("digitar", "#")], aceitos=[["#  abc!"], ["  #abc!"]])
        caso("Backspace apaga a seleção", ["abcd"], (1, 1), [("teclas", ["Shift+Right", "Shift+Right", "Backspace"])],
             ["ad"])
    finally:
        erros = p.ui.pedir("estado")["erros_de_protocolo"]
        p.fechar()
    falhas = [r for r in resultados if not r.get("ok")]
    return {"veredito": "APROVADO" if not falhas else "REPROVADO", "n": len(resultados), "acoes": resultados,
            "falhas": [r["acao"] for r in falhas], "erros_de_protocolo": erros}


# --- item 7 e R9 ----------------------------------------------------------------

def t_unicode_ime():
    proj = novo_dir("unicode")
    arq = fixtures.escrever(proj, "texto.txt", "x\n")
    p = Pilha(proj, nome="unicode")
    casos = []
    try:
        p.esperar_anexado()
        abrir(p, arq)

        def caso(nome, acao, esperado, avanco_celulas):
            preparar(p, arq, ["x"], 1, 1)
            antes = p.ui.pedir("estado")["cursor"]
            r = {"caso": nome}
            try:
                extra = acao() or {}
                r.update(extra if isinstance(extra, dict) else {})
                r["buffer"] = esperar(lambda: (texto(p, arq) == [esperado]) and texto(p, arq), 3,
                                      descricao=f"{nome}: buffer")
                p.ui.pedir("esperar_quieto", quieto_ms=80)
                depois = p.ui.pedir("estado")
                r["avanco_de_celulas"] = depois["cursor"][1] - antes[1]
                r["a11y"] = [depois["a11y_linha"], depois["a11y_coluna"]]
                r["ok"] = r.get("ok", True) and r["avanco_de_celulas"] == avanco_celulas and \
                    depois["a11y_linha"] == esperado and \
                    depois["a11y_coluna"] == len(esperado.encode("utf-16-le")) // 2
            except Exception as e:
                r["ok"] = False
                r["erro"] = f"{type(e).__name__}: {e}"[:500]
            casos.append(r)

        def celula(png, estado):
            from PIL import Image
            cw, ch = estado["celula"]
            lin, col = estado["cursor"]
            img = Image.open(png).convert("RGB")
            caixa = (round(col * cw), round(lin * ch), round((col + 1) * cw), round((lin + 1) * ch))
            return list(img.crop(caixa).getdata())

        def acento_morto():
            # A composição aparece no cursor antes de confirmar (a célula muda
            # de pixels), e o buffer não muda até o commit.
            p.ui.pedir("esperar_quieto", quieto_ms=80)
            antes_png = os.path.join(p.base, "ime-antes.png")
            antes = p.ui.pedir("captura", caminho=antes_png)["estado"]
            r1 = p.ui.pedir("ime", eventos=[{"preedit": "´"}])
            p.ui.pedir("esperar_quieto", quieto_ms=80)
            durante_png = os.path.join(p.base, "ime-durante.png")
            p.ui.pedir("captura", caminho=durante_png)
            mudou = sum(a != b for a, b in zip(celula(antes_png, antes), celula(durante_png, antes)))
            sem_mudar = texto(p, arq) == ["x"]
            p.ui.pedir("ime", eventos=[{"preedit": "", "commit": "é"}])
            return {"pixels_mudados_na_celula_do_cursor": mudou, "buffer_intacto_na_composicao": sem_mudar,
                    "retangulo_do_cursor": r1["cursor_rect"], "alvo_e_o_editor": r1["alvo_e_o_editor"],
                    "ok": mudou > 0 and sem_mudar and r1["alvo_e_o_editor"]}

        caso("acento morto (´ + e) pelo IME", acento_morto, "xé", 1)
        caso("tecla com texto direto (ç)", lambda: p.ui.pedir("digitar", texto="ç"), "xç", 1)
        caso("largura dupla (中文)", lambda: p.ui.pedir("digitar", texto="中文"), "x中文", 4)
        caso("emoji (😀)", lambda: p.ui.pedir("digitar", texto="😀"), "x😀", 2)
        caso("combinante (e + U+0301) pelo IME", lambda: p.ui.pedir("ime", eventos=[{"commit": "é"}]),
             "xé", 1)
        cap = os.path.join(p.base, "unicode.png")
        p.ui.pedir("captura", caminho=cap)
    finally:
        p.fechar()
    ime = shutil.which("ibus-daemon") or shutil.which("fcitx5") or shutil.which("fcitx")
    ok = all(c.get("ok") for c in casos)
    return {
        "veredito": ("APROVADO" if ok else "REPROVADO") + ("" if ime else " no caminho de eventos do Qt; "
                                                                          "IME real NÃO PROVADO"),
        "framework_de_ime": ime,
        "casos": casos,
        "captura": cap,
        "limite": "sem ibus nem fcitx instalados, o teste manda QInputMethodEvent direto ao editor: prova o "
                  "tratamento da composição pelo NvimView, não o caminho do framework até ele",
    }


# --- itens 8, 9, 11 e R7 ----------------------------------------------------------

def esperar_cliente(p, nome, timeout_s):
    return esperar(lambda: any(c["name"] == nome for c in estado_nvim(p)["clients"]), timeout_s,
                   descricao=f"cliente {nome} anexado")


def pedir_lista(p, gatilho, timeout_s):
    """Digita o gatilho e espera a lista do Neovim no popup da UI; sem lista
    pelo autotrigger, pede com Ctrl+Space. Devolve (cumprido, ms, itens, via)."""
    t0 = time.monotonic()
    p.ui.pedir("digitar", texto=gatilho)
    r = p.ui.pedir("esperar_popup", visivel=True, timeout_ms=int(timeout_s * 1000), timeout_s=timeout_s + 5)
    via = "autotrigger"
    if not r["cumprido"]:
        via = "Ctrl+Space"
        p.ui.pedir("teclas", seq=["Ctrl+Space"])
        r = p.ui.pedir("esperar_popup", visivel=True, timeout_ms=int(timeout_s * 1000), timeout_s=timeout_s + 5)
    return r["cumprido"], (time.monotonic() - t0) * 1000, [i["word"] for i in r["estado"]["popup_itens"]], via


def completar(p, caminho, base, pos, gatilho, membro, filtro, esperado, limite_s, por_tentativa_s=10):
    """O caminho da pessoa: gatilho, lista, filtro, Enter, preencher o campo do
    snippet, Tab para sair dele, ';'. Enquanto o servidor ainda não tem a
    análise (frio), a lista não tem `membro`: Esc e de novo, até `limite_s`.
    `esperado(linha)` confere a linha final."""
    tentativas = []
    t0 = time.monotonic()
    while True:
        preparar(p, caminho, base, *pos)
        ok, ms, itens, via = pedir_lista(p, gatilho, por_tentativa_s)
        tentativas.append({"popup_ms": round(ms, 1), "via": via, "itens": itens[:5]})
        if ok and any(i.lstrip().startswith(membro) for i in itens):
            break
        if time.monotonic() - t0 > limite_s:
            return {"ok": False, "tentativas": tentativas}
        p.ui.pedir("teclas", seq=["Escape"])
        time.sleep(1)
    r = {"tentativas": tentativas, "ate_a_primeira_lista_util_s": round(time.monotonic() - t0, 2),
         "popup_ms": tentativas[-1]["popup_ms"], "via": tentativas[-1]["via"]}
    linha = pos[0] - 1
    p.ui.pedir("digitar", texto=filtro)
    p.ui.pedir("esperar_quieto", quieto_ms=150)
    r["selecionado_depois_do_filtro"] = p.ui.pedir("estado")["popup_itens"][:3]
    p.ui.pedir("teclas", seq=["Return"])
    # O CompleteDone troca o rótulo pelo snippet; o campo fica selecionado.
    r["aceito"] = esperar(lambda: "…" not in texto(p, caminho)[linha] and "(" in texto(p, caminho)[linha]
                          and texto(p, caminho)[linha], 5, descricao="snippet aplicado")
    r["modo_depois_de_aceitar"] = estado_nvim(p)["mode"]
    p.ui.pedir("digitar", texto="7")
    p.ui.pedir("teclas", seq=["Tab"])
    # O rust-analyzer põe o ';' no snippet de um método que devolve () em
    # posição de instrução (push, não push_mut): a pessoa vê o ';' e não o
    # digita de novo.
    if r["aceito"].rstrip().endswith(";"):
        r["ponto_e_virgula"] = "do servidor"
    else:
        r["ponto_e_virgula"] = "digitado"
        p.ui.pedir("digitar", texto=";")
    p.ui.pedir("esperar_quieto", quieto_ms=150)
    r["linha"] = esperar(lambda: (lambda t: t.endswith(";") and "7" in t and t)(texto(p, caminho)[linha]), 3,
                         descricao="linha final")
    # Depois do snippet, a próxima tecla é texto: inserção, sem seleção aberta.
    r["modo_final"] = estado_nvim(p)["mode"]
    r["ok"] = bool(esperado(r["linha"])) and r["modo_final"] == "i"
    return r


def latencias(p, nome):
    for s in p.sup.pedir("estado")["lsp"]["servidores"]:
        if s["nome"] == nome:
            return {m: resumo(v) for m, v in s["latencias_ms"].items()}, s
    return {}, None


def erros_no_buffer(p, caminho):
    return p.lua("return #vim.diagnostic.get(vim.fn.bufnr(...), {severity = vim.diagnostic.severity.ERROR})",
                 caminho)


def t_clangd():
    """Item 8: completion pela UI, definição e diagnóstico, tudo pelo relé."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    main = os.path.join(proj, "src/main.cpp")
    p = Pilha(proj, nome="clangd")
    r = {}
    try:
        p.esperar_anexado()
        contexto_cpp(p, proj)
        t0 = time.monotonic()
        abrir(p, main)
        esperar_cliente(p, "clangd", 30)
        r["cliente_anexado_ms"] = (time.monotonic() - t0) * 1000
        # Completion: "v." numa std::vector, filtra "push_b", Enter, 7, Tab, ";".
        base = ["#include <vector>", "", "int main() {", "    std::vector<int> v;", "    ", "}"]
        quer = lambda linha: linha == "    v.push_back(7);"  # noqa: E731
        r["completion"] = completar(p, main, base, (5, 4), "v.", "push_back", "push_b", quer, 60)
        r["completion_quente"] = [completar(p, main, base, (5, 4), "v.", "push_back", "push_b", quer, 10)
                                  for _ in range(3)]
        r["completion"]["selection_depois"] = p.lua("return vim.o.selection")
        r["completion"]["ok"] = r["completion"]["ok"] and r["completion"]["selection_depois"] == "exclusive"
        # Definição: Ctrl+B em soma.
        preparar(p, main, ler(main).rstrip("\n").split("\n"), 8, 26)
        p.ui.pedir("teclas", seq=["Ctrl+B"])
        destino = esperar(lambda: estado_nvim(p)["name"].endswith(("util.h", "util.cpp")) and estado_nvim(p), 10,
                          descricao="definição aberta")
        r["definicao"] = {"arquivo": os.path.basename(destino["name"]), "linha": destino["row"],
                          "ok": (os.path.basename(destino["name"]), destino["row"]) in (("util.h", 8),
                                                                                       ("util.cpp", 3))}
        # Diagnóstico: um erro digitado aparece no buffer e no painel.
        abrir(p, main)
        preparar(p, main, ["int main() {", "    int y = 0;", "    return y;", "}"], 2, 12)
        p.ui.pedir("teclas", seq=["Shift+Right"])
        t0 = time.monotonic()
        p.ui.pedir("digitar", texto="\"texto\"")
        esperar(lambda: erros_no_buffer(p, main) > 0, 15, descricao="erro no buffer")
        r_buf = (time.monotonic() - t0) * 1000
        uri = "file://" + main
        painel = esperar(lambda: [d for d in p.sup.pedir("diagnosticos").get(uri, []) if d.get("severity") == 1], 15,
                         descricao="erro no painel")
        r["diagnostico"] = {"linha": texto(p, main)[1], "do_teclado_ao_buffer_ms": r_buf,
                            "no_painel": painel[0]["message"], "ok": True}
        r["latencias_no_rele"], servidor = latencias(p, "clangd")
        r["servidor"] = {k: servidor[k] for k in ("args", "lancamentos", "conexoes", "recusadas")}
    finally:
        p.fechar()
    ok = r.get("completion", {}).get("ok") and all(c.get("ok") for c in r.get("completion_quente", [{}])) and \
        r.get("definicao", {}).get("ok") and r.get("diagnostico", {}).get("ok")
    r["veredito"] = "APROVADO" if ok else "REPROVADO"
    return r


def t_rust_analyzer():
    """Item 9: o mesmo com o rust-analyzer (frio: indexa a std na primeira vez)."""
    proj = fixtures.projeto_rust(novo_dir("rust"))
    main = os.path.join(proj, "src/main.rs")
    p = Pilha(proj, nome="rust-analyzer")
    r = {}
    try:
        p.esperar_anexado()
        t0 = time.monotonic()
        abrir(p, main)
        esperar_cliente(p, "rust-analyzer", 30)
        r["cliente_anexado_ms"] = (time.monotonic() - t0) * 1000
        base = ["mod util;", "", "fn main() {", "    let mut w: Vec<i32> = Vec::new();", "    ", "}"]
        # Depois do filtro "push", o primeiro da lista é o que vier primeiro na
        # ordem do servidor (o Neovim não reordena): push ou push_mut.
        quer = lambda linha: linha in ("    w.push(7);", "    w.push_mut(7);")  # noqa: E731
        r["completion"] = completar(p, main, base, (5, 4), "w.", "push", "push", quer, 180, por_tentativa_s=2)
        r["completion_quente"] = [completar(p, main, base, (5, 4), "w.", "push", "push", quer, 10)
                                  for _ in range(3)]
        r["completion"]["selection_depois"] = p.lua("return vim.o.selection")
        r["completion"]["ok"] = r["completion"]["ok"] and r["completion"]["selection_depois"] == "exclusive"
        preparar(p, main, ["mod util;", "", "fn main() {", "    println!(\"{}\", util::dobro(2));", "}"], 4, 26)
        p.ui.pedir("teclas", seq=["Ctrl+B"])
        destino = esperar(lambda: estado_nvim(p)["name"].endswith("util.rs") and estado_nvim(p), 15,
                          descricao="definição aberta")
        r["definicao"] = {"arquivo": "util.rs", "linha": destino["row"], "ok": destino["row"] == 1}
        abrir(p, main)
        preparar(p, main, ["mod util;", "", "fn main() {", "    let x: i32 = \"s\";", "    let _ = x;", "}"], 4, 0)
        p.ui.pedir("teclas", seq=["Ctrl+S"])
        t0 = time.monotonic()
        esperar(lambda: erros_no_buffer(p, main) > 0, 120, descricao="erro no buffer", intervalo_s=0.2)
        r["diagnostico"] = {"do_salvar_ao_buffer_ms": (time.monotonic() - t0) * 1000, "ok": True}
        uri = "file://" + main
        painel = esperar(lambda: [d for d in p.sup.pedir("diagnosticos").get(uri, []) if d.get("severity") == 1], 30,
                         descricao="erro no painel")
        r["diagnostico"]["no_painel"] = painel[0]["message"][:200]
        r["latencias_no_rele"], servidor = latencias(p, "rust-analyzer")
        r["servidor"] = {k: servidor[k] for k in ("args", "lancamentos", "conexoes", "recusadas")}
    finally:
        p.fechar()
    ok = r.get("completion", {}).get("ok") and all(c.get("ok") for c in r.get("completion_quente", [{}])) and \
        r.get("definicao", {}).get("ok") and r.get("diagnostico", {}).get("ok")
    r["veredito"] = "APROVADO" if ok else "REPROVADO"
    return r


def t_contexto():
    """Item 10 e R11: trocar o preset (C++17 -> C++20) no core muda o que o
    clangd diz, pelo mesmo relé, sem configuração no Neovim."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    arq = os.path.join(proj, "src/padrao.cpp")
    uri = "file://" + arq
    p = Pilha(proj, nome="contexto")
    r = {}

    def erros_no_rele():
        return [d["message"] for d in p.sup.pedir("diagnosticos").get(uri, []) if d.get("severity") == 1]

    try:
        p.esperar_anexado()
        contexto_cpp(p, proj, "build-17")
        abrir(p, arq)
        esperar_cliente(p, "clangd", 30)
        t0 = time.monotonic()
        esperar(lambda: erros_no_buffer(p, arq) > 0, 20, descricao="erros com C++17")
        _, servidor = latencias(p, "clangd")
        r["cpp17"] = {"erros_no_buffer": erros_no_buffer(p, arq), "ms": (time.monotonic() - t0) * 1000,
                      "mensagens": erros_no_rele()[:4], "args": servidor["args"]}
        t0 = time.monotonic()
        r["buffers_reconectados"] = contexto_cpp(p, proj, "build-20")
        # O relé guarda o último publishDiagnostics: sem erro nele quer dizer
        # que o clangd novo já publicou o arquivo (o velho tinha erros).
        def visto():
            return {"lancamentos": latencias(p, "clangd")[1]["lancamentos"], "erros_no_rele": erros_no_rele(),
                    "clientes": [(c["name"], c["buffers"]) for c in estado_nvim(p)["clients"]],
                    "erros_no_buffer": erros_no_buffer(p, arq)}
        try:
            esperar(lambda: (lambda v: v["lancamentos"] == 2 and not v["erros_no_rele"] and v["clientes"] and
                             v["erros_no_buffer"] == 0)(visto()), 30, descricao="clangd novo sem erros com C++20")
            _, servidor = latencias(p, "clangd")
            r["cpp20"] = {"erros_no_buffer": 0, "ms": (time.monotonic() - t0) * 1000, "args": servidor["args"]}
        except TimeoutError as e:
            r["erro"] = str(e)[:300]
            r["visto_no_fim_da_espera"] = visto()
            r["eventos_do_supervisor"] = p.sup.pedir("estado")["eventos"][-6:]
        _, servidor = latencias(p, "clangd")
        r["servidor"] = {k: servidor[k] for k in ("lancamentos", "conexoes", "recusadas", "ativo", "pid")}
        r["eventos_do_rele"] = p.sup.pedir("estado")["lsp"]["eventos"]
    finally:
        p.fechar()
    ok = r.get("cpp17", {}).get("erros_no_buffer", 0) > 0 and "cpp20" in r and \
        any("build-17" in a for a in r["cpp17"]["args"]) and any("build-20" in a for a in r["cpp20"]["args"])
    r["veredito"] = "APROVADO" if ok else "REPROVADO"
    return r


def t_acoes():
    """Item 11: diagnóstico no texto e no painel; ação de código aplicada pelo
    Alt+Enter; renomear pelo Shift+F6 em três arquivos, e o projeto compila."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    main = os.path.join(proj, "src/main.cpp")
    util_h = os.path.join(proj, "src/util.h")
    util_cpp = os.path.join(proj, "src/util.cpp")
    p = Pilha(proj, nome="acoes")
    r = {}
    try:
        p.esperar_anexado()
        contexto_cpp(p, proj)
        abrir(p, main)
        esperar_cliente(p, "clangd", 30)
        uri = "file://" + main
        aviso = esperar(lambda: [d for d in p.sup.pedir("diagnosticos").get(uri, []) if "vector" in d["message"]], 20,
                        descricao="include sem uso no painel")
        r["diagnostico_no_painel"] = aviso[0]["message"]
        r["diagnostico_no_buffer"] = p.lua("return vim.tbl_map(function(d) return d.message end, "
                                           "vim.diagnostic.get(0))")
        p.lua("vim.api.nvim_win_set_cursor(0, {2, 0})")
        p.ui.pedir("esperar_quieto", quieto_ms=150)
        p.ui.pedir("teclas", seq=["Alt+Return"])
        lista = p.esperar_grade("Type number and <Enter>", timeout_ms=10000)
        if not lista["cumprido"]:
            raise RuntimeError("o menu de ações não apareceu: " + repr(lista["estado"]["grade"][-6:]))
        menu = [l.rstrip() for l in lista["estado"]["grade"] if l.strip()[:2].rstrip(":").isdigit()]
        r["menu_de_acoes"] = menu
        escolha = next((l.strip().split(":")[0] for l in menu if "emove" in l), "1")
        p.ui.pedir("digitar", texto=escolha + "\n")
        sem_include = esperar(lambda: "#include <vector>" not in texto(p, main) and texto(p, main), 10,
                              descricao="include removido")
        r["acao_aplicada"] = "#include <vector>" not in sem_include
        # Renomear soma -> somar a partir do util.h. O clangd roda sem índice
        # de fundo; os três arquivos abertos entram no índice dinâmico.
        abrir(p, util_cpp)
        abrir(p, util_h)
        p.lua("vim.api.nvim_win_set_cursor(0, {8, 5})")
        p.ui.pedir("esperar_quieto", quieto_ms=300)
        p.ui.pedir("teclas", seq=["Shift+F6"])
        prompt = p.esperar_grade("New Name", timeout_ms=10000)
        if not prompt["cumprido"]:
            raise RuntimeError("o prompt de renomear não apareceu: " + repr(prompt["estado"]["grade"][-3:]))
        p.ui.pedir("teclas", seq=["Ctrl+U"])
        p.ui.pedir("digitar", texto="somar\n")
        arquivos = [main, util_h, util_cpp]
        esperar(lambda: all(any("somar" in l for l in (texto(p, a) or [])) for a in arquivos), 15,
                descricao="renomeado nos três arquivos")
        r["renomeado_em"] = [os.path.basename(a) for a in arquivos]
        p.lua("vim.cmd('silent wall')")
        b = compilar(p, ["cmake", "--build", os.path.join(proj, "build-20")], proj)
        r["build_depois"] = {"status": b["status"], "stderr": b["stderr"][-800:]}
        r["latencias_no_rele"], _ = latencias(p, "clangd")
    finally:
        p.fechar()
    ok = r.get("acao_aplicada") and r.get("renomeado_em") and r.get("build_depois", {}).get("status") == 0
    r["veredito"] = "APROVADO" if ok else "REPROVADO"
    return r


def t_sessao_unica():
    """R7: quatro buffers da mesma linguagem e raiz usam uma sessão; um segundo
    cliente forçado é recusado pelo relé e não vira segundo processo."""
    proj = fixtures.projeto_cpp(novo_dir("cpp"))
    p = Pilha(proj, nome="sessao")
    r = {}
    try:
        p.esperar_anexado()
        contexto_cpp(p, proj)
        for arq in ("src/main.cpp", "src/util.cpp", "src/util.h", "src/padrao.cpp"):
            abrir(p, os.path.join(proj, arq))
        esperar_cliente(p, "clangd", 30)
        time.sleep(1)
        r["clientes"] = [{"nome": c["name"], "buffers": len(c["buffers"])} for c in estado_nvim(p)["clients"]]
        socket = p.sup.pedir("estado")["lsp"]["servidores"][0]["socket"]
        p.lua("local socket, root = ...\n"
              "vim.lsp.start({name = 'clangd-segundo', cmd = vim.lsp.rpc.connect(socket), root_dir = root},"
              " {reuse_client = function() return false end})", socket, proj)
        time.sleep(2)
        st = p.sup.pedir("estado")["lsp"]["servidores"]
        r["servidores"] = [{k: s[k] for k in ("nome", "lancamentos", "conexoes", "recusadas", "pid")} for s in st]
        r["processos_clangd"] = len([pid for pid in descendentes(p.pid_ui())
                                     if (ler_stat(pid) or {}).get("nome", "").startswith("clangd")])
    finally:
        p.fechar()
    s0 = r["servidores"][0]
    ok = r["processos_clangd"] == 1 and len(r["servidores"]) == 1 and s0["lancamentos"] == 1 and s0["recusadas"] >= 1
    r["veredito"] = "APROVADO" if ok else "REPROVADO"
    return r


def t_dono_unico():
    """R6: a edição da Kinein com changedtick velho é recusada; o espelho segue
    o buffer; o supervisor não tem caminho que escreva no buffer sem tick."""
    proj = novo_dir("dono")
    arq = fixtures.escrever(proj, "a.txt", "um\ndois\n")
    p = Pilha(proj, nome="dono")
    try:
        p.esperar_anexado()
        abrir(p, arq)
        t = p.sup.pedir("texto", path=arq)
        p.ui.pedir("digitar", texto="x")
        esperar(lambda: p.sup.pedir("texto", path=arq)["changedtick"] > t["changedtick"], 3, descricao="digitado")
        velho = p.sup.pedir("aplicar", path=arq, tick=t["changedtick"], first=0, last=1, lines=["SOBRESCRITO"])
        atual = p.sup.pedir("texto", path=arq)
        certo = p.sup.pedir("aplicar", path=arq, tick=atual["changedtick"], first=1, last=2, lines=["DOIS"])
        final = p.sup.pedir("texto", path=arq)["lines"]
        espelho = bool(esperar(lambda: p.sup.pedir("espelho", path=arq)["lines"] == final, 3,
                               descricao="espelho igual ao buffer"))
    finally:
        p.fechar()
    fontes = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "supervisor", "src")
    escritas = subprocess.run(["grep", "-rn", "nvim_buf_set_lines\\|nvim_buf_set_text\\|nvim_put\\|nvim_input", fontes],
                              capture_output=True, text=True).stdout.strip().splitlines()
    ok = velho["ok"] is False and certo["ok"] is True and "SOBRESCRITO" not in final and espelho and not escritas
    return {"veredito": "APROVADO" if ok else "REPROVADO", "velho_recusado": not velho["ok"],
            "certo_aplicado": certo["ok"], "buffer_final": final, "espelho_igual": espelho,
            "escritas_diretas_no_supervisor": escritas,
            "nota": "o supervisor só muda texto pelo kinein.apply (com changedtick) e pelo kinein.open da "
                    "recuperação, que roda antes de a UI receber o editor"}


# --- itens 12 e 13 (R4 e R5) ----------------------------------------------------

def digitar_no_meio(p, caminho, n, linha):
    preparar(p, caminho, texto(p, caminho), linha, 0)
    digitado = "".join(chr(ord("a") + i % 26) for i in range(n))
    for ch in digitado:
        p.ui.pedir("digitar", texto=ch)
        p.ui.pedir("esperar_quieto", quieto_ms=150)
    return digitado


def recuperados(linhas, linha, digitado):
    alvo = linhas[linha - 1] if linhas and len(linhas) >= linha else ""
    n = 0
    while n < len(digitado) and n < len(alvo) and alvo[n] == digitado[n]:
        n += 1
    return n


def esperar_reanexado(p, geracao, timeout_s=10):
    """A UI anexada ao nvim da `geracao`, depois do nvim_pronto dela."""
    def pronto():
        st = p.ui.pedir("estado")
        eventos = [e for e in st["eventos_supervisor"]
                   if e.get("evento") == "nvim_pronto" and e.get("geracao") == geracao]
        return bool(eventos) and st["anexado"] and st["socket"] == eventos[-1]["socket"] and st
    return esperar(pronto, timeout_s, descricao=f"UI anexada ao nvim da geração {geracao}")


def reabrir_e_ler(d, base, env, caminho, nome):
    q = Pilha(d, nome=nome, env=env, base=base)
    try:
        q.esperar_anexado()
        try:
            linhas = esperar(lambda: texto(q, caminho), 10, descricao="buffer restaurado")
            restaurado = True
        except TimeoutError:
            abrir(q, caminho)
            linhas = texto(q, caminho)
            restaurado = False
        modificado = q.sup.pedir("texto", path=caminho)["modified"]
    finally:
        q.fechar()
    return linhas, restaurado, modificado


def t_recuperacao():
    """Item 12 e R4: o texto digitado e não salvo depois de kill -9 no nvim, na
    UI e no supervisor (o papel do core), com o espelho e com o swap do Neovim."""
    resultados = []
    linha = 1231
    for modo in ("espelho", "swap"):
        for alvo in ("nvim", "ui", "supervisor"):
            for atraso_ms in (0, 200):
                d = novo_dir(f"rec-{modo}-{alvo}")
                caminho, _ = fixtures.fixture_desempenho(d, 60)
                env = {"KINEIN_PROVA_RECUPERACAO": modo}
                p = Pilha(d, nome=f"rec-{modo}-{alvo}", env=env)
                r = {"modo": modo, "queda": alvo, "atraso_ms": atraso_ms}
                try:
                    p.esperar_anexado()
                    abrir(p, caminho)
                    digitado = digitar_no_meio(p, caminho, 40, linha)
                    r["digitados"] = len(digitado)
                    sup_pid = int(p.ui.pedir("estado")["pid_supervisor"])
                    nvim_pid = p.sup.pedir("estado")["pid"]
                    time.sleep(atraso_ms / 1000)
                    t0 = time.monotonic()
                    if alvo == "nvim":
                        p.sup.pedir("matar_nvim")
                        # O supervisor publica a geração nova antes de reabrir os
                        # arquivos; o nvim_pronto sai depois da restauração, e só
                        # então a UI se anexa. Lê-se o que a pessoa vê.
                        esperar_reanexado(p, 2)
                        linhas = texto(p, caminho)
                        r["recuperacao_ms"] = (time.monotonic() - t0) * 1000
                        r["restaurado_sem_acao"] = True
                        r["modificado"] = p.sup.pedir("texto", path=caminho)["modified"]
                        p.fechar()
                    else:
                        os.kill(p.pid_ui() if alvo == "ui" else sup_pid, signal.SIGKILL)
                        if alvo == "ui":
                            p.proc.wait(10)
                        esperar_morte([sup_pid, nvim_pid])
                        p.fechar()
                        linhas, restaurado, modificado = reabrir_e_ler(d, p.base, env, caminho, "reaberta")
                        r["reabrir_e_restaurar_ms"] = (time.monotonic() - t0) * 1000
                        r["restaurado_sem_acao"] = restaurado
                        r["modificado"] = modificado
                    r["recuperados"] = recuperados(linhas, linha, digitado)
                    r["perdidos"] = r["digitados"] - r["recuperados"]
                except Exception as e:
                    r["erro"] = f"{type(e).__name__}: {e}"[:600]
                    r["perdidos"] = None
                    try:
                        p.fechar()
                    except Exception:
                        pass
                resultados.append(r)
                print("  ", r, flush=True)
    ok = all(c["perdidos"] == 0 for c in resultados if c["modo"] == "espelho")
    return {"veredito": "MEDIDO" if ok else "MEDIDO, COM PERDA NO ESPELHO", "casos": resultados,
            "nota": "40 teclas, uma por vez, com 150 ms de quieto entre elas; a queda vem 0 ou 200 ms depois do "
                    "quadro da última"}


def t_recuperacao_mutante():
    """O defeito plantado do R4: sem rascunho, a queda do core perde o texto (o
    teste de recuperação reprova pelo motivo certo)."""
    d = novo_dir("rec-mutante")
    caminho, _ = fixtures.fixture_desempenho(d, 60)
    env = {"KINEIN_PROVA_PLANTIO_SEM_RASCUNHO": "1"}
    p = Pilha(d, nome="rec-mutante", env=env)
    try:
        p.esperar_anexado()
        abrir(p, caminho)
        digitado = digitar_no_meio(p, caminho, 10, 1231)
        sup_pid = int(p.ui.pedir("estado")["pid_supervisor"])
        nvim_pid = p.sup.pedir("estado")["pid"]
        time.sleep(0.2)
        os.kill(sup_pid, signal.SIGKILL)
        esperar_morte([sup_pid, nvim_pid])
    finally:
        p.fechar()
    linhas, restaurado, _ = reabrir_e_ler(d, p.base, env, caminho, "rec-mutante-reaberta")
    n = recuperados(linhas or [], 1231, digitado)
    return {"veredito": "REPROVA PELO MOTIVO CERTO" if n == 0 else "NÃO REPROVOU",
            "digitados": len(digitado), "recuperados": n, "restaurado_sem_acao": restaurado}


def t_orfaos():
    """Item 13 e R5: kill -9 na UI, no supervisor e no nvim, com clangd e
    rust-analyzer rodando; depois, nenhum processo da árvore sobra, e a queda
    do nvim não deixa dois servidores da mesma linguagem."""
    casos = []
    for alvo in ("ui", "supervisor", "nvim", "saida_normal"):
        proj = novo_dir(f"orfaos-{alvo}")
        fixtures.projeto_cpp(proj)
        fixtures.projeto_rust(proj)
        p = Pilha(proj, nome=f"orfaos-{alvo}")
        r = {"queda": alvo}
        arvore = {}
        try:
            p.esperar_anexado()
            contexto_cpp(p, proj)
            abrir(p, os.path.join(proj, "src/main.cpp"))
            esperar_cliente(p, "clangd", 30)
            abrir(p, os.path.join(proj, "src/main.rs"))
            esperar_cliente(p, "rust-analyzer", 30)
            p.ui.pedir("teclas", seq=["Ctrl+S"])  # o cargo check do rust-analyzer
            time.sleep(4)
            arvore = {pid: ler_cmdline(pid)[:160] for pid in descendentes(p.pid_ui())}
            r["arvore_antes"] = sorted({c.split()[0].split("/")[-1] for c in arvore.values() if c})
            r["processos_antes"] = len(arvore)
            if alvo == "ui":
                os.kill(p.pid_ui(), signal.SIGKILL)
                p.proc.wait(10)
            elif alvo == "supervisor":
                os.kill(int(p.ui.pedir("estado")["pid_supervisor"]), signal.SIGKILL)
            elif alvo == "nvim":
                antigo = p.sup.pedir("estado")["pid"]
                p.sup.pedir("matar_nvim")
                esperar(lambda: p.sup.pedir("estado")["geracao"] == 2, 10, descricao="nvim novo")
                p.esperar_anexado()
                esperar_cliente(p, "clangd", 30)
                esperar_cliente(p, "rust-analyzer", 30)
                time.sleep(3)
                r["servidores_depois"] = [{k: s[k] for k in ("nome", "lancamentos", "recusadas", "pid", "ativo")}
                                          for s in p.sup.pedir("estado")["lsp"]["servidores"]]
                por_nome = {}
                for pid in descendentes(p.pid_ui()):
                    # O clangd renomeia a thread principal (clangd.main): pelo cmdline.
                    nome = os.path.basename(ler_cmdline(pid).split(" ")[0])
                    if nome in ("clangd", "rust-analyzer"):
                        por_nome[nome] = por_nome.get(nome, 0) + 1
                r["servidores_vivos_por_nome"] = por_nome
                r["clientes_no_nvim_novo"] = [c["name"] for c in estado_nvim(p)["clients"]]
                r["nvim_antigo_vivo"] = bool(vivos([antigo]))
                arvore.pop(p.sup.pedir("estado")["pid"], None)
                p.fechar()
            else:
                p.fechar()
            time.sleep(3)
            restantes = vivos(list(arvore))
            r["restantes"] = [ler_cmdline(pid)[:160] for pid in restantes]
            r["orfaos"] = len(restantes)
        except Exception as e:
            r["erro"] = f"{type(e).__name__}: {e}"[:600]
        finally:
            p.fechar()
            # Nada desta prova pode ficar rodando.
            for pid in vivos(list(arvore)):
                try:
                    os.kill(pid, signal.SIGKILL)
                except OSError:
                    pass
        casos.append(r)
        print("  ", {k: v for k, v in r.items() if k != "arvore_antes"}, flush=True)
    ok = all(c.get("orfaos") == 0 for c in casos)
    for c in casos:
        if c["queda"] == "nvim":
            ok = ok and c.get("servidores_vivos_por_nome") == {"clangd": 1, "rust-analyzer": 1} and \
                sorted(c.get("clientes_no_nvim_novo", [])) == ["clangd", "rust-analyzer"]
    return {"veredito": "APROVADO" if ok else "REPROVADO", "casos": casos}


# --- item 14 e R3 ------------------------------------------------------------

TENTATIVAS_DE_EXECUCAO = [
    ("ex :!", "vim.cmd('silent! !touch {m}/ex-bang')"),
    ("system()", "pcall(vim.fn.system, 'touch {m}/system')"),
    ("system() com lista", "pcall(vim.fn.system, {{'touch', '{m}/system-lista'}})"),
    ("jobstart()", "pcall(vim.fn.jobstart, {{'touch', '{m}/jobstart'}})"),
    ("vim.system()", "pcall(function() vim.system({{'touch', '{m}/vim-system'}}):wait() end)"),
    ("vim.uv.spawn sh", "vim.uv.spawn('/bin/sh', {{args = {{'-c', 'touch {m}/uv-sh'}}}}, function() end)"),
    ("io.popen", "pcall(function() local f = io.popen('touch {m}/io-popen'); if f then f:close() end end)"),
    ("os.execute", "pcall(os.execute, 'touch {m}/os-execute')"),
    ("terminal", "pcall(vim.cmd, 'terminal touch {m}/terminal')"),
    ("ld.so como trampolim", "vim.uv.spawn('/lib64/ld-linux-x86-64.so.2', {{args = {{'/usr/bin/touch', "
                             "'{m}/ldso'}}}}, function() end)"),
    ("o próprio nvim", "vim.uv.spawn(vim.v.progpath, {{args = {{'--headless', '-c', "
                       "'call writefile([1], \"{m}/nvim-filho\")', '-c', 'qa!'}}}}, function() end)"),
    (":make com makeprg", "vim.o.makeprg = 'touch {m}/make'; pcall(vim.cmd, 'silent! make')"),
    ("vim.ui.open", "vim.ui.open('{m}/ui-open')"),
    ("system(3) da libc pela FFI", "local ffi = require('ffi'); pcall(ffi.cdef, 'int system(const char *);');"
                                   " ffi.C.system('touch {m}/ffi-system')"),
]


def t_hostil(plantio=None):
    """Item 14 e R3: o projeto hostil, sem confiança, com a configuração da
    pessoa plantada, os arquivos abertos e as tentativas de execução de dentro
    do nvim (o que um código que chegasse a rodar no nvim tentaria)."""
    marcas = novo_dir("marcas")
    proj = fixtures.projeto_hostil(novo_dir("hostil"), marcas)
    env = fixtures.casa_plantada(novo_dir("casa"), marcas)
    if plantio in ("sem_landlock", "sem_landlock_sem_sonda"):
        env["KINEIN_PROVA_PLANTIO_SEM_LANDLOCK"] = "1"
    if plantio == "sem_landlock_sem_sonda":
        env["KINEIN_PROVA_PLANTIO_SEM_SONDA"] = "1"
    p = Pilha(proj, nome=f"hostil-{plantio or 'normal'}", env=env, confiavel=False)
    r = {"plantio": plantio}
    try:
        try:
            p.esperar_anexado(timeout_ms=8000)
            r["editor_entregue"] = True
        except RuntimeError:
            r["editor_entregue"] = False
            r["eventos"] = p.ui.pedir("estado")["eventos_supervisor"]
        if r["editor_entregue"]:
            # O contexto pede o compilador hostil no --query-driver: sem
            # confiança, o lançador não o passa ao clangd.
            p.sup.pedir("contexto_cpp", compile_commands_dir=proj, query_driver=f"{proj}/ferramentas/cc-hostil")
            for arq in ("src/hostil.c", "src/main.rs", "src/modeline.txt", "build.rs", "Cargo.toml"):
                abrir(p, os.path.join(proj, arq))
            antes = p.sup.pedir("estado")
            r["antes_das_tentativas"] = {"geracao": antes["geracao"], "pid_nvim": antes["pid"],
                                         "pid_supervisor": p.ui.pedir("estado")["pid_supervisor"],
                                         "servidores": [s["nome"] for s in antes["lsp"]["servidores"]]}
            r["tabstop_depois_da_modeline"] = p.lua("return vim.bo.tabstop")
            r["runtimepath"] = p.lua("return vim.o.runtimepath")
            r["package_cpath"] = p.lua("return package.cpath")
            r["tentativas"] = {}
            for nome, codigo in TENTATIVAS_DE_EXECUCAO:
                try:
                    p.lua(codigo.format(m=marcas))
                    r["tentativas"][nome] = "rodou sem erro de Lua"
                except Exception as e:
                    r["tentativas"][nome] = f"erro: {e}"[:200]
            r["sonda"] = p.sup.pedir("sonda")
            try:
                p.sup.pedir("build", cmd=["cargo", "build"], cwd=proj)
                r["build_sem_confianca"] = "executou"
            except RuntimeError as e:
                r["build_sem_confianca"] = f"recusado: {e}"
            depois = p.sup.pedir("estado")
            r["servidores"] = [{k: s[k] for k in ("nome", "args", "lancamentos")} for s in depois["lsp"]["servidores"]]
            r["depois_das_tentativas"] = {"geracao": depois["geracao"], "pid_nvim": depois["pid"],
                                          "pid_supervisor": p.ui.pedir("estado")["pid_supervisor"]}
            time.sleep(3)
    finally:
        p.fechar()
    r["marcas"] = sorted(os.listdir(marcas))
    if plantio is None:
        ok = not r["marcas"] and r.get("editor_entregue") and r.get("tabstop_depois_da_modeline") != 13
        r["veredito"] = "APROVADO" if ok else "REPROVADO"
    elif plantio == "sem_landlock":
        r["veredito"] = "REPROVA PELO MOTIVO CERTO" if not r.get("editor_entregue") else "NÃO REPROVOU"
    else:
        r["veredito"] = "REPROVA PELO MOTIVO CERTO" if r["marcas"] else "NÃO REPROVOU"
    return r


def t_hostil_biblioteca():
    """O limite da contenção, medido: com Lua já rodando dentro do nvim, uma
    biblioteca nativa carrega (o Landlock EXECUTE trata o execve, não o mmap),
    e o código dela também não consegue executar programa."""
    marcas = novo_dir("marcas-lib")
    proj = fixtures.projeto_hostil(novo_dir("hostil-lib"), marcas)
    p = Pilha(proj, nome="hostil-biblioteca", confiavel=False)
    r = {}
    try:
        p.esperar_anexado()
        try:
            p.lua("return package.loadlib(..., '*')", os.path.join(proj, "kinein.so"))
            r["loadlib"] = "carregou"
        except Exception as e:
            r["loadlib"] = f"erro: {e}"[:200]
        r["sonda_depois"] = p.sup.pedir("sonda")
    finally:
        p.fechar()
    r["marcas"] = sorted(os.listdir(marcas))
    r["veredito"] = "LIMITE CONFIRMADO" if "biblioteca" in r["marcas"] else "SEM EFEITO"
    r["nota"] = "o que impede o projeto de chegar a este ponto é o runtimepath, o package.path/cpath fechados e " \
                "modeline/exrc desligados (os vetores passivos do teste hostil); a contenção do execve vale também " \
                "para o código da biblioteca"
    return r


TESTES = {
    "inicializacao": t_inicializacao,
    "desenho": t_desenho,
    "editar_cpp": t_editar_cpp,
    "editar_cpp_core_grava": lambda: t_editar_cpp("core"),
    "editar_rust": t_editar_rust,
    "atalhos": t_atalhos,
    "unicode_ime": t_unicode_ime,
    "clangd": t_clangd,
    "rust_analyzer": t_rust_analyzer,
    "contexto": t_contexto,
    "acoes": t_acoes,
    "sessao_unica": t_sessao_unica,
    "dono_unico": t_dono_unico,
    "recuperacao": t_recuperacao,
    "recuperacao_mutante": t_recuperacao_mutante,
    "orfaos": t_orfaos,
    "hostil": t_hostil,
    "hostil_mutante_sem_landlock": lambda: t_hostil("sem_landlock"),
    "hostil_mutante_sem_landlock_sem_sonda": lambda: t_hostil("sem_landlock_sem_sonda"),
    "hostil_biblioteca": t_hostil_biblioteca,
}


if __name__ == "__main__":
    nomes = sys.argv[1:] or list(TESTES)
    for nome in nomes:
        rodar(nome, TESTES[nome])
