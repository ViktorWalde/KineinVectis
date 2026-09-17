#!/usr/bin/env python3
"""Language server FALSO, deterministico, para os testes do core (roadmap 30 §3).

# Por que existe

Ate 2026-09-02, **15 metodos IPC `lsp.*` nao tinham um unico teste de
comportamento**: as 286 linhas de `src/tests/lsp.rs` eram todas "requires open
workspace" ou "reports unavailable", e nenhum teste deste repositorio subia um
language server. Era a maior lacuna de evidencia do projeto — tudo o que o core
FALA com o servidor (`didOpen`, `didChange`, `didClose`, versao do documento, o
curto-circuito por hash) passava verde sem ninguem olhar.

Um servidor de verdade nao resolve isso: clangd e rust-analyzer sao lentos, nao
sao deterministicos, podem nem estar instalados e nao deixam ver o que
receberam. Este servidor faz o contrario das tres coisas — responde na hora,
sempre igual, e **grava tudo o que recebe**.

# Contrato

Fala LSP sobre stdio com framing `Content-Length` (a mesma moldura do
`lsp/framing.rs`). Cada mensagem recebida e gravada no arquivo passado como
PRIMEIRO ARGUMENTO, uma por linha, em JSON compacto, com `flush` imediato —
e' isso que torna a observacao livre de corrida: o teste espera a linha
aparecer, com prazo.

O log vem por argumento e nao por variavel de ambiente de proposito: `cargo
test` roda os testes em paralelo no MESMO processo, e `set_var` seria estado
global compartilhado entre eles — a classe de bug que este repositorio ja pagou
com a store de rascunhos orfa.

Responde:

    initialize                     capabilities fixas + legend de semantic tokens
    shutdown / exit                encerra
    textDocument/definition        um Location fixo no proprio arquivo
    textDocument/hover             um markdown fixo
    qualquer outro request         `result: null` (nao trava o cliente)

Notificacoes nao geram resposta — so entram no log, que e o ponto.

Com `--publica NOME` (2026-09-13, o companheiro de linguagem): a cada
`didOpen`/`didChange` publica UM diagnostico na linha 1 com a mensagem
`NOME: diagnostico falso` e responde `textDocument/codeAction` com UMA acao
`NOME: corrigir` (quickfix com `edit.documentChanges` versionado na linha 1). Com dois servidores
falsos publicando nomes diferentes para o mesmo arquivo, o teste ve se o core
FUNDE os diagnosticos e as acoes — ou se um apaga o outro.

Com `--stderr N` (2026-09-17, o stderr dos filhos): escreve N linhas
`fake-lsp stderr K` no stderr ANTES de ler qualquer coisa — como o clangd
que anuncia a versao e o `compile_commands.json` ao subir. Com `--morre`,
sai com codigo 2 logo depois, sem responder o `initialize`: e' o servidor
que morre no berco, e o que o core tem de mostrar e' o que ele disse.

Uso (sempre indireto, pelo `Core::use_language_server_command` e pelo
`Core::use_language_server_companion`):

    python3 scripts/fake_lsp_server.py /tmp/mensagens.jsonl [--publica NOME]
                                       [--stderr N] [--morre]
"""
import json
import sys

LOG = sys.argv[1] if len(sys.argv) > 1 else None
ARGS = sys.argv[2:]
PUBLICA = ARGS[ARGS.index("--publica") + 1] if "--publica" in ARGS else None
STDERR = int(ARGS[ARGS.index("--stderr") + 1]) if "--stderr" in ARGS else 0
MORRE = "--morre" in ARGS

# Legend anunciada no initialize. O core a le em
# `capabilities.semanticTokensProvider.legend.tokenTypes` e a usa para decodificar
# os tokens; fixa-la aqui torna essa leitura verificavel.
VERSOES = {}

TOKEN_TYPES = ["variable", "function", "keyword"]


def registrar(mensagem):
    """Grava a mensagem recebida no log, uma por linha, com flush imediato."""
    if not LOG:
        return
    with open(LOG, "a", encoding="utf-8") as arquivo:
        arquivo.write(json.dumps(mensagem, separators=(",", ":")) + "\n")
        arquivo.flush()


