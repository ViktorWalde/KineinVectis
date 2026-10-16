# Migrar de Rust + Qt para C#/.NET 10 — a hipótese

> **Classe: PESQUISA** ([`README.md`](README.md)). Não é alvo, não é decisão,
> e nada aqui se implementa sem um gesto datado do autor no
> [`roadmaps/40`](../roadmaps/40-estado-e-continuidade.md) §5.
>
> **Origem:** o autor, em 2026-10-16, ao encerrar a 0.3.9: "pode deixar
> anotado na documentação uma possível migração de Rust + Qt para ecossistema
> C#/.NET10". Os fatos externos abaixo foram consultados nessa data (fontes
> no §7); os números do projeto foram medidos no checkout no mesmo dia.

## 1. A pergunta

Valeria trocar a pilha atual (o core em Rust, a UI em Qt/QML com uma camada
C++) por C#/.NET 10, inteira ou em parte, **sem perder o que a IDE já prova**:
latência de tecla, primeiro quadro, memória, Linux de primeira classe (X11 e
Wayland) e Windows 11?

## 2. O que falsearia a hipótese

Esta seção vem antes das ideias, de propósito. Cada linha é uma régua que já
existe no projeto; um protótipo que não a cumpra derruba o caminho dele.

```text
régua (roadmaps/21)            medido hoje (2026-10-16, 40.7 §7.300)   falha se
tecla→quadro 16 ms (p95 20)    8,1 ms (p95 9,4)                        o editor do protótipo passar da régua
primeiro quadro 400 ms         321 ms (offscreen)                      a UI nova passar da régua
RSS da UI 200 MB               150 MB                                  idem
RSS do core 60 MB              47 MB                                   o core em .NET (com ou sem AOT) passar
Linux X11 + Wayland            Qt 6.10 nos dois                        o toolkit só servir pelo XWayland
Windows 11                     porte até a W6b4 (60 §3.3)              idem
licença                        Qt LGPL, Rust MIT/Apache, cargo-deny    exigir licença comercial
```

E o que não tem equivalente conhecido ainda falseia o caminho que dependa
dele, até ser medido: o terminal (PTY/ConPTY e o emulador VT próprio, com
reflow, 16.3), o Tree-sitter incremental (33,2 ms por tecla hoje), o
`kinein-sys` (Job Object, o único `unsafe` do projeto) e o cliente DAP.

## 3. O tamanho do que existe (medido em 2026-10-16)

```text
Rust (crates/)          117.439 linhas, das quais 22.789 de testes; 1.158 testes do core
QML (ui/qml)             57.965 linhas em 414 arquivos
C++ (ui/src)              9.787 linhas (CoreClient, ponte, harness de medida)
harness QML              15.337 linhas, 148 harnesses
scripts (py/sh)          14.796 linhas (os gates, os aceites no Xvfb)
protocolo                0.172.0: 187 métodos e 60 eventos (arquitetura/03)
```

O que torna uma troca **em partes** pensável: o core é um processo separado,
e a UI só o conhece pelo protocolo JSON-RPC versionado e com gate
(`verificar-fiacao-ipc.sh`). Trocar um lado sem tocar o outro é possível por
construção; os 1.158 testes do core são Rust, mas descrevem o comportamento
que um core novo teria de repetir.

## 4. Os caminhos, cada um com o que falta medir

**A. O core em C#/.NET 10, a UI como está.** A fronteira é o protocolo.
Ganha drivers de banco maduros e oficiais (Npgsql, o driver oficial do
MongoDB, `Microsoft.Data.Sqlite`, `System.Data.Odbc`), o que conversa com os
adiados do Banco (59 §2.3), e um cliente LSP pronto
(`OmniSharp.Extensions.LanguageClient`). Arrisca a régua de memória do core
(60 MB) e o tempo de partida: medir com Native AOT. Uma forma incremental,
também hipótese: um core C# que atende os domínios já portados e repassa os
outros ao core Rust como processo filho, até o último sair.

**B. A camada C++ em C#, com o QML intacto.** O Qt Group publicou o Qt
Bridge for C# em beta pública em 2026-05-20 (NuGet 0.3.0-beta, .NET 8 ou
mais novo, LGPL-3.0-only ou comercial): objetos C# como elementos QML,
propriedades, sinais e coleções. O ganho é pequeno aqui (a camada C++ tem
9,8 mil linhas) e o risco é a beta; serve se o A acontecer e a ponte C++
virar o gargalo.

**C. A UI em Avalonia (MIT).** A 12.1 trouxe um backend Wayland nativo, ainda
experimental e fora da detecção automática (opt-in, `UseWayland()`).
Significa reescrever 58 mil linhas de QML e os 148 harnesses. Cruza com a
0.6 (o motor do editor, roadmaps/21 §M5.4): o editor da UI nova seria a
mesma decisão.

**D. Tudo junto (A + C).** A reescrita inteira; o maior risco, e nenhuma
régua do §2 medida antes.

## 5. O que se mede primeiro, se o autor um dia quiser seguir

Nada daqui tem ordem sem decisão. Se houver: um protótipo do caminho A num
domínio pequeno (`workspace.open` + `fs.*`), com Native AOT, medindo RSS,
partida e tamanho contra o core Rust na mesma máquina; e, só depois, o
editor (a tecla→quadro) se o C entrar na conversa.

## 6. O que fica fora

Datas, versão de destino e qualquer mudança na fila: a 0.4 segue em Rust e
Qt (40 §4).

## 7. Fontes (consultadas em 2026-10-16)

- .NET 10: LTS, lançado em 2025-11-11, suporte até 2028-11-14 —
  [política de suporte](https://dotnet.microsoft.com/en-us/platform/support/policy/dotnet-core).
- Qt Bridge for C#: [anúncio da beta pública](https://www.qt.io/blog/qt-bridges-public-beta-for-csharp),
  [documentação](https://doc-snapshots.qt.io/qtbridge-csharp/),
  [NuGet](https://www.nuget.org/packages/QtGroup.Qt.Bridge.CSharp.linux-x64).
- Avalonia: [12.1 e o Wayland nativo](https://avaloniaui.net/blog/release-12-1),
  [o backend Wayland](https://avaloniaui.net/blog/bringing-wayland-support-to-avalonia).
- Cliente LSP em .NET: [OmniSharp.Extensions.LanguageClient](https://feed.nuget.org/packages/OmniSharp.Extensions.LanguageClient).
