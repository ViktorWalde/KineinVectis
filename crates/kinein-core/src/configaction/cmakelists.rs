//! As seis acoes que editam o `CMakeLists.txt` da raiz.
//!
//! Nenhuma delas faz parser de `CMakeLists.txt`: a IDE **acrescenta** comandos
//! e, para decidir se um target existe, olha as invocacoes literais de
//! `add_executable`/`add_library` no texto. Um parser universal de `CMake` e
//! explicitamente NAO-MVP (spec de MVP §11.2), e a lista de targets de verdade
//! ja vem do file-api oficial (`crate::cmake::list_targets`) depois do
//! configure.
//!
//! Nomes de target sao validados pelo padrao que a propria `CMake` documenta na
//! politica CMP0037 (medido em `cmake --help-policy CMP0037`, `CMake` 4.3.0):
//! letras, digitos, `_`, `.`, `+` e `-`, e nomes reservados (`all`, `clean`,
//! `help`, `install`, `test`, `package`) recusados.

use std::collections::BTreeMap;

use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_required};
use super::{optional_param, required_param};

/// Arquivo editado por todas as acoes deste modulo.
pub(super) const CMAKELISTS: &str = "CMakeLists.txt";

/// Variavel que exporta a compilation database consumida pelo clangd.
pub(super) const EXPORT_COMPILE_COMMANDS: &str = "CMAKE_EXPORT_COMPILE_COMMANDS";

/// Nomes que os geradores da `CMake` reservam (CMP0037).
const RESERVED_TARGETS: &[&str] = &["all", "clean", "help", "install", "test", "package"];

/// Visibilidades aceitas nos comandos `target_*`.
const VISIBILITIES: &[&str] = &["PRIVATE", "PUBLIC", "INTERFACE"];

/// Targets declarados literalmente no texto do `CMakeLists.txt`.
#[must_use]
pub(super) fn declared_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        for command in ["add_executable(", "add_library("] {
            let Some(rest) = trimmed.strip_prefix(command) else {
                continue;
            };
            let name = rest
                .split(|ch: char| ch.is_whitespace() || ch == ')')
                .next()
                .unwrap_or("")
                .trim();
            if !name.is_empty() && !targets.iter().any(|existing| existing == name) {
                targets.push(name.to_owned());
            }
        }
    }
    targets
}

/// O `CMakeLists.txt` ja liga a exportacao da compilation database?
#[must_use]
pub(super) fn exports_compile_commands(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with('#') && trimmed.contains(EXPORT_COMPILE_COMMANDS)
    })
}

/// Liga `CMAKE_EXPORT_COMPILE_COMMANDS` logo apos o `cmake_minimum_required`.
///
/// A posicao importa e nao e estetica: a documentacao oficial (`CMake` 4.3.0,
/// `cmake --help-variable CMAKE_EXPORT_COMPILE_COMMANDS`) diz que a variavel
/// "initializes the `EXPORT_COMPILE_COMMANDS` target property for all targets"
/// — ou seja, ela precisa valer **antes** de o target ser criado. No fim do
/// arquivo ela nao afetaria nenhum target ja declarado.
pub(super) fn enable_compile_commands(
    root: &std::path::Path,
) -> Result<ActionPlan, ConfigActionError> {
    let before = read_required(root, CMAKELISTS)?;
    if exports_compile_commands(&before) {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{CMAKELISTS} ja define {EXPORT_COMPILE_COMMANDS}"),
        });
    }

    let statement = format!("set({EXPORT_COMPILE_COMMANDS} ON)\n");
    let after = insert_after_minimum_required(&before, &statement);
    Ok(ActionPlan::edit(
        format!("Liga {EXPORT_COMPILE_COMMANDS} no {CMAKELISTS}"),
        PlannedFile {
            path: CMAKELISTS.to_owned(),
            before: Some(before),
            after,
        },
    )
    .with_note(
        "Vale na proxima configuracao do CMake; rode 'Configurar' para o clangd \
         receber as flags novas."
            .to_owned(),
    ))
}

/// `add_executable(<name> <sources...>)`.
pub(super) fn add_executable(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    add_target(root, params, "add_executable", None)
}

/// `add_library(<name> STATIC <sources...>)`.
pub(super) fn add_static_library(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    add_target(root, params, "add_library", Some("STATIC"))
}

fn add_target(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
    command: &str,
    kind: Option<&str>,
) -> Result<ActionPlan, ConfigActionError> {
    let name = validate_target_name(required_param(params, "name")?)?;
    let sources = validate_sources(required_param(params, "sources")?)?;
    let before = read_required(root, CMAKELISTS)?;
    if declared_targets(&before).contains(&name) {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("o target {name} ja esta declarado no {CMAKELISTS}"),
        });
    }

    let head = kind.map_or_else(|| name.clone(), |kind| format!("{name} {kind}"));
    let block = command_block(command, &head, &sources);
    Ok(edit_plan(
        format!("Acrescenta {command}({name}) ao {CMAKELISTS}"),
        before,
        &block,
    ))
}

