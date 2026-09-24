# Kinein Vectis — Icons Only

> **Revisão de 2026-09-22:** os três IDs `ai_*` ainda presentes no catálogo são
> artefatos legados, não features-alvo. Não devem aparecer na IDE nem orientar
> implementação nova; serão removidos quando a compatibilidade do catálogo
> puder ser quebrada com segurança.

Este pacote contém **somente iconografia e infraestrutura para renderizá-la**.

Não contém:

```text
layout da IDE;
Project Explorer;
editor;
Assistente;
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

é um ID legado do antigo **AI CLI Bridge**, agora cancelado. Não há ícone ou
integração de IA a implementar no produto.

Também foram adicionados:

```text
kv.view.configuration_actions
kv.view.project_health
kv.view.resources
kv.view.toolchains
kv.view.targets
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
