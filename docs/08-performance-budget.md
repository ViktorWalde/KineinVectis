# 08 — Performance Budget

## Objetivo

Kernwerk Studio deve ser rápido e leve por padrão.

## Metas iniciais

```text
Abrir janela inicial:          < 1s
Iniciar core:                  < 500ms
Ping UI ↔ Core:                < 50ms
Abrir projeto pequeno:         < 2s
UI travada por trabalho pesado: nunca
Uso de RAM em projeto pequeno: < 300 MB como meta aspiracional
```

## Princípios

1. UI nunca executa trabalho pesado.
2. Builds, LSPs, IA e busca rodam fora da thread de UI.
3. Core deve emitir eventos incrementais.
4. Logs não devem bloquear comandos.
5. File watching deve ser eficiente.
6. Cache deve acelerar abertura sem esconder erros.
7. Evitar dependências pesadas no core.
8. Carregamento preguiçoso de módulos.
9. Painéis só carregam dados quando abertos, quando possível.
10. IA nunca bloqueia editor.

## Métricas a coletar localmente

Sem telemetria externa. Apenas diagnóstico local:

```text
tempo de start
tempo de abertura de workspace
tempo de resposta IPC
número de processos externos
memória do core
memória da UI
tempo de build
tempo de indexação
```

## Painel Diagnostics

O painel deve mostrar:

```text
Core: running
UI: connected
IPC latency: 12ms
clangd: running
cmake: ready
git: ready
memory core: 42 MB
memory ui: 180 MB
```

## Regra

O projeto deve falhar em desenvolvimento se uma mudança introduzir bloqueio óbvio na UI ou acoplamento indevido entre UI e ferramenta externa.
