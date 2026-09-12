# Kinein Vectis — Sistema Visual da IDE

> **Parte 1 — Iconografia da IDE**  
> Documento de direção visual para os ícones internos, identidade KV e linguagem de interface da IDE **Kinein Vectis**.

---

## 0. Escopo deste documento

Este documento cobre **apenas a primeira parte do sistema visual**: a iconografia da IDE.

Ele não define ainda, em profundidade:

- layout completo da tela principal;
- viewport OpenGL;
- dashboards de telemetria;
- painéis avançados de target embarcado;
- telas de onboarding;
- documentação visual do site/landing page.

Essas partes devem ser tratadas em documentos separados para evitar mistura de conceitos e preservar qualidade.

O foco aqui é construir uma linguagem visual consistente para:

- ícone principal da aplicação;
- monograma **KV**;
- ícones de navegação lateral;
- ícones da barra superior;
- ícones de ações comuns da IDE;
- ícones de build, CMake, toolchain e compilador;
- ícones de status, problemas, Git, terminal e assistente contextual;
- regras de exportação, estados visuais e uso em Qt/QML.

---

## 1. Identidade do produto

### 1.1 Nome oficial

```text
Kinein Vectis
```

### 1.2 Nome curto de uso diário

```text
Kinein
```

### 1.3 Sigla visual

```text
KV
```

### 1.4 Posicionamento técnico

```text
Kinein Vectis é uma IDE Linux-first para C, C++ e Rust,
focada em toolchains, CMake, sistemas embarcados, Linux embarcado,
software embarcado, ambientes de build e debug.
```

### 1.5 Frase-guia do design

```text
Familiar como uma IDE profissional, própria como uma ferramenta de engenharia.
```

### 1.6 Sensação desejada

A interface e os ícones devem passar a sensação de:

- controle;
- clareza;
- precisão;
- conforto visual;
- baixa ansiedade;
- engenharia séria;
- ambiente profissional;
- configuração guiada, não caótica;
- foco no código, não no sofrimento com toolchain.

A estética deve ser **JetBrains-like na experiência psicológica**, mas **não deve copiar visualmente os ícones, marcas, formas ou composições da JetBrains**.

O objetivo é trazer a mesma sensação de produto maduro: familiaridade, densidade controlada, atalhos previsíveis, ações bem posicionadas e ícones claros.

---

## 2. Filosofia visual da iconografia

### 2.1 Ideia central

A iconografia da Kinein deve nascer de quatro famílias formais:

```text
1. Vetores
2. Circuitos
3. Chevrons de código
4. Geometria técnica / malhas 3D
```

Essas quatro famílias comunicam diretamente o foco da IDE:

- C, C++ e Rust;
- CMake;
- compiladores;
- sistemas embarcados;
- Linux embarcado;
- hardware targets;
- engenharia de software de baixo e alto nível.

### 2.2 O que evitar

Evitar ícones clichês ou genéricos demais:

- engrenagem grande como símbolo dominante;
- chave inglesa como símbolo principal de ferramenta;
- martelo para build;
- foguete literal para deploy;
- cérebro para IA;
- robô para assistente;
- placas eletrônicas excessivamente detalhadas;
- ícones cartoon;
- neon exagerado;
- símbolos com texto dentro;
- formas que pareçam cópia direta de IDEs existentes.

### 2.3 O que priorizar

Priorizar:

- formas geométricas simples;
- leitura clara em 16px, 20px e 24px;
- silhueta forte;
- poucos detalhes;
- cantos levemente arredondados;
- traços técnicos;
- uso controlado do âmbar;
- hierarquia visual;
- consistência entre famílias de ícones.

---

## 3. Sistema de cores dos ícones

### 3.1 Paleta base

```text
Background principal:        #0B0D10
Surface / painel:            #111418
Surface elevada:             #171B21
Editor background:           #0F1216
Linha atual:                 #1A1F26
Borda sutil:                 #2A2F37

Texto principal:             #E7E2D8
Texto secundário:            #A9A39A
Texto fraco:                 #6F737A

Ícone padrão:                #A9A39A
Ícone hover:                 #E7E2D8
Ícone ativo:                 #FFB000
Ícone ativo forte:           #FFC93D
Ícone desabilitado:          #4E535A

Sucesso:                     #7CCF6A
Aviso:                       #FFB000
Erro:                        #D45F5F
Info técnica:                #5C8DFF
Roxo orbital/acento:         #8A5CFF
```

