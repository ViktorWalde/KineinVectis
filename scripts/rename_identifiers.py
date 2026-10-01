#!/usr/bin/env python3
"""rename_identifiers.py ARQUIVO velho=novo ... — renomeia IDENTIFICADORES so' em codigo.

Ferramenta da varredura de idioma (roadmap 53 §G0). Rust, C++, QML e JS.

Comentarios (// e /* */) e literais de string ficam intactos: usa a mesma
mascara do gate de idioma (strip_c_like), comparando posicao a posicao."""
import re, sys
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

path = Path(sys.argv[1]); text = path.read_text()
pairs = [a.split('=', 1) for a in sys.argv[2:]]
mask = code_mask(text)
out, last = [], 0
pattern = re.compile(r'\b(' + '|'.join(re.escape(a) for a, _ in pairs) + r')\b')
table = dict(pairs); count = 0
for m in pattern.finditer(text):
    if all(mask[k] for k in range(m.start(), m.end())):
        out.append(text[last:m.start()]); out.append(table[m.group(1)]); last = m.end(); count += 1
out.append(text[last:])
path.write_text(''.join(out))
print(f"{path}: {count} trocas")
