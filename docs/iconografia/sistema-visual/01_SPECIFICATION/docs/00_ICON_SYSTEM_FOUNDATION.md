# Kinein Vectis — Sistema Profissional de Iconografia

> Especificação visual e técnica para todos os ícones funcionais, abas e estados da primeira arquitetura completa do Kinein Vectis.

## 1. Objetivo

Este pacote impede que a implementação produza uma mistura de ícones retirados de bibliotecas diferentes, com pesos, tamanhos e metáforas incompatíveis.

A regra principal é:

```text
O ID semântico é permanente.
O desenho visual pode evoluir ou ser substituído por tema.
A ação nunca deve depender do caminho do SVG.
```

O sistema foi modelado com base em práticas públicas de IDEs e toolkits profissionais:

- IDs semânticos e temas de product icons;
- versões específicas para 20×20 e 16×16;
- ícones simbólicos, monocromáticos e legíveis em tamanhos pequenos;
- modifiers/badges posicionados consistentemente;
- suporte a temas, fallback e High DPI no Qt;
- não depender apenas de cor para transmitir estado.

## 2. Decisão visual

```text
Tipo: SVG simbólico próprio.
Estilo: outline técnico, compacto e confortável.
Base de Tool Window: 20×20.
Modo compacto e ações: 16×16.
Toolbar confortável: 20×20.
Ações de destaque: até 24×24.
App icon: projeto separado, full color.
```

Não usar um icon font como fonte canônica. SVGs individuais são mais fáceis de auditar, versionar, revisar e substituir por tema.

## 3. Grid e construção

### 3.1 Grid 20×20

```text
Canvas: 20×20 unidades.
Safe area: x/y entre 2 e 18.
Stroke principal: 1.5.
Stroke pesado excepcional: 2.
Cap: square ou round conforme família, nunca misturados no mesmo ícone.
Join: round para formas orgânicas controladas; miter/round para formas técnicas.
```

### 3.2 Grid 16×16

```text
Canvas: 16×16.
Safe area: x/y entre 1.5 e 14.5.
Stroke principal: 1.25–1.5.
Detalhe mínimo: 1.5 px visual.
Espaço negativo mínimo: 1 px.
```

Os masters de 20 e 16 devem ser revisados separadamente. Não confiar apenas em reduzir automaticamente o SVG de 20 para 16.

## 4. Cor

Paleta funcional do Kinein Vectis:

```text
icon.foreground.default   #B9B3A5
icon.foreground.strong    #EAE6E1
icon.foreground.muted     #8F8A7C
icon.accent               #FFBB00
icon.accent.dim           #6E5C01
icon.success              token semântico do tema
icon.warning              token semântico do tema
icon.error                token semântico do tema
icon.info                 token semântico do tema
```

Regras:

```text
1. Ícone neutro por padrão.
2. Accent apenas para selected/active ou ação principal.
3. Error/warning/success precisam também de forma, badge ou texto.
4. Sem gradientes.
5. Sem sombras.
6. Sem glow.
7. Sem preenchimentos multicoloridos em barras densas.
```

## 5. Estados

Cada ícone acionável deve suportar:

```text
default
hover
pressed
selected
focused
disabled
busy
success
warning
error
```

O SVG normalmente permanece o mesmo; a camada Qt/QML altera tokens de cor, fundo, opacidade, badge e animação.

## 6. Modifiers e badges

```text
Tamanho: 6–9 px no master 20×20.
Posição padrão: canto inferior direito.
Distância da forma base: 1–2 px.
Usar canto alternativo quando encobrir a metáfora.
No máximo dois modifiers simultâneos.
```

Modifiers oficiais:

```text
add
remove
check
error
warning
lock
remote
running
paused
sync
dirty
count
```

## 7. Acessibilidade

```text
- Todo botão somente com ícone precisa de tooltip.
- Ícones ambíguos em sidebars e view switchers podem ter label.
- Focus ring pertence ao componente, não ao SVG.
- Estado não pode ser comunicado somente por cor.
- Hit target mínimo recomendado: 28×28 no modo compacto e 32×32 no confortável.
- Tooltip deve descrever a ação, não a aparência.
```

## 8. Naming

IDs:

```text
kv.view.*
kv.action.*
kv.editor.*
kv.project.*
kv.search.*
kv.git.*
kv.build.*
kv.run.*
kv.debug.*
kv.test.*
kv.quality.*
kv.embedded.*
kv.status.*
kv.file.*
kv.modifier.*
```

Arquivos:

```text
resources/icons/20/view/project.svg
resources/icons/16/view/project.svg
resources/icons/20-dark/view/project.svg   # somente se realmente necessário
```

A preferência é colorização por token, evitando duplicar dark/light.

## 9. Contrato com Qt/QML

Cada ícone deve ser resolvido pelo `IconRegistry`, nunca por URL hardcoded.

```qml
KIconButton {
    actionId: "workspace.project.toggle"
    iconId: "kv.view.project"
    tooltip: qsTr("Projeto — arquivos, estrutura, targets e módulos")
}
```

O registry resolve:

```text
ID semântico
→ tema de produto ativo
→ variante de tamanho
→ SVG padrão
→ fallback de theme icon do sistema quando permitido
```

## 10. Critérios de aprovação de um SVG

```text
[ ] reconhecível sem tooltip em contexto comum;
[ ] distinguível dos vizinhos;
[ ] legível em 16 e 20;
[ ] não depende apenas de cor;
[ ] sem detalhes menores que o limite;
[ ] alinhado opticamente;
[ ] strokes consistentes;
[ ] área de clique separada do glyph;
[ ] dark/light testados;
[ ] 100%, 125%, 150%, 200% testados;
[ ] nome e ID estáveis;
[ ] licença/proveniência registradas;
```
