# Kinein Vectis — Contrato de Iconografia para o Frontend Qt/QML

## 1. Regra

O frontend não pode referenciar diretamente:

```text
qrc:/icons/project.svg
../../../assets/debug.png
:/dark/search.svg
```

Deve referenciar:

```text
kv.view.project
kv.view.debug
kv.action.search_everywhere
```

## 2. Estruturas previstas

```text
IconRegistry
IconTheme
IconDescriptor
IconState
IconBadge
KIcon
KIconButton
KToolWindowButton
KStatusIcon
KFileIcon
```

## 3. Descriptor

```json
{
  "id": "kv.view.project",
  "variants": {
    "16": "qrc:/icons/16/view/project.svg",
    "20": "qrc:/icons/20/view/project.svg"
  },
  "symbolic": true,
  "recolorable": true,
  "defaultTooltip": "Projeto — arquivos, estrutura, targets e módulos"
}
```

## 4. Exemplo QML futuro

```qml
KToolWindowButton {
    id: projectButton

    actionId: "workspace.project.toggle"
    iconId: "kv.view.project"
    text: qsTr("Projeto")
    shortcutText: "Alt+1"
    checked: WorkspaceUi.projectVisible
    badgeCount: ProjectModel.pendingChanges
}
```

## 5. Separação de responsabilidades

```text
SVG:
geometria.

IconRegistry:
resolução do ID.

Theme:
cor e variantes.

Action Registry:
função, enabled, checked, shortcut e tooltip.

Componente QML:
layout, hover, focus, badge, animação e hit target.
```

## 6. Theme switching

Trocar o tema não deve alterar IDs de ações.

```text
kv.view.project
→ Kinein Default
→ High Contrast
→ System/Breeze fallback opcional
→ tema de produto futuro
```

## 7. Recursos obrigatórios antes do frontend

```text
1. Gerar SVGs 16/20.
2. Validar viewBox.
3. Gerar qrc.
4. Criar IconRegistry.
5. Criar ActionRegistry.
6. Criar tokens do tema.
7. Criar KIcon e KIconButton.
8. Criar storybook/gallery QML interno.
9. Testar escala 100–200%.
10. Testar light, dark e high contrast.
```

## 8. Galeria interna

O frontend deve conter uma tela de desenvolvimento:

```text
Settings > Developer > Icon Gallery
```

Ela deve mostrar:

```text
ID
glyph em 16/20/24
default/hover/selected/disabled
dark/light/high contrast
tooltip
badge
ação vinculada
arquivo de origem
```

## 9. Próxima etapa

O protótipo visual do frontend deve consumir o catálogo JSON e implementar primeiro:

```text
top bar;
activity bar;
Project Explorer;
editor tabs;
painel direito;
painel inferior;
status bar;
Embedded & Remote;
Quality Center.
```
