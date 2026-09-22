# Kinein Vectis — Special File Icon Pack

Pacote de ícones de arquivos especiais para a árvore de projetos do Kinein
Vectis.

## Conteúdo

```text
ui/assets/icons/tree/*.png      masters de produção de 64 px
files/*.png                     entradas autorais em alta resolução
FILE_ICON_MAPPINGS.json         precedência por nome e extensão
ICON_CATALOG.json               catálogo semântico
docs/                           dimensionamento, integração e validação
```

## Ícones de produção

- `file-cmakelists.png`: `CMakeLists.txt`;
- `file-c.png`: fontes C;
- `file-cpp.png`: fontes C++;
- `file-h.png`: headers `.h`;
- `file-hpp.png`: headers C++ (`.hpp`, `.hh`, `.hxx`, `.h++`, `.ipp`);
- `file-cargo.png`: `Cargo.toml` e `Cargo.lock`;
- `file-makefile.png`: `Makefile`, `GNUmakefile` e `*.mk`;
- `file-pyproject.png`: `pyproject.toml`;
- `file-ros.png`: `package.xml` e arquivos `*.launch.xml` do ROS 2;
- `file-rust.png`: Rust;
- `file-python.png`: Python;
- `file-yaml.png`: YAML genérico;
- `file-sql.png`: SQL;
- `file-markdown.png`: Markdown;
- `file-docker.png`: Dockerfile e Docker Compose.

Os PNGs de produção vivem em `ui/assets/icons/tree`. O arquivo de entrada
`file-aql.png` foi interpretado como SQL porque sua metáfora é um banco de
dados e o conjunto solicitado associa esse asset a `.sql`.

Os tipos de arquivo compilados pela UI usam apenas PNG. Os SVGs legados de C,
C++, Rust e Python foram removidos de `ui/assets/icons/tree`; os dois SVGs que
permanecem nesse diretório são os assets ativos de pasta aberta e fechada.

Os PNGs compilados foram reduzidos para masters de 64 px, adequados à exibição
em 16–20 px e a telas HiDPI sem carregar os originais de mais de 1200 px na
memória da interface.

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
