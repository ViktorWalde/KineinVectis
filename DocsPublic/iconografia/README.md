# Iconografia da Kinein Vectis

Índice único dos pacotes visuais. Antes de 2026-07-16 eles viviam em três
lugares diferentes (dois na raiz do repositório) sem declarar qual era fonte de
verdade de cada família; agora estão agrupados aqui.

## Fonte de verdade por família

| Família | Pacote | Contrato |
| --- | --- | --- |
| Ícones da IDE (views, ações, Git, build/run/debug, status) | [`sistema-visual/`](sistema-visual/) | `sistema-visual/01_SPECIFICATION/KINEIN_VECTIS_ICON_SYSTEM_MASTER.md` |
| Ícones de tipos de arquivo especiais da árvore | [`icones-de-arquivo/`](icones-de-arquivo/) | `icones-de-arquivo/FILE_ICON_MAPPINGS.json` |
| Ícones individuais da árvore de projetos | [`icones-da-arvore/`](icones-da-arvore/) | `icones-da-arvore/kinein_vectis_tree_icons_individual/` |

A **spec normativa** do sistema visual é
[`../especificacoes/sistema-visual-e-icones.md`](../especificacoes/sistema-visual-e-icones.md).
Os pacotes aqui são os assets e o detalhamento que a implementam; onde houver
divergência, a spec vale sobre o pacote.

## sistema-visual/

Pacote principal: identidade visual, grade, paleta e o conjunto completo de
ícones da aplicação.

```text
01_SPECIFICATION/
    KINEIN_VECTIS_ICON_SYSTEM_MASTER.md   fonte da especificação
    DocsPublic/00_ICON_SYSTEM_FOUNDATION.md     fundação (grade, traço, tokens)
    DocsPublic/01..07_*.md                      por área (views, ações, Git, build…)
    DocsPublic/08_QML_FRONTEND_ICON_CONTRACT.md contrato de consumo no QML
02_QML_SVG_IMPLEMENTATION/                SVGs + exemplo de integração
MANIFEST.md · FILE_LIST.txt · SHA256SUMS.json
```

## icones-de-arquivo/

Ícones de tipos de arquivo especiais da árvore de projetos (`cmake-lists`,
`project-config`, `sql`, `docker-yaml`) em light/dark 16/20/24 px.

O contrato de resolução vive em `FILE_ICON_MAPPINGS.json`, com precedência:

```text
nome exato → padrão → caminho → extensão composta → extensão → genérico
```

Regras que não podem regredir:

- honrar a regra de segurança do `.env`: não vazar valores nem segredos;
- não dar o ícone Docker a YAML genérico.

Detalhamento em `DocsPublic/01_FILE_SEMANTICS.md` (semântica),
`DocsPublic/02_ICON_VISIBILITY_AND_OPTICAL_SIZING.md` (sizing ótico),
`DocsPublic/03_QT_QML_IMPLEMENTATION.md` (implementação) e
`DocsPublic/04_VALIDATION_CHECKLIST.md` (checklist). Licença dos assets em
`LICENSE-NOTICE.md`: originais MIT OR Apache-2.0, sem logos oficiais.

## icones-da-arvore/

Ícones individuais da árvore. Os cinco SVGs fornecidos pelo autor foram
integrados sem alterar seus bytes — conferir antes de reprocessar.

## Onde os ícones em uso realmente vivem

Estes pacotes são **design e origem**. Os assets que a aplicação compila estão
em `ui/assets/icons`, e o consumo no QML segue o contrato da parte 08. Mover ou
renomear um pacote aqui não afeta o build; alterar `ui/assets` afeta.