/// `target_sources(<target> <visibility> <source>)`.
pub(super) fn add_source_to_target(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let sources = validate_sources(required_param(params, "sources")?)?;
    let visibility = visibility_of(params)?;
    let block = command_block(
        "target_sources",
        &format!("{target} {visibility}"),
        &sources,
    );
    Ok(edit_plan(
        format!("Acrescenta {} fonte(s) ao target {target}", sources.len()),
        before,
        &block,
    ))
}

/// `target_include_directories(<target> <visibility> <dir>)`.
pub(super) fn add_include_directory(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let directories = validate_sources(required_param(params, "directories")?)?;
    let visibility = visibility_of(params)?;
    let block = command_block(
        "target_include_directories",
        &format!("{target} {visibility}"),
        &directories,
    );
    Ok(edit_plan(
        format!("Acrescenta include dir ao target {target}"),
        before,
        &block,
    ))
}

/// `target_link_libraries(<target> <visibility> <libs...>)`.
pub(super) fn add_target_link_libraries(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let (target, before) = existing_target(root, params)?;
    let libraries = validate_arguments(required_param(params, "libraries")?, "libraries")?;
    let visibility = visibility_of(params)?;
    let block = command_block(
        "target_link_libraries",
        &format!("{target} {visibility}"),
        &libraries,
    );
    Ok(edit_plan(
        format!("Linka {} biblioteca(s) ao target {target}", libraries.len()),
        before,
        &block,
    ))
}

/// `find_package(<pkg> [<versao>] CONFIG REQUIRED)`.
///
/// `CONFIG` e' explicito de proposito: sem ele o `CMake` tambem aceita um
/// find module do proprio `CMake`, e a mensagem de erro quando nada e' achado
/// fica ambigua entre "nao instalado" e "nao ha find module".
pub(super) fn find_package(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let before = read_required(root, CMAKELISTS)?;
    let package = one_argument(required_param(params, "package")?, "package")?;
    let version = params
        .get("version")
        .map(String::as_str)
        .filter(|v| !v.trim().is_empty());
    let argumentos = match version {
        Some(version) => {
            let version = one_argument(version, "version")?;
            format!("{package} {version} CONFIG REQUIRED")
        }
        None => format!("{package} CONFIG REQUIRED"),
    };
    Ok(edit_plan(
        format!("Procura o pacote {package} instalado no sistema"),
        before,
        &format!("find_package({argumentos})\n"),
    ))
}

