# 12 — Política de IA

## Objetivo

Kernwerk Studio deve integrar IA sem quebrar privacidade, controle e filosofia open source.

## Providers previstos

```text
Ollama
GPT CLI
Claude CLI
OpenAI API
Anthropic API
OpenRouter
```

## Modos

```text
Local only
Ask before external context
External allowed for selected files
```

Padrão:

```text
Ask before external context
```

## Regras

1. Nunca enviar projeto inteiro sem confirmação.
2. Mostrar arquivos/trechos que serão usados como contexto.
3. Permitir IA local sem confirmação extra.
4. Permitir desativar IA completamente.
5. Logs de prompts devem ser locais e opcionais.
6. Chaves/API tokens nunca devem ir para logs.
7. IA não executa comando destrutivo sem confirmação.

## Ações de IA

```text
ai.explainFile
ai.explainSelection
ai.reviewDiff
ai.generateCommitMessage
ai.fixBuildError
ai.generateTests
ai.createDocs
ai.suggestRefactor
ai.explainArchitecture
```

## Painel Assistente KW

Deve mostrar:

```text
Contexto atual
Arquivos incluídos
Provider ativo
Política de privacidade
Ações rápidas
Chat
```
