# Kernwerk Studio — Crash Recovery e Autosave

> Este documento define como o Kernwerk deve proteger o trabalho do usuário contra crash, fechamento inesperado, perda de energia e conflitos de arquivo.

---

## 1. Objetivo

Uma IDE profissional deve evitar perda de trabalho.

O Kernwerk deve proteger:

```text
arquivos editados;
layout;
abas abertas;
estado do workspace;
tarefas recentes;
logs;
relatórios.
```

---

## 2. Sessão

Salvar sessão local:

```text
arquivos abertos;
cursor;
scroll;
layout;
painéis;
workspace ativo;
run config ativa.
```

Arquivo:

```text
.kernwerk/ui-state.json
```

ou cache local:

```text
~/.local/share/kernwerk-studio/sessions/
```

---

## 3. Autosave

Modos:

```text
off;
on focus lost;
after delay;
before build/run/debug;
manual only.
```

Padrão recomendado:

```text
off ou before build/run/debug
```

Para uso profissional:

```text
salvar antes de build/run deve ser configurável.
```

---

## 4. Recovery buffer

Para arquivos não salvos, manter cópia temporária.

Diretório:

```text
~/.local/share/kernwerk-studio/recovery/
```

Regra:

```text
Recovery não substitui arquivo original sem confirmação.
```

---

## 5. Ao reiniciar após crash

Mostrar:

```text
Sessão anterior não foi encerrada corretamente.

Arquivos recuperáveis:
- src/main.cpp
- crates/core/src/lib.rs

Ações:
[Restaurar todos]
[Comparar]
[Descartar]
```

---

## 6. Alteração externa

Se arquivo mudou no disco enquanto estava aberto:

```text
Arquivo alterado fora do Kernwerk.

Ações:
[Recarregar do disco]
[Comparar]
[Manter minha versão]
[Salvar como]
```

Nunca sobrescrever silenciosamente.

---

## 7. Salvamento atômico

Quando possível:

```text
escrever arquivo temporário;
fsync opcional;
renomear atomicamente;
preservar permissões.
```

---

## 8. Tarefas em andamento

Se IDE fechar com tarefas rodando:

```text
Build em execução.
[Cancelar e sair]
[Esperar finalizar]
[Forçar saída]
```

Para operações perigosas:

```text
flash firmware;
deploy remoto;
migração;
escrita de arquivos;
```

exigir confirmação extra.

---

## 9. Logs de crash

Logs locais:

```text
~/.cache/kernwerk-studio/logs/crash/
```

Não enviar automaticamente.

---

## 10. Critérios de aceite

MVP pronto quando:

```text
restaura abas;
detecta dirty files;
não perde conteúdo não salvo em fechamento normal;
detecta alteração externa;
não sobrescreve conflito sem aviso.
```

---

## 11. Decisão final

O Kernwerk deve ser conservador com dados do usuário.

É melhor perguntar demais em caso de conflito do que perder trabalho.
