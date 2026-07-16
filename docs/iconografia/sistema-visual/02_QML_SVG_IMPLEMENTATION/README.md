# Kinein Vectis — Icons Only

Este pacote contém **somente iconografia e infraestrutura para renderizá-la**.

Não contém:

```text
layout da IDE;
Project Explorer;
editor;
KV Context;
painéis de build;
terminal;
status bar;
mockup de produto.
```

## Conteúdo

```text
icons/                  129 SVGs simbólicos próprios
qml/KIcon.qml           renderizador recolorível
qml/IconRegistry.js     ID semântico → recurso SVG
qml/IconGallery.qml     galeria técnica isolada
qml/Main.qml            launcher da galeria
ICON_CATALOG.json       catálogo consumível por IA/Core/UI
CMakeLists.txt
main.cpp
```

## Decisões corrigidas

```text
kv.view.ai_terminal
```

representa o **AI CLI Bridge externo**. Não há ícone nem implementação de chat de IA embutido.

Também foram adicionados:

```text
kv.view.configuration_actions
kv.view.project_health
kv.view.resources
kv.view.toolchains
kv.view.targets
kv.view.simulation
```

Não foram adicionados como views centrais:

```text
Database
Marketplace público
chat interno de IA
```

porque não pertencem ao escopo congelado do MVP.

## Executar a galeria

```bash
cmake -S . -B build -G Ninja
cmake --build build
./build/kinein-icons
```

## Uso

```qml
KIcon {
    iconId: "kv.view.project"
    iconSize: 20
    color: active ? "#FFB000" : "#AAA39A"
}
```