### 3.2 Regra de uso do âmbar

O âmbar é a identidade visual da Kinein, mas deve ser usado com disciplina.

Usar âmbar para:

- botão principal de Run;
- aba ativa;
- item selecionado;
- estado de build em andamento;
- foco de componente;
- ícone ativo na lateral;
- pequenos pontos de energia em ícones técnicos.

Não usar âmbar para:

- todos os ícones ao mesmo tempo;
- texto longo;
- bordas de todos os painéis;
- fundo de grandes áreas;
- alertas que não sejam de ação/atenção.

A regra psicológica é:

```text
O fundo acalma. O âmbar guia.
```

---

## 4. Grid e geometria dos ícones internos

### 4.1 Tamanho padrão

A maioria dos ícones internos deve ser desenhada em:

```text
Grid: 24x24
Área útil: 20x20
Margem óptica: 2px
Stroke padrão: 1.75px ou 2px
Cantos: 1.5px a 2px de raio visual
```

### 4.2 Tamanhos obrigatórios

Todo ícone importante deve funcionar em:

```text
16x16  — status bar, tabs, badges pequenos
20x20  — menus, listas, árvore de projeto
24x24  — toolbar, sidebar, botões principais
32x32  — cards, quick actions, onboarding
```

### 4.3 Estilo de traço

Regras:

- usar stroke consistente;
- evitar preenchimento pesado;
- manter ícones majoritariamente lineares;
- usar preenchimento apenas quando o estado ativo exigir;
- não depender de sombras para leitura;
- não depender de gradiente em ícones pequenos.

### 4.4 Forma base

Os ícones devem ter geometria:

- angular;
- precisa;
- levemente arredondada;
- vetorial;
- técnica;
- sem parecer agressiva.

A Kinein não deve parecer militar ou hacker neon. Deve parecer **ferramenta profissional de engenharia**.

---

## 5. Estados dos ícones

Todo ícone interativo deve ter pelo menos estes estados:

### 5.1 Default

```text
Cor: #A9A39A
Fundo: transparente
```

Uso normal. Não chama atenção demais.

### 5.2 Hover

```text
Cor: #E7E2D8
Fundo: #171B21
Borda: opcional #2A2F37
```

Deve indicar que o item é clicável, sem efeito exagerado.

### 5.3 Active / Selected

```text
Cor: #FFB000
Fundo: rgba(255, 176, 0, 0.08)
Borda lateral ou inferior: #FFB000
```

Usado em:

- tool window ativa;
- aba ativa;
- botão selecionado;
- modo atual.

### 5.4 Disabled

```text
Cor: #4E535A
Fundo: transparente
```

Não usar opacidade baixa demais, pois pode ficar ilegível em monitores com contraste ruim.

### 5.5 Warning

```text
Cor: #FFB000
```

Usado para avisos de build, CMake, toolchain parcialmente configurada.

### 5.6 Error

```text
Cor: #D45F5F
```

Usado para erro de compilação, falha de toolchain, falha de conexão com target, teste quebrado.

### 5.7 Success

```text
Cor: #7CCF6A
```

Usado para build concluído, testes passando, target conectado, toolchain válida.

---

## 6. Ícone principal da aplicação

### 6.1 Objetivo

O ícone principal da Kinein Vectis deve ser reconhecível em:

- launcher Linux;
- dock;
- menu de aplicações;
- alt-tab;
- README;
- GitHub;
- splash screen;
- janela About;
- instaladores futuros.

### 6.2 Forma geral

```text
Formato: quadrado com bordas arredondadas
Fundo externo: transparente real
Fundo interno: grafite escuro técnico
Monograma: KV grande e central
```

### 6.3 Monograma KV

O monograma deve ser o elemento dominante.

#### K

Representa:

- Kinein;
- kernel;
- kinematics;
- conhecimento técnico;
- base estrutural da IDE.

Visual:

```text
K metálico/prata
robusto
angular
com profundidade leve
sem excesso de reflexo
```

#### V

Representa:

- Vectis;
- vetor;
- velocidade;
- visualização;
- alavanca;
- direção;
- execução.

Visual:

```text
V âmbar/dourado industrial
parece vetor em aceleração
pode lembrar braço de alavanca
pode ter ponta levemente projetada
continua claramente legível como V
```