def ler_mensagem(entrada):
    """Le uma mensagem com framing Content-Length; None no fim do stream."""
    tamanho = None
    while True:
        linha = entrada.readline()
        if not linha:
            return None
        linha = linha.decode("utf-8", "replace").strip()
        if linha == "":
            break
        if linha.lower().startswith("content-length:"):
            tamanho = int(linha.split(":", 1)[1].strip())
    if tamanho is None:
        return None
    corpo = entrada.read(tamanho)
    if not corpo:
        return None
    return json.loads(corpo.decode("utf-8"))


def escrever(saida, mensagem):
    corpo = json.dumps(mensagem).encode("utf-8")
    saida.write(f"Content-Length: {len(corpo)}\r\n\r\n".encode("ascii"))
    saida.write(corpo)
    saida.flush()


def resultado(metodo, params):
    """Resposta deterministica por metodo; None quando nao ha nada a dizer."""
    if metodo == "initialize":
        return {
            "capabilities": {
                "textDocumentSync": 1,
                "definitionProvider": True,
                "hoverProvider": True,
                "semanticTokensProvider": {
                    "legend": {"tokenTypes": TOKEN_TYPES, "tokenModifiers": []},
                    "full": True,
                },
            },
            "serverInfo": {"name": "kinein-fake-lsp", "version": "1"},
        }
    if metodo == "textDocument/definition":
        uri = params.get("textDocument", {}).get("uri", "")
        # Linha 2, coluna 4 (0-based no LSP) -> 3:5 na resposta do core, que
        # converte para 1-based. O teste checa exatamente esses numeros.
        return {
            "uri": uri,
            "range": {
                "start": {"line": 2, "character": 4},
                "end": {"line": 2, "character": 9},
            },
        }
    if metodo == "textDocument/hover":
        return {"contents": {"kind": "markdown", "value": "fake hover"}}
    if metodo == "textDocument/codeAction" and PUBLICA:
        uri = params.get("textDocument", {}).get("uri", "")
        linha = {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}
        return [{
            "title": f"{PUBLICA}: corrigir",
            "kind": "quickfix",
            "edit": {"documentChanges": [{
                "textDocument": {"uri": uri, "version": VERSOES.get(uri)},
                "edits": [{"range": linha, "newText": f"# {PUBLICA}\n"}],
            }]},
        }]
    return None


def diagnostico_falso(uri):
    """O `publishDiagnostics` do modo --publica: um aviso fixo na linha 1."""
    return {
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": [{
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 3}},
                "severity": 2,
                "code": PUBLICA.upper(),
                "source": PUBLICA,
                "message": f"{PUBLICA}: diagnostico falso",
            }],
        },
    }


def main():
    for k in range(1, STDERR + 1):
        print(f"fake-lsp stderr {k}", file=sys.stderr, flush=True)
    if MORRE:
        return 2
    entrada = sys.stdin.buffer
    saida = sys.stdout.buffer
    while True:
        mensagem = ler_mensagem(entrada)
        if mensagem is None:
            return 0
        registrar(mensagem)
        metodo = mensagem.get("method")
        if metodo == "exit":
            return 0
        if metodo == "initialized":
            # Como o pyright faz: logo depois do initialized, pergunta a
            # configuracao ao cliente. A resposta do core entra no log como
            # qualquer mensagem (sem `method`, com este id) — e' o que o teste
            # da fatia 2 da cadeia Python confere.
            escrever(saida, {
                "jsonrpc": "2.0",
                "id": 9001,
                "method": "workspace/configuration",
                "params": {"items": [{"section": "python"}, {"section": "python.analysis"},
                                     {"section": "inexistente"}, {}, {"section": ""}]},
            })
            continue
        if "id" not in mensagem:
            if PUBLICA and metodo in ("textDocument/didOpen", "textDocument/didChange"):
                documento = (mensagem.get("params") or {}).get("textDocument", {})
                uri = documento.get("uri", "")
                VERSOES[uri] = documento.get("version")
                escrever(saida, diagnostico_falso(uri))
            if PUBLICA and metodo == "textDocument/didClose":
                uri = (mensagem.get("params") or {}).get("textDocument", {}).get("uri", "")
                vazio = diagnostico_falso(uri)
                vazio["params"]["diagnostics"] = []
                escrever(saida, vazio)
            continue  # notificacao: so o log importa
        if metodo == "shutdown":
            escrever(saida, {"jsonrpc": "2.0", "id": mensagem["id"], "result": None})
            continue
        escrever(saida, {
            "jsonrpc": "2.0",
            "id": mensagem["id"],
            "result": resultado(metodo, mensagem.get("params") or {}),
        })


if __name__ == "__main__":
    sys.exit(main())
