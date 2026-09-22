# Kinein Vectis — Special File Icon Pack

Pacote de ícones de arquivos especiais para a árvore de projetos do Kinein
Vectis.

## Conteúdo

```text
ui/qml/components/KvFileIconGlyphs.js  glifos vetoriais de produção 24x24
files/*.png                           referências autorais em alta resolução
FILE_ICON_MAPPINGS.json         precedência por nome e extensão
ICON_CATALOG.json               catálogo semântico
docs/                           dimensionamento, integração e validação
```

## Ícones de produção

- `tree-file-cmakelists`: `CMakeLists.txt`;
- `tree-file-c` e `tree-file-cpp`: fontes C e C++;
- `tree-file-h` e `tree-file-hpp`: headers C e C++;
- `tree-file-cargo`: `Cargo.toml` e `Cargo.lock`;
- `tree-file-makefile`: `Makefile`, `GNUmakefile` e `*.mk`;
- `tree-file-pyproject`: `pyproject.toml`;
- `tree-file-ros`: `package.xml` e arquivos `*.launch.xml` do ROS 2;
- `tree-file-rust`, `tree-file-python`, `tree-file-yaml`, `tree-file-sql` e
  `tree-file-markdown`: linguagens e formatos;
- `tree-file-docker`: Dockerfile e Docker Compose.

Os glifos de produção vivem em `KvFileIconGlyphs.js`, são renderizados pelo
`Canvas` de `KvIcon.qml` e usam uma grade vetorial 24x24. O arquivo de entrada
`file-aql.png` foi interpretado como referência de SQL porque sua metáfora é um
banco de dados e o conjunto solicitado associa esse asset a `.sql`.

As ilustrações PNG de alta resolução em `files/` ficam como referência visual e
não entram no recurso Qt. A redução direta dessas ilustrações criava microtexto,
sombras e detalhes abaixo de um pixel. A versão de produção limita cada ícone a
uma metáfora principal, traço mínimo de 1,35 unidades e contraste próprio para
16–24 px. Os dois SVGs em `ui/assets/icons/tree` continuam sendo os assets de
pasta aberta e fechada.

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