### 6.4 Fundo interno do app icon

O fundo interno pode conter, de forma sutil:

- linhas de circuito;
- trilhas de PCB;
- malha 3D wireframe;
- trajetória orbital;
- vetores de força;
- pequena integral ou símbolo matemático discreto;
- pontos luminosos controlados;
- textura técnica suave.

Não deve conter:

- texto grande;
- fórmulas dominantes;
- elementos que disputem com o KV;
- símbolos genéricos demais;
- outro logotipo pequeno embaixo;
- checkerboard renderizado;
- sombra externa de mockup.

### 6.5 Versões do app icon

Criar quatro versões oficiais:

#### 1. Principal detalhada

Uso:

- README;
- site;
- splash;
- ícone 512/1024.

#### 2. Principal simplificada

Uso:

- launcher;
- dock;
- 256/128.

#### 3. Small icon

Uso:

- 64/48/32/16;
- favicon;
- status pequeno.

Características:

```text
menos textura
menos brilho
KV mais grosso
fundo mais limpo
sem fórmula visível
```

#### 4. Monocromática

Uso:

- tray;
- about minimalista;
- documentação;
- temas de alto contraste.

---

## 7. Ícones de navegação lateral

A barra lateral esquerda deve ter ícones previsíveis, inspirados no padrão mental de IDEs profissionais.

Ordem recomendada:

```text
1. Project
2. Search
3. Git
4. Build
5. Debug
6. Targets
7. Tools
8. Extensions
```

Mesmo que algumas áreas sejam implementadas depois, a linguagem visual já deve ser planejada.

---

### 7.1 Project

#### Função

Abrir árvore de arquivos e estrutura do projeto.

#### Conceito visual

Pasta angular técnica.

#### Forma

```text
pasta minimalista
canto superior levemente inclinado
linha interna curta sugerindo raiz do projeto
```

#### Evitar

- pasta colorida estilo sistema operacional;
- pasta muito arredondada;
- ícone infantil.

---

### 7.2 Search

#### Função

Busca global no projeto.

#### Conceito visual

Lupa minimalista com cabo angular.

#### Forma

```text
círculo incompleto
cabo em 45 graus
pequeno corte no círculo para manter estilo técnico
```

#### Estado ativo

Lupa âmbar com fundo sutil.

---

### 7.3 Git

#### Função

Controle de versão.

#### Conceito visual

Grafo de commits com três nós.

#### Forma

```text
linha vertical quebrada
três pontos conectados
um branch lateral curto
```

#### Evitar

- copiar logo do Git;
- usar símbolo muito detalhado.

---

### 7.4 Build

#### Função

Painel de compilação e tarefas de build.

#### Conceito visual

Chevron + base de compilação.

#### Forma

```text
chevron >
linha inferior curta
pequena abertura entre segmentos
```

Transmite código avançando para artefato compilado.

---

### 7.5 Debug

#### Função

Depuração local/remota.

#### Conceito visual

Execução controlada.

#### Forma

```text
chevron pequeno
ponto central de breakpoint
duas marcas laterais discretas
```

#### Cores

- padrão cinza;
- ativo âmbar;
- breakpoint pode usar vermelho discreto em contexto específico.

---

### 7.6 Targets

#### Função

Gerenciar targets, arquiteturas, placas, toolchains remotas.

#### Conceito visual

Chip técnico angular.

#### Forma

```text
quadrado com canto cortado
pinos em dois lados apenas
pequeno vetor diagonal interno
```

#### Evitar

- microchip cheio de pinos em todos os lados;
- excesso de detalhe.

---


### 7.8 Tools

#### Função

Ferramentas gerais da IDE.

#### Conceito visual

Nó técnico abstrato.

#### Forma

```text
três linhas conectadas
três pequenos nós
um nó central destacado
```

#### Evitar

Engrenagem grande como símbolo principal.

---

### 7.9 Extensions

#### Função

Plugins, extensões e integrações.

#### Conceito visual

Módulos encaixáveis.

#### Forma

```text
três blocos pequenos conectados
um bloco deslocado sugerindo plug-in
```

#### Evitar

Peça de quebra-cabeça genérica.

---

## 8. Ícones da barra superior

A barra superior deve priorizar ações diárias de C/C++/Rust:

