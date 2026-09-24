# Contribuir com a Kinein Vectis — o guia de quem chega

> Escrito em 2026-09-19. Este é o ponto de entrada para quem vai **mexer
> no código ou na documentação**, seja uma linha ou uma etapa inteira, seja
> com as próprias mãos ou com um agente de IA ao lado. A versão curta
> ("onde olhar para mudar o quê") continua em
> [`../contribuindo.md`](../contribuindo.md); este guia é o caminho
> completo.

```text
01-o-que-o-projeto-e.md        a ideia, o que ele recusa ser, e por que as regras existem
02-preparar-o-ambiente.md      instalar, compilar, abrir pelo checkout, os presets
03-o-ritual-de-uma-fatia.md    desenhar → contrato → core → ponte → UI → provar → documentar → commit
04-os-gates-que-dizem-nao.md   o verificar.sh e cada gate: o que mede, por que existe, como ler o "não"
05-com-um-agente-de-ia.md      como colaborar com Claude Code/Codex/outros sem que quebrem as regras
06-onde-mexer.md               o mapa por área (código, contrato, docs, testes)
07-fluxo-e-responsabilidades-dos-gates.md  orquestração, comunicação e dono do código de cada gate
```

## Em uma tela

A Kinein é uma IDE Linux-first para C, C++, Rust e Python. **UI em Qt/QML**
que só apresenta e recebe gestos; **core em Rust** que valida, guarda
estado e chama ferramentas maduras (clangd, rust-analyzer, CMake, Cargo,
LLDB, Git…); entre as duas, um **JSON-RPC local tipado** e versionado
(`crates/kinein-protocol`). Operação longa vira **job cancelável**;
resposta lenta sai do laço (`defer_work`/`defer_then`). A IDE não instala
nada em silêncio, nunca roda `sudo`, não tem telemetria nem IA embutida.

A regra que resume o método de trabalho, e que os gates cobram:

> **Contrato primeiro, medir antes de afirmar, um commit por fatia, e o
> contexto salvo na documentação antes do código.**

## Por onde começar, por perfil

| Você quer… | Leia | Depois |
| --- | --- | --- |
| Entender o projeto antes de opinar | [01](01-o-que-o-projeto-e.md) | [`../arquitetura/ARCHITECTURE.md`](../arquitetura/ARCHITECTURE.md) |
| Compilar e abrir a IDE | [02](02-preparar-o-ambiente.md) | [`../build/como-executar.md`](../build/como-executar.md) |
| Corrigir um bug pequeno | [06](06-onde-mexer.md) → [03](03-o-ritual-de-uma-fatia.md) §"a fatia mínima" | [04](04-os-gates-que-dizem-nao.md) |
| Implementar uma funcionalidade | [03](03-o-ritual-de-uma-fatia.md) inteiro | o roadmap da etapa (`../roadmaps/README.md`) |
| Trabalhar com um agente de IA | [05](05-com-um-agente-de-ia.md) | [03](03-o-ritual-de-uma-fatia.md) |
| Só documentação | [03](03-o-ritual-de-uma-fatia.md) §"documentação" | [`../README.md`](../README.md) (o índice) |
| Entender ou alterar os gates | [04](04-os-gates-que-dizem-nao.md) | [07](07-fluxo-e-responsabilidades-dos-gates.md) |

## O estado do projeto (para não começar do lugar errado)

O estado real, medido e datado, vive em
[`../roadmaps/40-estado-e-continuidade.md`](../roadmaps/40-estado-e-continuidade.md)
(o cabeçalho tem os números; a §7 é o diário de cada fatia). A etapa em
curso está em `../roadmaps/README.md`. **Nunca comece por um roadmap
antigo sem conferir o `40`**: ele diz o que já foi feito e o que foi
superado.
