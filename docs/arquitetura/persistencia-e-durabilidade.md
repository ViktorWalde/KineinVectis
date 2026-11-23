# Persistência e durabilidade

Dono: onde cada arquivo próprio da IDE mora, como ele é gravado e quanto texto
uma queda pode levar. Fecha as lições L04 e L13 ([lições](../licoes-da-0.3.md)).

## Três classes, mais a árvore de build

Do 64 §5.9 (64:603-672), decisão DP-08 do Viktor (64:943).

| Classe | Onde | O quê |
| --- | --- | --- |
| Configuração do projeto, compartilhável | `<raiz>/.kinein/` | o que um colega deve receber pelo git |
| Estado da pessoa no projeto | `$XDG_STATE_HOME/kinein-vectis/projetos/<id>/` | sessão, rascunhos, escolhas locais |
| Cache do projeto | `$XDG_CACHE_HOME/kinein-vectis/projetos/<id>/` | derivado e recriável |
| Árvore de build | `<raiz>/.kinein/build/` | saída do build, com `.gitignore` próprio |

- O `<id>` é derivado do caminho canônico da raiz, e o estado segue o projeto
  renomeado no mesmo volume pela identidade do diretório (64:620-635).
- Abrir uma pasta sem confiança não grava nela.
- Cada arquivo próprio tem dono, classe e versão de esquema num catálogo único.

## Regras de gravação

| Regra | Origem |
| --- | --- |
| Gravação atômica: arquivo temporário na mesma pasta, `fsync`, troca de nome, `fsync` da pasta | RNF-DUR-02 |
| Gravar com o conteúdo esperado: se o disco mudou desde a leitura, a gravação é recusada com o conflito | `fs.write` da 0.3 (65:83-84) |
| Não seguir link, não abrir o que não é arquivo regular | R-DADOS-02 do 63 |
| Não gravar por cima de arquivo que não se entende (versão de esquema mais nova): avisar e preservar | R-DADOS-03 do 63 |
| Migração do `.kinein` da 0.3: copiar, reler e só então remover | 64:660-672 |

## Durabilidade do texto digitado

- **O problema da 0.3:** debounce sem prazo máximo num `Timer` de QML. Quem
  digita sem pausa de 1,5 s não tem nada gravado, e a perda não tem teto (INF-26).
- **A regra:** a política de durabilidade é do core. Toda mudança do espelho
  do buffer agenda a gravação do rascunho com **prazo máximo**; a pausa da
  digitação pode adiantar a gravação, nunca adiar além do prazo.
- **O limite:** RNF-DUR-01, com 2 s propostos
  ([decisões](../decisoes-pendentes.md)).
- **O caminho:** o rascunho sai do espelho que o core mantém pelo
  `nvim_buf_attach`, sem o swap do Neovim (decisão 3 do 65 §21, 65:895-896).
  O arquivo do projeto é gravado pelo core, pelo `BufWriteCmd` (65:892-894).
- **Aceite:** `kill -9` na UI, no core e no `nvim` depois de digitação longa
  sem pausa, com a perda medida, e um mutante sem prazo máximo que precisa
  reprovar.