```text
Target selector
Build profile
Configure
Build
Run
Debug
Stop
Test
Flash futuramente
Search
Settings
```

---

### 8.1 Configure

#### Função

Rodar configuração do projeto:

- CMake configure;
- detectar preset;
- validar toolchain;
- preparar diretório de build.

#### Conceito visual

Delta aberto.

#### Forma

```text
triângulo incompleto
um vértice aberto
pequeno nó no canto inferior
linha interna âmbar quando ativo
```

#### Evitar

- engrenagem;
- símbolo do CMake copiado;
- triângulo cheio demais.

---

### 8.2 Build

#### Função

Compilar projeto.

#### Conceito visual

Chevron com base.

#### Forma

```text
>_
```

Mas desenhado como ícone vetorial, não texto.

#### Estado em progresso

Pode receber:

```text
pequeno arco parcial
barra de progresso inferior
ponto âmbar pulsante
```

---

### 8.3 Run

#### Função

Executar o binário ou target atual.

#### Conceito visual

Chevron dinâmico.

#### Forma

```text
>
```

Com três características próprias:

- segmentos levemente separados;
- ponta central projetada;
- sensação de vetor/velocidade.

#### Cor

Run é uma das poucas ações que pode usar âmbar por padrão.

---

### 8.4 Debug

#### Função

Executar com depurador.

#### Conceito visual

Chevron + ponto de controle.

#### Forma

```text
chevron externo
ponto interno
pequena linha de inspeção
```

#### Evitar

Inseto/bug genérico como ícone principal.

---

### 8.5 Stop

#### Função

Parar execução, build ou debug.

#### Conceito visual

Quadrado técnico quebrado.

#### Forma

```text
quadrado pequeno
um canto cortado
stroke vermelho discreto
```

#### Evitar

Quadrado vermelho saturado demais.

---

### 8.6 Test

#### Função

Rodar testes.

#### Conceito visual

Check vetorial.

#### Forma

```text
check angular
pequeno ponto de validação
linha inferior curta
```

#### Cores

- padrão cinza;
- sucesso verde;
- falha vermelho;
- em execução âmbar.

---

### 8.7 Clean

#### Função

Limpar artefatos de build.

#### Conceito visual

Bloco sendo removido.

#### Forma

```text
pequeno bloco angular
linha diagonal de corte
partícula mínima saindo
```

#### Evitar

Vassoura.

---

### 8.8 Rebuild

#### Função

Limpar e compilar novamente.

#### Conceito visual

Chevron + ciclo parcial.

#### Forma

```text
chevron build
arco parcial ao redor
sem seta circular genérica
```

---

## 9. Ícones de CMake, toolchain e compilador

Essa é uma das famílias mais importantes da Kinein.

A IDE deve comunicar visualmente:

```text
O ambiente está entendido.
A toolchain está sob controle.
O programador não está sozinho configurando CMake.
```

---

### 9.1 CMake

#### Função

Representar área/painel CMake.

#### Conceito visual

Delta técnico aberto.

#### Forma

```text
três linhas formando triângulo incompleto
nó em um vértice
linha interna diagonal
```

#### Estados

```text
CMake válido: verde discreto
CMake configurando: âmbar
CMake com erro: vermelho
CMake ausente: cinza desabilitado
```

---

### 9.2 Toolchain

#### Função

Representar cadeia de ferramentas.

#### Conceito visual

Pipeline de nós.

#### Forma

```text
três círculos conectados horizontalmente
primeiro = compiler
segundo = linker
terceiro = artifact/target
```

#### Variações

- toolchain local;
- cross-toolchain;
- remote toolchain;
- toolchain incompleta.

---

### 9.3 Compiler

#### Função

Representar compilador.

#### Conceito visual

Código entra, binário sai.

#### Forma

```text
retângulo técnico
duas linhas pequenas à esquerda
chevron de saída à direita
```

---

### 9.4 Linker

#### Função

Representar etapa de linkedição.

#### Conceito visual

Dois módulos conectando em um artefato.

#### Forma

```text
dois blocos pequenos à esquerda
uma linha convergente
um bloco final à direita
```

---

### 9.5 Presets

#### Função

Representar presets de build.

#### Conceito visual

Placas empilhadas.

#### Forma

```text
três camadas finas
uma camada ativa em âmbar
```

---

### 9.6 Environment

#### Função

Representar ambiente:

