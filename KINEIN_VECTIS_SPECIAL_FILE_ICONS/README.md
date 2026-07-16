# Kinein Vectis — Special File Icon Pack

Pacote de ícones de arquivos especiais para a árvore de projetos do Kinein
Vectis.

## Conteúdo

```text
icons/
├── light/
│   ├── 16/
│   ├── 20/
│   └── 24/
├── dark/
│   ├── 16/
│   ├── 20/
│   └── 24/
└── source/64/

docs/
├── 01_FILE_SEMANTICS.md
├── 02_ICON_VISIBILITY_AND_OPTICAL_SIZING.md
├── 03_QT_QML_IMPLEMENTATION.md
└── 04_VALIDATION_CHECKLIST.md

ICON_CATALOG.json
FILE_ICON_MAPPINGS.json
```

## Ícones

- `cmakelists.svg`: `CMakeLists.txt`;
- `project-config.svg`: configurações de ambiente e ferramentas;
- `sql.svg`: arquivos SQL;
- `docker-yaml.svg`: Docker Compose em YAML.

Os SVGs de produção são interpretações vetoriais simplificadas dos conceitos
visuais aprovados. Eles não são miniaturas reduzidas das ilustrações grandes:
cada tamanho foi condicionado para permanecer legível na árvore.

## Modos recomendados

```text
Compacto:    ícone 16 px, linha 20 px
Padrão:      ícone 20 px, linha 24 px
Confortável: ícone 24 px, linha 28 px
```

O modo padrão recomendado para o Kinein é 20 px. O modo compacto preserva a
densidade tradicional das IDEs, enquanto o confortável atende telas HiDPI e
usuários que priorizam legibilidade.

## Licença

Assets originais para o projeto Kinein Vectis, destinados à mesma expressão de
licença do projeto:

```text
MIT OR Apache-2.0
```

Os ícones são metáforas próprias. O ícone Docker YAML usa um motivo original de
container/whale e não é o logotipo oficial da Docker.
