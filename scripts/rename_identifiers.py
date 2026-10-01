#!/usr/bin/env python3
"""rename_identifiers.py ARQUIVO velho=novo ... — renomeia IDENTIFICADORES so' em codigo.

Ferramenta da varredura de idioma (roadmap 53 §G0). Rust, C++, QML, JS e Python.

Comentarios e literais de string ficam intactos. Em Rust/C++/QML/JS usa a
mesma mascara do gate de idioma (strip_c_like), comparando posicao a posicao;
em Python, o proprio `tokenize`: so' token NAME e' trocado (no Python 3.12+ o
nome dentro das chaves de uma f-string tambem e' NAME, e e' codigo)."""
import io, re, sys, tokenize
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_identifier_language as v

def code_mask(text: str) -> list[bool]:
    stripped = v.strip_c_like(text)
    # strip_c_like preserva o texto de codigo e troca comentario/string por
    # quebras de linha; alinhar caractere a caractere reconstruindo a mascara.
    mask, i, j = [False] * len(text), 0, 0
    while i < len(text) and j < len(stripped):
        if text[i] == stripped[j]:
            mask[i] = True; i += 1; j += 1
        else:
            i += 1
    return mask

def python_mask(text: str) -> list[bool]:
    line_starts, offset = [], 0
    for line in text.splitlines(keepends=True):
        line_starts.append(offset); offset += len(line)
    mask = [False] * len(text)
    for tok in tokenize.generate_tokens(io.StringIO(text).readline):
        if tok.type == tokenize.NAME:
            start = line_starts[tok.start[0] - 1] + tok.start[1]
            for k in range(start, start + len(tok.string)):
                mask[k] = True
    return mask

path = Path(sys.argv[1]); text = path.read_text()
pairs = [a.split('=', 1) for a in sys.argv[2:]]
if path.suffix == ".py":
    mask = python_mask(text)
elif path.suffix in (".rs", ".cpp", ".h", ".hpp", ".qml", ".js"):
    mask = code_mask(text)
else:
    sys.exit(f"erro: {path.suffix} sem mascara de codigo (shell ainda nao)")
out, last = [], 0
pattern = re.compile(r'\b(' + '|'.join(re.escape(a) for a, _ in pairs) + r')\b')
table = dict(pairs); count = 0
for m in pattern.finditer(text):
    if all(mask[k] for k in range(m.start(), m.end())):
        out.append(text[last:m.start()]); out.append(table[m.group(1)]); last = m.end(); count += 1
out.append(text[last:])
path.write_text(''.join(out))
print(f"{path}: {count} trocas")