- PATH;
- SDK;
- sysroot;
- variáveis;
- toolchain file.

#### Conceito visual

Caixa aberta com vetor entrando.

#### Forma

```text
caixa angular
seta/vetor discreto entrando
pequeno ponto de validação
```

---

### 9.7 Artifact

#### Função

Representar binário gerado:

- executável;
- firmware;
- biblioteca;
- objeto;
- pacote.

#### Conceito visual

Bloco final.

#### Forma

```text
hexágono ou bloco angular pequeno
linha de saída
ponto inferior indicando arquivo final
```

---

## 10. Ícones de editor

Esses ícones devem ser menos chamativos que os de ação.

### 10.1 File

```text
folha angular
canto dobrado mínimo
sem preenchimento pesado
```

### 10.2 C file

```text
ícone de arquivo + pequeno C
```

Mas o `C` deve ser discreto. Não usar texto grande.

### 10.3 C++ file

```text
ícone de arquivo + dois pequenos marcadores ++
```

### 10.4 Rust file

```text
ícone de arquivo + pequeno nó hexagonal
```

Evitar copiar diretamente o logo do Rust.

### 10.5 Header file

```text
arquivo + linha superior dupla
```

### 10.6 Modified file

```text
pequeno ponto âmbar na aba
```

### 10.7 Read-only file

```text
cadeado minimalista
```

### 10.8 Generated file

```text
arquivo + pequeno padrão de nós
```

### 10.9 Excluded file

```text
arquivo cinza + linha diagonal discreta
```

---

## 11. Ícones de status bar

A status bar deve ser discreta e informativa.

### 11.1 Branch Git

```text
ramo pequeno com dois nós
```

### 11.2 Warning count

```text
triângulo aberto pequeno
```

### 11.3 Error count

```text
losango ou círculo com corte
```

### 11.4 Build success

```text
check vetorial pequeno
```

### 11.5 Build running

```text
ponto âmbar + arco parcial
```

### 11.6 Target connected

```text
chip pequeno + ponto verde
```

### 11.7 Target disconnected

```text
chip pequeno + traço cinza
```

### 11.8 Debug active

```text
ponto de breakpoint + linha curta
```

### 11.9 Encoding / line endings

Não precisa de ícones fortes. Texto é suficiente.

Exemplo:

```text
UTF-8 | LF | C++ | clang++ 17 | Debug
```

---

## 12. Ícones de problemas e diagnósticos

### 12.1 Error

```text
círculo angular ou losango
pequeno X interno
vermelho discreto
```

### 12.2 Warning

```text
triângulo aberto
âmbar
```

### 12.3 Info

```text
círculo pequeno com ponto/linha
azul técnico
```

### 12.4 Hint

```text
pequeno vetor luminoso
cinza/âmbar suave
```

### 12.5 Quick fix

```text
raio mínimo ou vetor curto
```

Evitar lâmpada genérica demais. Se usar lâmpada, estilizar como forma técnica angular.

---

## 13. Ícones de Git

### 13.1 Commit

```text
ponto em linha
```

### 13.2 Branch

```text
linha bifurcada com dois nós
```

### 13.3 Merge

```text
duas linhas convergindo
```

### 13.4 Rebase

```text
linha principal + deslocamento angular
```

### 13.5 Pull

```text
vetor entrando em nó local
```

### 13.6 Push

```text
vetor saindo de nó local
```

### 13.7 Modified

```text
M âmbar pode ser texto pequeno em listas
ícone alternativo: ponto âmbar
```

### 13.8 Added

```text
pequeno + verde
```

### 13.9 Deleted

```text
traço vermelho discreto
```

---

## 14. Ícones do Assistente

O painel de assistência contextual não deve parecer chatbot genérico.

Nome recomendado:

```text
Assistente
```

### 14.1 Context

```text
nó central com três linhas ao redor
```

Representa contexto do arquivo atual.

### 14.2 Explain

```text
linhas de texto + pequeno vetor
```

Representa explicação direcionada.

### 14.3 Fix

```text
vetor corrigindo linha quebrada
```

Representa correção técnica.

### 14.4 Toolchain

```text
três nós conectados
```

Representa análise de ambiente.

### 14.5 Docs

```text
folha dupla angular
```

Representa documentação.

### 14.6 Suggestion card

Ícone pequeno recomendado:

```text
quadrado com canto cortado + vetor interno
```

