# Semântica e uso dos ícones

## 1. `kv.tree.file.cmake-lists`

Arquivo reconhecido:

```text
CMakeLists.txt
```

Uso:

- definição principal de um diretório CMake;
- criação de executáveis, bibliotecas, testes e targets personalizados;
- inclusão de subdiretórios;
- declaração de dependências e propriedades;
- configuração de instalação e empacotamento.

O nome completo tem precedência sobre a extensão `.txt`.

Ações contextuais possíveis:

```text
Configurar projeto;
Recarregar modelo CMake;
Mostrar targets;
Selecionar preset;
Executar build;
Abrir CMake Tool Window.
```

## 2. `kv.tree.file.project-config`

Ícone genérico para arquivos que alteram o ambiente, o toolchain ou o
comportamento das ferramentas do projeto.

Mapeamentos iniciais:

```text
.env
.env.*
.clangd
.clang-format
.clang-tidy
CMakePresets.json
CMakeUserPresets.json
compile_commands.json
Cargo.toml
Cargo.lock
rust-toolchain.toml
rustfmt.toml
clippy.toml
Cross.toml
.cargo/config.toml
```

Esse ícone é um fallback semântico. Ícones especializados futuros devem ter
precedência, por exemplo:

```text
kv.tree.file.cmake-presets
kv.tree.file.cmake-user-presets
kv.tree.file.cargo-manifest
kv.tree.file.rust-toolchain
kv.tree.file.clang-config
```

Arquivos `.env` devem receber tratamento de segurança adicional:

- não exibir valores em tooltip;
- não enviar conteúdo para ferramentas externas automaticamente;
- alertar quando segredos forem versionados;
- evitar inclusão em logs.

## 3. `kv.tree.file.sql`

Mapeamento:

```text
*.sql
```

Uso:

- DDL: schemas, tabelas, índices e constraints;
- DML: consultas e alterações de dados;
- migrations;
- seeds;
- views, functions e triggers.

Variações futuras:

```text
kv.tree.file.database-schema
kv.tree.file.database-migration
kv.tree.file.database-seed
```

A resolução por nome e caminho deve preceder a extensão:

```text
schema.sql
seed.sql
migrations/V001__create_table.sql
```

## 4. `kv.tree.file.docker-yaml`

Mapeamentos seguros:

```text
compose.yaml
compose.yml
docker-compose.yaml
docker-compose.yml
compose.*.yaml
compose.*.yml
docker-compose.*.yaml
docker-compose.*.yml
```

O ícone não deve ser aplicado a todo arquivo `.yaml` ou `.yml`.

Um YAML só deve receber o ícone Docker quando:

1. o nome é reconhecido como arquivo Compose;
2. o caminho foi configurado pelo workspace;
3. o conteúdo foi validado pelo schema Compose;
4. a ferramenta Docker/Compose confirmou o arquivo.

Isso evita classificar incorretamente:

```text
GitHub Actions;
Kubernetes;
Ansible;
OpenAPI;
Yocto;
west manifest;
configuração genérica.
```

## 5. Precedência do resolver

```text
1. nome completo exato;
2. padrão especial;
3. caminho especial;
4. extensão composta;
5. extensão simples;
6. arquivo genérico.
```

Exemplos:

```text
CMakeLists.txt     → cmake-lists
compose.yaml       → docker-yaml
config.yaml        → YAML genérico, não Docker
query.sql          → SQL
CMakePresets.json  → project-config
README.txt         → texto genérico
```
