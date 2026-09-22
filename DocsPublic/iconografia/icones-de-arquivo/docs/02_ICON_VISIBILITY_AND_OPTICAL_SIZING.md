# Visibilidade, dimensionamento óptico e densidade da árvore

## 1. Problema

Reduzir uma ilustração detalhada de 512 ou 1024 px diretamente para 16 px causa:

- desaparecimento de linhas;
- mistura entre cores adjacentes;
- excesso de área transparente;
- metáfora ilegível;
- bordas borradas por escalonamento fracionário.

A correção não consiste apenas em aumentar a imagem. O procedimento profissional
é criar recursos condicionados ao tamanho final.

## 2. Padrões observados em IDEs e plataformas

### Recursos dedicados por modo

A documentação da IntelliJ Platform registra ícones dedicados de 20×20 para a
New UI e 16×16 no Compact Mode, além de variantes claras e escuras. O princípio
aplicado ao Kinein é manter recursos distintos por densidade, em vez de reduzir
um único desenho complexo.

### Tamanho tradicional de hierarquia

O Visual Studio registra 16×16 como tamanho padrão de ícones usados em comandos
e hierarquias. A mesma documentação enfatiza clareza, simplificação e contexto.

### SVG e associação semântica

O VS Code recomenda SVG para file icon themes e separa a definição visual da
associação por nome ou tipo de arquivo. O Kinein adota o mesmo princípio por
meio de IDs semânticos e `FILE_ICON_MAPPINGS.json`.

### Correspondência exata de tamanho

A orientação de iconografia da Microsoft informa que fornecer vários tamanhos
reduz o escalonamento e aumenta a chance de uma correspondência pixel-perfect.
O pacote de estudo mantém amostras nesses tamanhos; a produção usa geometria
vetorial em grade lógica 24x24, conferida nos alvos de 16, 20 e 24 px.

### Qt e renderização vetorial

Qt Quick pode carregar SVG por `Image`. Para recursos estáticos exibidos sempre
no mesmo tamanho, a documentação Qt recomenda considerar rasterização
prévia/cache, pois o SVG é rasterizado antes de virar textura. `VectorImage` ou
`svgtoqml` preservam nitidez quando o recurso precisa ser escalado.

## 3. Política Kinein

### Modos

```text
Compacto
Ícone: 16×16
Altura da linha: 20
Uso: grande quantidade de arquivos e telas menores.

Padrão
Ícone: 20×20
Altura da linha: 24
Uso: configuração recomendada.

Confortável
Ícone: 24×24
Altura da linha: 28
Uso: HiDPI, acessibilidade visual e telas grandes.
```

A árvore deve permitir troca de densidade sem reiniciar a IDE.

### Ocupação óptica

O emblema principal ocupa aproximadamente 82–90% da área útil. A margem
transparente não deve consumir grande parte do canvas.

A ocupação é óptica, não matemática. Elementos estreitos podem ultrapassar o
limite nominal, enquanto formas pesadas podem precisar de mais respiro.

### Detalhe por tamanho

```text
16 px:
uma metáfora principal;
um elemento secundário no máximo;
sem texto;
sem sombra;
sem linhas decorativas.

20 px:
metáfora principal;
um detalhe funcional;
duas ou três camadas planas.

24 px:
pequenos detalhes adicionais;
ainda sem texto ou microtipografia.
```

### Geometria

- posicionamento em coordenadas inteiras ou meias coordenadas coerentes com a
  largura do traço;
- contorno mínimo visível;
- cantos simples;
- nenhuma linha abaixo da espessura útil do alvo;
- nenhuma transformação fracionária aplicada pelo delegate.

### Cor e contraste

- neutro de documento para formar a família;
- uma cor dominante por domínio;
- âmbar apenas como assinatura;
- contraste validado em fundo claro, escuro e selecionado;
- não depender exclusivamente da cor para distinguir estados.

### Seleção

A seleção deve alterar o fundo da linha, não destruir as cores do ícone.

Não aplicar uma única tintura sobre ícones de arquivo multicoloridos. Tintura
dinâmica é mais apropriada para product icons monocromáticos.

### Badges

- um badge por ícone;
- 5–6 px no master de 16;
- 6–7 px no master de 20;
- 7–8 px no master de 24;
- canto inferior direito;
- não cobrir a metáfora principal.

## 4. Por que 20 px é o padrão recomendado

16 px continua disponível para densidade compacta, mas os ícones aprovados têm
mais personalidade visual do que glifos monocromáticos simples.

A exibição em 20 px oferece:

- silhueta maior;
- cores reconhecíveis;
- espaço para dobra do arquivo;
- um detalhe secundário;
- linha de 24 px ainda eficiente para navegação.

A decisão de usar 20 px é específica do Kinein e não uma exigência universal.

## 5. Fontes técnicas

- IntelliJ Platform Plugin SDK — Working with Icons:
  https://plugins.jetbrains.com/docs/intellij/icons.html
- Visual Studio — Images and Icons:
  https://learn.microsoft.com/visualstudio/extensibility/ux-guidelines/images-and-icons-for-visual-studio
- VS Code — File Icon Theme:
  https://code.visualstudio.com/api/extension-guides/file-icon-theme
- Windows icon construction:
  https://learn.microsoft.com/windows/apps/design/iconography/app-icon-construction
- Qt Quick Image:
  https://doc.qt.io/qt-6/qml-qtquick-image.html
- Qt 2D Graphics:
  https://doc.qt.io/qt-6/topics-graphics2d.html