### 14.7 Apply suggestion

```text
check vetorial âmbar
```

---

## 15. Ícones de terminal e saída

### 15.1 Terminal

```text
retângulo angular
chevron >
linha cursor
```

### 15.2 Output

```text
linhas horizontais + ponto de status
```

### 15.3 Build log

```text
linhas empilhadas + chevron pequeno
```

### 15.4 CMake log

```text
delta aberto + linhas
```

### 15.5 Debug console

```text
ponto de debug + cursor
```

---

## 16. Ícones de configuração

Mesmo evitando engrenagem como símbolo dominante, a IDE ainda precisa de ícones para configurações.

### 16.1 Settings geral

Pode usar engrenagem **apenas se for minimalista e secundária**.

Melhor alternativa:

```text
painel de controles com três sliders técnicos
```

### 16.2 Keymap

```text
tecla angular + ponto
```

### 16.3 Theme

```text
círculo dividido sutil
```

### 16.4 Plugins

```text
módulos encaixáveis
```

### 16.5 Project settings

```text
pasta + slider pequeno
```

### 16.6 Toolchain settings

```text
três nós + slider
```

---

## 17. Pacote inicial de ícones prioritários

Para o MVP visual, não tentar criar 100 ícones de uma vez.

Priorizar estes:

### 17.1 Prioridade 1 — Essenciais da IDE

```text
app-icon-kv
brand-kv-mark
project
search
git
build
debug
run
stop
test
terminal
problems
settings
file
folder
modified
error
warning
success
```

### 17.2 Prioridade 2 — C/C++/Rust e CMake

```text
cmake
configure
toolchain
compiler
linker
preset
environment
artifact
c-file
cpp-file
header-file
rust-file
```

### 17.3 Prioridade 3 — Sistemas e targets

```text
target-board
remote-target
serial-monitor
flash
qemu
linux-target
cross-compile
```


## 18. Nomenclatura dos arquivos

Usar nomes sem maiúsculas, com hífen, agrupados por domínio.

Estrutura sugerida:

```text
assets/
  icons/
    brand/
      kv-mark.svg
      app-icon.svg
      app-icon-small.svg
      app-icon-mono.svg

    actions/
      run.svg
      build.svg
      rebuild.svg
      clean.svg
      configure.svg
      debug.svg
      stop.svg
      test.svg

    navigation/
      project.svg
      search.svg
      git.svg
      build.svg
      debug.svg
      targets.svg
      tools.svg
      extensions.svg

    editor/
      file.svg
      folder.svg
      c-file.svg
      cpp-file.svg
      header-file.svg
      rust-file.svg
      modified.svg
      readonly.svg
      generated.svg
      excluded.svg

    toolchain/
      cmake.svg
      toolchain.svg
      compiler.svg
      linker.svg
      presets.svg
      environment.svg
      artifact.svg

    status/
      error.svg
      warning.svg
      info.svg
      success.svg
      running.svg
      disconnected.svg
      connected.svg

    vcs/
      branch.svg
      commit.svg
      merge.svg
      rebase.svg
      pull.svg
      push.svg
      added.svg
      deleted.svg
      modified.svg

    context/
      context.svg
      explain.svg
      fix.svg
      docs.svg
      suggestion.svg
      apply.svg
```

---

## 19. Regras para SVG

### 19.1 Requisitos técnicos

Cada SVG deve:

- usar `viewBox="0 0 24 24"` para ícones internos;
- evitar largura/altura fixa quando possível;
- usar `currentColor` para stroke/fill;
- não embutir cor fixa exceto em versões especiais;
- não usar filtros pesados;
- não usar blur em ícones pequenos;
- não depender de gradientes para leitura;
- ter paths limpos;
- evitar metadados exportados desnecessários.

### 19.2 Exemplo de estilo SVG ideal

```xml
<svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path
    d="M8 5L16 12L8 19"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round" />
</svg>
```

### 19.3 Ícones multicoloridos

Evitar multicolorido em ícones pequenos.

Quando necessário:

- usar uma versão mono para UI;
- usar versão colorida apenas em telas de boas-vindas, cards e About.

---

## 20. Integração em Qt/QML

### 20.1 Componente base de ícone

Criar um componente central para todos os ícones.

Exemplo conceitual:

```qml
KIcon {
    name: "run"
    size: 24
    state: "active"
    accessibleName: "Run current target"
}
```