/// `FetchContent_Declare` mais `FetchContent_MakeAvailable`, com tag PINADA.
///
/// A tag e' obrigatoria e nunca e' um nome de branch por acidente: `main` muda
/// sob os pes do usuario e transforma um build que passava em um build que
/// falha sem ninguem ter mexido no codigo.
pub(super) fn fetch_content(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<ActionPlan, ConfigActionError> {
    let before = read_required(root, CMAKELISTS)?;
    let name = one_argument(required_param(params, "name")?, "name")?;
    let repository = one_argument(required_param(params, "repository")?, "repository")?;
    let tag = one_argument(required_param(params, "tag")?, "tag")?;
    // Indentacao de QUATRO espacos, uniforme. Ate' 2026-09-04 este bloco saia
    // com treze espacos antes do `GIT_REPOSITORY` e nove antes do
    // `FetchContent_MakeAvailable` — a IDE escrevia CMake torto no arquivo do
    // autor, e foi lendo o proprio resultado que ele reclamou. Um `format!`
    // de uma linha so' com `\n` embutido esconde exatamente esse tipo de erro;
    // por isso o bloco agora e' escrito como texto, com as linhas a vista.
    let bloco = format!(
        "include(FetchContent)\n\
         FetchContent_Declare(\n\
         \x20   {name}\n\
         \x20   GIT_REPOSITORY {repository}\n\
         \x20   GIT_TAG {tag}\n\
         )\n\
         FetchContent_MakeAvailable({name})\n"
    );
    Ok(edit_plan(
        format!("Baixa {name} na tag {tag} e disponibiliza ao projeto"),
        before,
        &bloco,
    ))
}

pub(super) fn edit_plan(summary: String, before: String, block: &str) -> ActionPlan {
    let after = append_block(&before, block);
    ActionPlan::edit(
        summary,
        PlannedFile {
            path: CMAKELISTS.to_owned(),
            before: Some(before),
            after,
        },
    )
}

/// Le o `CMakeLists.txt` e exige que o target pedido exista nele.
pub(super) fn existing_target(
    root: &std::path::Path,
    params: &BTreeMap<String, String>,
) -> Result<(String, String), ConfigActionError> {
    let target = validate_target_name(required_param(params, "target")?)?;
    let before = read_required(root, CMAKELISTS)?;
    let declared = declared_targets(&before);
    if !declared.contains(&target) {
        return Err(ConfigActionError::NotApplicable {
            reason: format!(
                "o target {target} nao esta declarado no {CMAKELISTS} (declarados: {})",
                if declared.is_empty() {
                    "nenhum".to_owned()
                } else {
                    declared.join(", ")
                }
            ),
        });
    }
    Ok((target, before))
}

fn visibility_of(params: &BTreeMap<String, String>) -> Result<String, ConfigActionError> {
    let raw = optional_param(params, "visibility").unwrap_or("PRIVATE");
    let upper = raw.trim().to_uppercase();
    if VISIBILITIES.contains(&upper.as_str()) {
        Ok(upper)
    } else {
        Err(ConfigActionError::InvalidParam {
            name: "visibility",
            reason: format!("use uma de {}", VISIBILITIES.join(", ")),
        })
    }
}

/// Monta o bloco no mesmo estilo dos templates de projeto da IDE.
pub(super) fn command_block(command: &str, head: &str, arguments: &[String]) -> String {
    let mut block = format!("{command}({head}\n");
    for argument in arguments {
        block.push_str("    ");
        block.push_str(argument);
        block.push('\n');
    }
    block.push_str(")\n");
    block
}

/// Acrescenta o bloco ao fim do arquivo, sempre com uma linha em branco antes.
pub(super) fn append_block(before: &str, block: &str) -> String {
    let mut after = before.trim_end().to_owned();
    if !after.is_empty() {
        after.push_str("\n\n");
    }
    after.push_str(block);
    after
}

/// Insere `statement` depois do `cmake_minimum_required(...)`; sem ele, no topo.
pub(super) fn insert_after_minimum_required(before: &str, statement: &str) -> String {
    let lines: Vec<&str> = before.lines().collect();
    let anchor = lines
        .iter()
        .position(|line| line.trim_start().starts_with("cmake_minimum_required("));
    let mut after = String::with_capacity(before.len() + statement.len() + 2);
    if let Some(index) = anchor {
        for line in &lines[..=index] {
            after.push_str(line);
            after.push('\n');
        }
        after.push('\n');
        after.push_str(statement);
        for line in &lines[index + 1..] {
            if after.ends_with("\n\n") && line.trim().is_empty() {
                continue;
            }
            after.push_str(line);
            after.push('\n');
        }
    } else {
        after.push_str(statement);
        after.push('\n');
        after.push_str(before);
    }
    after
}

/// Valida um nome de target contra o padrao documentado em CMP0037.
fn validate_target_name(raw: &str) -> Result<String, ConfigActionError> {
    let name = raw.trim();
    let param = "name";
    if name.is_empty() {
        return Err(ConfigActionError::InvalidParam {
            name: param,
            reason: "o nome do target nao pode ser vazio".to_owned(),
        });
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '+' | '-'))
    {
        return Err(ConfigActionError::InvalidParam {
            name: param,
            reason: "CMP0037: use letras, digitos, '_', '.', '+' ou '-'".to_owned(),
        });
    }
    if RESERVED_TARGETS
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(name))
    {
        return Err(ConfigActionError::InvalidParam {
            name: param,
            reason: format!("CMP0037: {name} e um nome reservado pelos geradores da CMake"),
        });
    }
    Ok(name.to_owned())
}

/// Valida caminhos de fonte/diretorio: relativos, sem escapar do projeto.
fn validate_sources(raw: &str) -> Result<Vec<String>, ConfigActionError> {
    let values = validate_arguments(raw, "sources")?;
    for value in &values {
        if value.starts_with('/') {
            return Err(ConfigActionError::InvalidParam {
                name: "sources",
                reason: format!("{value} e absoluto; use caminho relativo ao projeto"),
            });
        }
        if value.split('/').any(|part| part == "..") {
            return Err(ConfigActionError::InvalidParam {
                name: "sources",
                reason: format!("{value} sai da raiz do projeto"),
            });
        }
    }
    Ok(values)
}

/// Quebra a lista por espaco/virgula e recusa o que quebraria a sintaxe.
/// Um argumento so', com a MESMA validacao de caractere dos varios: um pacote
/// nao pode carregar `$` ou `)` por acidente e virar comando de `CMake`.
pub(super) fn one_argument(raw: &str, name: &'static str) -> Result<String, ConfigActionError> {
    let mut valores = validate_arguments(raw, name)?;
    if valores.len() != 1 {
        return Err(ConfigActionError::InvalidParam {
            name,
            reason: format!("esperado um valor so', veio {}", valores.len()),
        });
    }
    Ok(valores.remove(0))
}

fn validate_arguments(raw: &str, name: &'static str) -> Result<Vec<String>, ConfigActionError> {
    let values: Vec<String> = raw
        .split([',', ' ', '\t', '\n'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect();
    if values.is_empty() {
        return Err(ConfigActionError::MissingParam { name });
    }
    for value in &values {
        if value.contains(['"', '(', ')', '$', '\\', ';']) {
            return Err(ConfigActionError::InvalidParam {
                name,
                reason: format!("{value} tem caractere que a IDE nao escreve em CMake"),
            });
        }
    }
    Ok(values)
}
