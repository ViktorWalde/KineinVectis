# 00 — Visão do Produto

## Nome

**Kernwerk Studio**

## Frase curta

Uma IDE open source, Linux-first, rígida por padrão e visualmente plug and play para engenharia de software moderna.

## Público inicial

O primeiro usuário é o próprio autor do projeto:

- usa Linux/CachyOS/KDE;
- gosta da filosofia open source;
- está acostumado ao ecossistema JetBrains;
- quer conforto visual para longas sessões;
- quer memória muscular preservada;
- trabalha/estuda C++, Java, Python, backend, integração OT/TI, Linux embarcado e robótica/simulação;
- prefere uma IDE completa e visual, com menos configuração manual do que VS Code.

## Filosofia

Kernwerk Studio deve ser:

- familiar como JetBrains;
- modular como VS Code;
- rápido como editores modernos;
- auditável como software open source;
- rígido como CI profissional;
- confortável para uso diário;
- visualmente plug and play;
- sem telemetria obrigatória;
- IA opcional e controlada.

## O que a IDE não é

Kernwerk Studio não é:

- um compilador;
- um parser universal;
- um novo debugger;
- um novo build system;
- uma cópia de código ou assets da JetBrains;
- um clone literal de identidade visual proprietária.

Ela pode ser visualmente familiar, mas deve ter identidade própria.

## Pilares

1. **Visual plug and play**
   - Novo projeto deve gerar estrutura, build, lint, format, testes e docs.
   - Configurações complexas devem ter UI.
   - Linhas de comando devem existir, mas não serem obrigatórias para o fluxo principal.

2. **Strict by default**
   - Projetos novos começam no modo mais rígido.
   - Warnings devem virar erros quando apropriado.
   - Sanitizers, linters, formatadores e testes devem estar integrados.
   - Relaxar regras deve ser escolha explícita.

3. **Linux-first**
   - CachyOS/Arch como ambiente principal inicial.
   - Boa integração com KDE/Wayland.
   - Configs em diretórios Linux padrão.
   - Empacotamento futuro via PKGBUILD/AUR, AppImage e Flatpak.

4. **Core em Rust**
   - Segurança, concorrência e robustez.
   - Processo separado da UI.
   - Testável sem interface.
   - Sem `unsafe` por padrão.

5. **UI Qt/QML**
   - Interface moderna e eficiente.
   - Painéis previsíveis.
   - Memória muscular inspirada em JetBrains.
   - Tema escuro confortável por padrão.

6. **Ferramentas existentes**
   - clangd, jdtls, pyright, CMake, Ninja, Gradle, Maven, uv, Git, GDB/LLDB etc.
   - A IDE integra, organiza e visualiza essas ferramentas.
