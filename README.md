# Kinein Vectis

Kinein Vectis é uma IDE open source, Linux-first e rígida por padrão para C,
C++ e Rust. A interface nativa em Qt/QML conversa por JSON-RPC local com um
core em Rust e orquestra ferramentas maduras, em vez de reimplementar
compiladores, servidores de linguagem, build systems ou debugadores.

O projeto está em desenvolvimento ativo. A versão de teste atual é a `0.1.0`,
distribuída como AppImage para Linux x86_64.

## Princípios

- execução local, sem telemetria;
- UI e lógica de negócio separadas;
- qualidade estrita por padrão;
- CMake/Cargo, clangd/rust-analyzer e LLDB integrados como ferramentas
  externas;
- nenhuma instalação silenciosa de toolchain;
- nenhuma chamada de IA ou envio de código sem ação explícita do usuário.

## O que já funciona

- abertura e criação de projetos C++/CMake e Rust/Cargo, com workspaces
  recentes fixáveis na Start Screen e no menu Arquivo;
- explorer, editor com múltiplas abas e recuperação de rascunhos;
- Tree-sitter incremental, autocomplete, diagnósticos, navegação, rename e
  quick fixes com preview;
- build, testes, análise, execução e debug;
- terminal PTY com múltiplas sessões;
- Git diário: status, diff, stage, commit, branches, pull, push e stash;
- configurações, Project Health e KV Context para Claude/Codex CLI já
  instalados pelo usuário.

## Testar ou instalar

O AppImage inclui a interface, o `kinein-core`, o runtime Qt/QML, plugins,
licenças e o manual. Rust e Qt não precisam estar instalados para abrir a IDE;
as ferramentas usadas pelos projetos continuam opcionais e externas.

Para saber exatamente quais arquivos enviar, verificar o SHA-256, executar,
atualizar com segurança ou gerar uma nova versão, consulte o
[Tutorial.md](Tutorial.md).

Depois de abrir a IDE, o [MANUAL.md](MANUAL.md) explica os recursos, fluxos e
atalhos. O manual trata somente do uso da Kinein; instalação e distribuição
ficam no tutorial.

## Arquitetura

```text
Qt/QML Frontend
       ↕ JSON-RPC local
Rust Core
       ↕
CMake · Cargo · clangd · rust-analyzer · LLDB · Git · outras ferramentas
```

A UI apresenta e recebe ações. O core valida, mantém estado e chama as
ferramentas externas. Operações longas são jobs assíncronos e canceláveis para
não bloquear a interface.

## Plataforma

- Linux x86_64;
- baseline do AppImage: glibc 2.36, equivalente ao Debian 12 ou posterior;
- desktop Linux com pilha gráfica e fontes normais;
- Windows ainda não é suportado.

O smoke do pacote já passou no host Arch/CachyOS e em um runtime Debian 12
mínimo, sem Qt, Rust, CMake ou compiladores instalados.

## Privacidade do repositório e distribuição do código

O repositório de desenvolvimento permanece privado. Se o código for entregue
a terceiros, será usada uma cópia sanitizada, sem o histórico Git privado, que
contém o código do projeto e, entre arquivos Markdown, somente:

- `README.md`;
- `MANUAL.md`;
- `Tutorial.md`.

Documentos internos de IA, contexto, planejamento, prompts, roadmaps e specs de
trabalho não fazem parte dessa cópia. A visibilidade do repositório-fonte não
deve ser alterada para realizar uma distribuição; um eventual espelho começa
com histórico próprio.

## Licença

Kinein Vectis é disponibilizado sob licença dupla MIT ou Apache-2.0. Consulte
`LICENSE-MIT.txt` e `LICENSE-APACHE-2.0.txt`.
