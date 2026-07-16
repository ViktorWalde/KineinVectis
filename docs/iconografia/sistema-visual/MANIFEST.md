# Kinein Vectis — Complete Icon Package

Este pacote reúne toda a entrega de iconografia em um único diretório.

## Estrutura

```text
docs/iconografia/sistema-visual/
├── 01_SPECIFICATION/
│   ├── documentação detalhada dos ícones;
│   ├── fundamentos do sistema visual;
│   ├── IDs semânticos;
│   ├── contratos para Qt/QML;
│   └── catálogo original.
│
├── 02_QML_SVG_IMPLEMENTATION/
│   ├── 129 SVGs simbólicos;
│   ├── KIcon.qml;
│   ├── IconRegistry.js;
│   ├── IconGallery.qml;
│   ├── ICON_CATALOG.json;
│   ├── CMakeLists.txt;
│   ├── launcher C++;
│   └── previews dark/light e HTML.
│
├── MANIFEST.md
└── SHA256SUMS.json
```

## Uso recomendado

```text
1. Ler 01_SPECIFICATION/docs/00_ICON_SYSTEM_FOUNDATION.md.
2. Usar os IDs de 01_SPECIFICATION/ICON_CATALOG.json.
3. Integrar 02_QML_SVG_IMPLEMENTATION/qml/KIcon.qml.
4. Adicionar os SVGs ao Qt Resource System.
5. Validar visualmente em preview/gallery.html ou na IconGallery.qml.
```

## Regra principal

```text
A interface deve referenciar IDs semânticos, como:

kv.view.project
kv.view.toolchains
kv.embedded.qemu
kv.action.open_ai_terminal

Nunca depender diretamente do caminho físico do SVG no código de negócio.
```

## Conteúdo verificado

```text
Arquivos totais: 153
SVGs: 129
Documentos Markdown: 12
```
