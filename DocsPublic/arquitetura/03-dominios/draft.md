# `draft.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Rascunhos / autosave (`draft.save` / `draft.clear`)

Rede de segurança contra perda de dado (protocolo `0.40.0`, fatia S1 de
`DocsPublic/seguranca/23`). O core persiste em **SQLite** (`.kinein/kinein.db`, WAL +
escrita atômica por transação) o buffer NÃO SALVO de arquivos sujos. Se a
UI cair (ou power loss), no próximo `workspace.open` as edições voltam.
Complementa a escrita atômica de `fs.write` (temp + `fsync` + `rename`, que
elimina o arquivo truncado/zerado em crash).

```text
draft.save { path, content } → { savedAt }   (autosave de buffer sujo)
draft.clear { path }          → { ok: true }  (save/close limpo)
workspace.open ... → { ..., drafts?: [{ path, content, savedAt }] }
```

- `path` é absoluto e confinado à raiz (como `fs.*`); a chave é o caminho
  absoluto. A UI autosalva com debounce (~1.5s) só quando o buffer está
  modificado; ao fechar a aba manda `draft.clear`.
- `fs.write` **também** limpa o rascunho do path salvo (o arquivo em disco
  passou a ser a verdade). Então rascunho **só sobrevive a um CRASH**.
- Recuperação: embutida na resposta de `workspace.open` (`drafts`), já
  FILTRADA — rascunho igual ao disco é obsoleto e apagado; só volta o que
  DIFERE. A UI abre a aba, sobrepõe o buffer e a marca MODIFICADA (o
  savedContent segue o disco: salvar/reverter continuam corretos).
- Erros: `NO_WORKSPACE` (sem workspace); `INVALID_PARAMS` (path fora da
  raiz / inexistente); `INTERNAL_ERROR` (store indisponível/falha).