### 20.2 Propriedades recomendadas

```text
name
size
colorRole
state
active
disabled
warning
error
success
accessibleName
tooltip
```

### 20.3 Componente de botão de ícone

```qml
KIconButton {
    iconName: "build"
    tooltip: "Build project"
    shortcut: "Ctrl+F9"
    onClicked: buildController.buildCurrentTarget()
}
```

### 20.4 Regras de acessibilidade

Todo botão com ícone deve ter:

- tooltip;
- nome acessível;
- estado visual claro;
- área clicável mínima de 32x32;
- foco por teclado;
- contraste suficiente.

---

## 21. Comportamento visual dos ícones na IDE

### 21.1 Sidebar

```text
Ícone padrão: cinza
Ícone ativo: âmbar
Fundo ativo: âmbar com baixa opacidade
Indicador ativo: barra lateral âmbar de 2px
```

### 21.2 Toolbar

```text
Run pode ser âmbar sempre
Debug pode ser verde/cinza conforme estado
Stop só aparece forte quando há processo ativo
Build mostra progresso quando executando
```

### 21.3 Tabs

```text
Arquivo modificado: ponto âmbar
Arquivo com erro: sublinhado vermelho discreto
Arquivo com warning: sublinhado âmbar discreto
Aba ativa: linha inferior âmbar
```

### 21.4 Project tree

```text
Pastas: cinza
Arquivos C/C++/Rust: ícones discretos
Arquivo atual: texto claro + fundo sutil
Modificado: marcador âmbar
Erro: badge vermelho pequeno
```

### 21.5 Status bar

```text
Ícones pequenos, sem brilho
Somente status importante recebe cor
Evitar poluição visual
```

---

## 22. Direção específica para o chevron dinâmico

O chevron é um elemento-chave da identidade visual.

### 22.1 Forma-base

```text
>
```

Mas deve parecer:

- vetor de força;
- avanço de código;
- execução;
- direção;
- energia controlada.

### 22.2 Aplicações

```text
Run: chevron puro
Build: chevron + base
Debug: chevron + ponto
Test: chevron + check
Flash: chevron entrando em chip
Deploy: chevron saindo para target
Next: chevron simples
```

### 22.3 Regras

- não fazer o chevron parecer botão de play comum;
- manter pontas técnicas;
- usar separação interna de segmentos em ícones grandes;
- simplificar em 16px.

---

## 23. Direção específica para o nó de circuito

O nó de circuito é a base para ícones de ferramenta, Git, toolchain e contexto.

### 23.1 Forma-base

```text
linha + ponto + bifurcação
```

### 23.2 Aplicações

```text
Git
Toolchain
Context
Remote target
Dependency graph
Build pipeline
```

### 23.3 Regras

- usar poucos nós;
- evitar parecer diagrama complexo;
- manter legível em 16px;
- usar pontos pequenos e consistentes.

---

## 24. Direção específica para malha 3D/cubo aberto

Essa linguagem deve ser usada com cuidado.

### 24.1 Aplicações principais

```text
QEMU
Emulation
OpenGL
Mesh
3D View
```

### 24.2 Forma-base

```text
cubo wireframe incompleto
uma quina aberta
um ponto de luz em vértice
```

### 24.3 Regras

- não usar como ícone de tudo;
- não transformar a IDE padrão em tela de visualização;
- manter essa família para recursos específicos.

---

## 25. Direção específica para símbolos matemáticos

Símbolos matemáticos devem existir como acento, não como ruído.

### 25.1 Aplicações

```text
Plot
Numerical analysis
Solver
```

### 25.2 Símbolos permitidos

- integral discreta;
- vetor;
- curva;
- ponto de massa;
- eixo cartesiano;
- arco orbital;
- malha técnica.

### 25.3 Regras

- não colocar fórmulas longas em ícones pequenos;
- evitar `E=mc²` como símbolo dominante por ser genérico/clichê;
- usar integrais e vetores de forma abstrata;
- priorizar legibilidade.

---

## 26. Checklist de qualidade de cada ícone

Antes de aprovar um ícone, verificar:

