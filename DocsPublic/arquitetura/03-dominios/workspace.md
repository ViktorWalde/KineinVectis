# `workspace.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Workspace (`workspace.browse` / `workspace.createFolder` / `workspace.createProject` / `workspace.open` / `workspace.recent.*`)

`workspace.open` foi implementado no protocolo `0.3.0`. `workspace.browse`
foi adicionado no protocolo `0.5.0` para o seletor proprio de workspace da UI.
`workspace.createFolder` e `workspace.createProject` foram adicionados no
protocolo `0.9.0` para permitir o fluxo JetBrains-like de criar pasta/projeto
sem sair da IDE.

`workspace.open` recebe
`{ "path": "/dir" }`, canonicaliza o caminho, identifica o tipo de projeto por
marcadores (precedência: `Cargo.toml` > `CMakeLists.txt` > `pom.xml` >
Gradle > Python) e persiste `.kinein/workspace.json`
(`schemas/workspace.schema.json`).

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "name": "meu-projeto",
    "root": "/home/user/dev/meu-projeto",
    "kind": "rustCargo",
    "markers": ["Cargo.toml", "CMakeLists.txt"],
    "capabilities": { "buildSystems": ["cargo", "cmake"] }
  }
}
```

Desde o protocolo `0.55.0`, `kind` continua sendo a classificação primária
compatível pela precedência de marcadores, enquanto `capabilities.buildSystems`
contém todos os sistemas reconhecidos na mesma varredura. Assim, um repositório
híbrido pode ser `rustCargo` e oferecer Cargo+CMake simultaneamente. O metadata
persistido usa `schemas/workspace.schema.json` 0.2.0; a UI consome o snapshot e
não repete detecção por arquivo.

**`kind: make` / `buildSystems: [make]` (`0.115.0`, P0):** um `Makefile` ou
`GNUmakefile` na raiz, sem marcador de precedência maior (um Makefile ao lado
de um `CMakeLists.txt` continua `cmake`, e `make` aparece em `buildSystems`).
`build.run` num `make` roda `bear -- <make>` na raiz quando o `bear` está
detectado (Bear 3, README: `bear -- <your-build-command>`; a
`compile_commands.json` sai na raiz e o `cdb` a vê como `"."`), senão `make` a
seco com uma linha de `event.build.output` dizendo que sem `bear` não há CDB
para o clangd — nunca um make "diferente" para fingir CDB. Sem alvo de run,
teste ou debug automático (o `Makefile` não os declara).

**`kind: platformIo` / `buildSystems: [platformIo]` (`0.117.0`, bloco E):**
um `platformio.ini` na raiz — atrás do `CMakeLists.txt` (um projeto PlatformIO
com `framework = espidf` também é uma árvore CMake; o MOTOR de build ainda
escolhe o `pio`, pelo framework), na frente de um `Makefile`. `build.run`
roda `pio run`; sem alvo de run, teste ou debug automático.

`workspace.browse` recebe `{ "path": "/dir" }`, canonicaliza o diretorio e
retorna apenas subdiretorios para a UI navegar sem depender de dialogo nativo do
desktop. A UI continua proibida de listar o filesystem diretamente.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "path": "/home/user",
    "parent": "/home",
    "entries": [
      {
        "name": "api",
        "path": "/home/user/api",
        "kind": "rustCargo"
      },
      {
        "name": "dev",
        "path": "/home/user/dev"
      }
    ],
    "places": [
      { "id": "home", "path": "/home/user" },
      { "id": "documents", "path": "/home/user/Documentos" },
      { "id": "root", "path": "/" }
    ]
  }
}
```

Desde a `0.147.0`, `kind` marca a pasta de projeto (ausente quando não há
marcador) e `places` traz os locais rápidos do seletor (ver o topo deste
documento).

`workspace.createFolder` recebe `{ "parent": "/dir", "name": "modulo" }`.
O core canonicaliza `parent`, valida que `name` é apenas um segmento de caminho
e cria o diretório filho. A resposta é `{ "path": "/dir/modulo" }`.

`workspace.createProject` recebe
`{ "parent": "/dir", "name": "demo", "template": "empty|cppCmake|rustCargo|python" }`.
O core cria o diretório do projeto, aplica o template e abre o projeto como
workspace, retornando o mesmo payload de `workspace.open`.

- `empty`: cria diretório vazio e persiste `.kinein/workspace.json`.
- `cppCmake`: cria projeto C++23/CMake strict e target-based com
  `CMakeLists.txt`, presets Ninja Debug/Release, `src/main.cpp`, diretórios
  `include/` e `tests/`, `.gitignore` e `README.md`.
- `rustCargo`: usa `cargo new --bin --vcs none`; se `cargo` não existir,
  retorna `TOOL_NOT_FOUND`.
