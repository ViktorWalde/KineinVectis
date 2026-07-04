# 11 — Layout e Interações

## Layout principal

```text
┌──────────────────────────────────────────────────────────────┐
│ KW | Projeto | Branch | Run Config | Build | Run | Debug | IA │
├────┬───────────────────────────────┬─────────────────────────┤
│    │ Tabs                          │ Assistente / Inspector  │
│Bar │ Editor                        │ Docs / Outline          │
│    │                               │                         │
├────┴───────────────────────────────┴─────────────────────────┤
│ Terminal | Problems | Build | Git | Debug | Tests             │
├──────────────────────────────────────────────────────────────┤
│ Status: Git | Toolchain | LSP | Build | Encoding | Line/Col   │
└──────────────────────────────────────────────────────────────┘
```

## Sidebar esquerda

Ícones fixos:

```text
Projeto
Buscar
Git
Run
Debug
Services
Embedded
IA
Settings
```

Regra:

- clique abre/recolhe painel;
- botão direito abre opções;
- posição não muda automaticamente.

## Top Bar — Projeto

Menu:

```text
Abrir projeto
Abrir recente
Novo projeto
Fechar projeto
Adicionar projeto ao workspace
Configurações do workspace
Reindexar workspace
Executar verificação completa
```

## Top Bar — Branch

Menu:

```text
Criar branch
Trocar branch
Fetch
Pull
Push
Merge
Rebase
Abrir painel Git
```

## Top Bar — Run Config

Menu:

```text
Debug Strict
Release Strict
Sanitized Debug
Release Hardened
Editar configurações
Nova configuração
Executar testes
CMake Configure
CMake Build
CMake Clean
```

## Painel Project

Modos:

```text
Projeto
Arquivos
Estrutura
Pacotes
CMake Targets
Services
Embedded Targets
```

Direção de UX:

- o painel Project deve ter sensação limpa e confortável, próxima do Project
  View das IDEs JetBrains;
- deve parecer uma árvore de projeto da IDE, não um gerenciador de arquivos
  pesado;
- clique em diretório expande/recolhe;
- clique em arquivo seleciona/abre conforme o estágio atual da UI; no futuro,
  duplo clique ou Enter deve ser o gesto explícito de abertura;
- trocar workspace deve continuar sendo comando próprio (`Abrir projeto`,
  `Fechar projeto`, `Abrir recente`), nunca efeito colateral de navegar na
  árvore;
- ações de cabeçalho devem ser compactas, preferencialmente ícones com tooltip
  futuro;
- no MVP atual, ações compactas no cabeçalho do Project criam novo arquivo e
  nova pasta via core, usando a pasta selecionada, o pai do arquivo selecionado
  ou a raiz do workspace como alvo;
- seleção, hover e foco devem ser sutis, sem barras muito saturadas ou blocos
  pesados.

Regra visual:

- não usar cards dentro do Project;
- não colocar bordas internas em excesso;
- não criar uma estética de explorer fechado/pesado como VS Code;
- manter indentação clara e leitura rápida.

## Clique direito em arquivo

```text
Abrir
Abrir ao lado
Renomear
Mover
Duplicar
Excluir
Formatar arquivo
Analisar arquivo
Gerar teste
Explicar com IA
Ver alterações Git
Histórico do arquivo
```

## Editor — clique direito

```text
Quick Fix
Refatorar
Renomear símbolo
Extrair função
Extrair variável
Ir para definição
Encontrar usos
Mostrar documentação
Formatar seleção
Explicar seleção com IA
Gerar teste
Revisar performance
```

## Painel inferior

Tabs:

```text
Terminal
Problems
Build
Output
Git
Debug
Tests
```

## Painel direito

Tabs:

```text
Assistente KW
Inspector
Docs
Outline
```

## Settings

Seções:

```text
Geral
Aparência
Editor
Atalhos
Projetos
C++
Ambiente do Projeto
Modos do Compilador
Quality Center
Java
Python
Embedded
Backend
Git
IA
Terminal
Build
Debug
Plugins
Privacidade
```

## Privacidade padrão

```text
Telemetria: desligada
IA local: permitida
IA externa: perguntar antes de enviar contexto
Nunca enviar projeto inteiro sem confirmação
```