```text
[ ] Funciona em 24px?
[ ] Funciona em 16px?
[ ] A silhueta é reconhecível?
[ ] Não depende de brilho/sombra?
[ ] Usa currentColor?
[ ] Tem versão ativa?
[ ] Tem versão disabled?
[ ] Não copia ícones de outra IDE?
[ ] Combina com os outros ícones?
[ ] Parece técnico sem parecer agressivo?
[ ] O significado fica claro com tooltip?
[ ] O ícone não tem detalhe excessivo?
```

---

## 27. Prompt para gerar pacote inicial de ícones

Use este prompt quando for gerar referências visuais, não como especificação final de SVG.

```text
Criar uma folha de ícones vetoriais para uma IDE chamada Kinein Vectis, nome curto Kinein, sigla KV. A IDE é Linux-first e focada em C, C++ e Rust, CMake, toolchains, compiladores, debug e sistemas embarcados. Os ícones devem ser lineares, técnicos, minimalistas, angulares, com cantos levemente arredondados, desenhados em grid 24x24, stroke consistente, sem preenchimento excessivo.

Estilo visual: profissional, confortável, JetBrains-like na sensação de maturidade e clareza, mas completamente original. Usar fundo grafite escuro, ícones em cinza claro e versões ativas em âmbar industrial. A linguagem visual deve usar chevrons de código, vetores, nós de circuito, pequenos chips, deltas abertos e malhas técnicas discretas.

Gerar ícones para: Project, Search, Git, Build, Debug, Targets, Tools, Extensions, Run, Configure, Rebuild, Stop, Test, CMake, Toolchain, Compiler, Linker, Presets, Environment, Terminal, Problems, Error, Warning, Success, Context, Explain, Fix, Docs.

Não usar engrenagens grandes, chaves inglesas, foguetes, robôs, cartoons, neon exagerado ou símbolos copiados de IDEs existentes.
```

---

## 28. Prompt para gerar app icon KV final

```text
Criar um app icon profissional para uma IDE chamada Kinein Vectis, nome curto Kinein, sigla KV. O ícone deve ter formato quadrado com bordas arredondadas, fundo externo transparente real e nenhum elemento fora do quadrado. Dentro do ícone, criar um fundo escuro grafite com estética de engenharia, C/C++/Rust, CMake, toolchains, sistemas embarcados, Linux embarcado.

O fundo interno deve conter detalhes sutis de circuitos, trilhas de PCB, malha 3D wireframe, vetores de força, trajetória orbital e integrais discretas, sem poluir e sem competir com o monograma.

No centro, criar um monograma grande KV, geométrico, angular e vetorial. O K deve ser metálico/prata, robusto e técnico. O V deve ser âmbar/dourado industrial, como uma alavanca/vetor em aceleração. O V pode se estender como seta vetorial ou braço de força, mas ainda deve ser claramente a letra V.

Estilo premium, moderno, com presença visual de ícone profissional de IDE, mas completamente original. Alto contraste, legível em tamanhos pequenos, sem texto, sem logotipo pequeno embaixo, sem mockup, sem fundo branco, sem checkerboard renderizado.
```

---

## 29. Próximos documentos recomendados

Depois desta parte de iconografia, criar documentos separados:

```text
01_VISUAL_SYSTEM_ICONS.md              este documento
02_IDE_LAYOUT_CORE.md                  layout principal da IDE
03_EDITOR_EXPERIENCE.md                editor, tabs, breadcrumbs, gutter, diagnóstico
04_CMAKE_TOOLCHAIN_UX.md               fluxo visual para CMake/toolchain/compilador
05_EMBEDDED_TARGETS_UX.md              targets, flash, serial, remote debug
07_KV_CONTEXT_ASSISTANT_UX.md          painel contextual/assistente
08_THEME_TOKENS_QML.md                 tokens, componentes e implementação Qt/QML
09_BRANDING_APP_ICON_SPLASH.md         marca, splash, about, README, site
```

---

## 30. Decisão final desta parte

A iconografia da Kinein Vectis deve ser construída como um sistema próprio:

```text
KV como marca principal.
Chevron como linguagem de ação.
Nó de circuito como linguagem de fluxo e toolchain.
Delta aberto como linguagem de CMake/configuração.
Chip angular como linguagem de target embarcado.
Âmbar como guia visual, não como decoração excessiva.
```

O resultado esperado é uma IDE que pareça:

```text
profissional,
familiar,
confortável,
técnica,
visualmenta polida,
e voltada para engenheiros que querem focar no código, não no caos do ambiente.
```