- `python` (`0.105.0`, 2026-09-13): geração interna, sem ferramenta —
  `pyproject.toml` como a PEP 621 escreve (`requires-python = ">=3.12"`,
  `[project.optional-dependencies] dev = ["pytest"]`, `[tool.ruff]`,
  `[tool.pytest.ini_options]` com `testpaths` e `pythonpath = ["."]`),
  `main.py` (o ponto de entrada que o botão Executar procura primeiro), o
  pacote `<nome_com_underscores>/__init__.py` **na raiz** — layout plano, para
  `python main.py` e `python -m pytest` acharem o pacote sem `pip install -e .`
  antes do primeiro clique —, `tests/test_main.py`, `.gitignore` com o `.venv`
  e `README.md`. O workspace nasce como `python`; o ambiente é o clique da
  faixa de saúde. Medido em 2026-09-13: o projeto gerado roda, passa no pytest
  e no `ruff check`/`format --check` sem editar nada.

`workspace.status` responde `{ "workspace": <objeto acima> | null }`.
`workspace.close` responde `{ "status": "ok", "closed": <root | null> }`.

**Confiança no projeto** (protocolo `0.171.0`, 16.4b do `roadmaps/59`
§7.1.4): o objeto do workspace traz `trusted: bool`. A confiança é um gesto
da pessoa, `workspace.trust {}` → `{ trusted: true }`, guardado em
`trusted-projects.json` no estado global da IDE, pelo caminho canônico da
raiz; nada dentro do projeto a declara (o `.kinein/workspace.json` grava
sempre `false`). O projeto criado pelo `workspace.createProject` nasce
confiável. Sem confiança, o rust-analyzer sobe com as
`initializationOptions` (e o `workspace/configuration`)
`cargo.buildScripts.enable = false`, `procMacro.enable = false` e
`checkOnSave = false`; o `workspace.trust` o derruba (`event.lsp.restarted`)
para subir completo. A UI não roda o configure automático do CMake nem o
reconfigure ao salvar `CMakeLists.txt` sem a confiança; os gestos
(Configurar, Compilar, Executar) continuam valendo.

**Sessão por workspace** (protocolo `0.24.0`, fatia M1.5): a
resposta de `workspace.open` ganha o campo opcional
`session: { openFiles: ["/abs/..."], activeFile? }`, presente apenas quando
`.kinein/session.json` existe, tem `schemaVersion` conhecida (1) e ao menos
um arquivo ainda válido — arquivos apagados/fora do root são filtrados no
carregamento e schema desconhecido é ignorado sem erro. A escrita é feita por
`workspace.saveSession { openFiles: ["/abs/..."], activeFile? }` →
`{ files: N }` (N = entradas efetivamente salvas; caminhos inválidos são
pulados, nunca falham a sessão inteira). Em disco os caminhos são RELATIVOS
ao root (mover a pasta do projeto preserva a sessão); no contrato IPC são
sempre absolutos canônicos. A UI salva com debounce (~1.2s) a cada mudança de
abas/aba ativa e restaura pedindo `fs.read` na ordem da sessão, com a aba
ativa por último.
Caminho inexistente ou sem `path` nos params retorna `INVALID_PARAMS`; falha de
IO ao persistir, listar ou criar diretorios retorna `INTERNAL_ERROR`.

**Workspaces recentes globais** (protocolo `0.53.0`, fatia A1):
somente `workspace.open` e `workspace.createProject` concluídos com sucesso
registram a raiz canônica. O core persiste até 12 entradas em
`$XDG_CONFIG_HOME/kinein-vectis/recent-workspaces.json` (ou
`~/.config/kinein-vectis/`), conforme
`schemas/recent-workspaces.schema.json`; a UI nunca consulta o filesystem.

```text
workspace.recent.list {} → { workspaces: [RecentWorkspace] }
workspace.recent.pin { root, pinned } → mesmo snapshot completo
workspace.recent.remove { root } → mesmo snapshot completo
workspace.recent.clear {} → { workspaces: [] }

RecentWorkspace {
  name: string,
  root: string,
  lastOpenedAt: inteiro (Unix epoch em milissegundos),
  pinned: bool,
  available: bool
}
```

- entradas fixadas vêm primeiro; cada grupo é ordenado por
  `lastOpenedAt` decrescente e uma raiz canônica nunca é duplicada;
- `available` é recalculado pelo core em cada snapshot. Uma pasta removida
  continua listada, desabilitada e removível, sem ser aberta pela UI;
- abrir uma entrada reutiliza `workspace.open` e, portanto, restaura a sessão
  por workspace já existente;
- o formato legado `schemaVersion: 0` sem `pinned` é aceito e promovido na
  próxima escrita. JSON inválido ou schema futuro é tratado como lista vazia;
- pin/remover exigem uma raiz absoluta já registrada e retornam
  `INVALID_PARAMS` caso contrário. Erro de escrita retorna `INTERNAL_ERROR`;
  uma falha ao atualizar o histórico não desfaz uma abertura válida;
- o arquivo guarda apenas nome, raiz, último acesso e fixação: nunca conteúdo,
  credenciais ou contexto de IA.
