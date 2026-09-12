//! Servico `CMake`: configure explicito, presets, targets via file-api e status.
//!
//! O diretorio de build e UNICO e imutavel (`<root>/.kinein/build`) — o mesmo
//! usado por `build.run` e `run.start`; presets nao mudam o diretorio (o `-B`
//! explicito tem precedencia). Targets vem da resposta `codemodel-v2` do
//! file-api oficial do `CMake`, nunca de parser proprio de `CMakeLists.txt`.

pub mod model;

use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use kinein_protocol::{CmakePresetInfo, CmakeTargetInfo, ToolchainRole};
use serde_json::Value;

/// Maximo de targets repassados a UI por request.
const MAX_TARGETS: usize = 200;

/// Diretorio de build canonico do workspace.
#[must_use]
pub fn build_dir(root: &Path) -> PathBuf {
    root.join(".kinein").join("build")
}

/// Estado do configure para `cmake.status`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CmakeStatus {
    /// `CMakeCache.txt` existe no build dir.
    pub configured: bool,
    /// `compile_commands.json` existe no build dir.
    pub has_compile_commands: bool,
    /// Build dir canonico.
    pub build_dir: PathBuf,
}

/// Le o estado atual por stat dos artefatos do configure.
#[must_use]
pub fn status(root: &Path) -> CmakeStatus {
    let build_dir = build_dir(root);
    CmakeStatus {
        configured: build_dir.join("CMakeCache.txt").is_file(),
        has_compile_commands: build_dir.join("compile_commands.json").is_file(),
        build_dir,
    }
}

/// Escreve a query `codemodel-v2` do file-api antes do configure, para o
/// `CMake` responder com os targets em `.cmake/api/v1/reply`.
pub fn write_file_api_query(root: &Path) -> io::Result<()> {
    let query = build_dir(root)
        .join(".cmake")
        .join("api")
        .join("v1")
        .join("query");
    fs::create_dir_all(&query)?;
    // A toolchains-v1 (CMake >= 3.20) da' o compilador de cada linguagem com
    // versao e includes implicitos: e' o que preenche a unidade de compilacao
    // de um arquivo quando nao ha' compile_commands.json (cmake/model.rs).
    fs::write(query.join("toolchains-v1"), "")?;
    fs::write(query.join("codemodel-v2"), "")
}

/// Monta o comando de configure com a CDB exportada e o `-B` fixo.
///
/// `toolchain` e a escolha do usuario (roadmap 30, etapa 5). Ela entra por
/// dois caminhos: o EXECUTAVEL do `cmake` (quando fixado) e os argumentos
/// `-G`/`-DCMAKE_*_COMPILER`. Sem escolha nenhuma, o comando sai byte a byte
/// como saia antes — o padrao continua sendo o `PATH`.
#[must_use]
pub fn configure_command(
    root: &Path,
    preset: Option<&str>,
    toolchain: &crate::toolchain::Toolchain,
) -> Command {
    let programa = toolchain
        .program_for(ToolchainRole::Cmake)
        .unwrap_or_else(|| PathBuf::from("cmake"));
    let mut command = Command::new(programa);
    if let Some(preset) = preset {
        command.arg("--preset").arg(preset);
    }
    command.arg("-S").arg(root).arg("-B").arg(build_dir(root));
    command.arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON");
    // Depois do `-D` fixo e antes de nada: um preset que ja escolhe gerador
    // conflita com um `-G` explicito, e o CMake reclama em vez de adivinhar —
    // que e o comportamento certo, e a mensagem dele nomeia o conflito.
    command.args(toolchain.cmake_arguments());
    command
}

/// Lista os configure presets nao ocultos de `CMakePresets.json` e
/// `CMakeUserPresets.json`, na ordem dos arquivos.
pub fn list_presets(root: &Path) -> Result<Vec<CmakePresetInfo>, String> {
    let mut presets = Vec::new();
    for file in ["CMakePresets.json", "CMakeUserPresets.json"] {
        let path = root.join(file);
        if !path.is_file() {
            continue;
        }
        let body =
            fs::read_to_string(&path).map_err(|error| format!("falha lendo {file}: {error}"))?;
        let value: Value =
            serde_json::from_str(&body).map_err(|error| format!("{file} invalido: {error}"))?;
        let Some(items) = value.get("configurePresets").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            if item.get("hidden").and_then(Value::as_bool) == Some(true) {
                continue;
            }
            let Some(name) = item.get("name").and_then(Value::as_str) else {
                continue;
            };
            presets.push(CmakePresetInfo {
                name: name.to_owned(),
                display_name: item
                    .get("displayName")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
    }
    Ok(presets)
}

/// Os `kind` de target em que se PODE linkar uma biblioteca.
///
/// POR QUE ESTA LISTA EXISTE (2026-09-04). Relato de uso do autor: "apareceram
/// varios arquivos para onde linkar e isso ficou confuso". Medido no proprio
/// repositorio da IDE, o `cmake.targets.list` devolvia 19 nomes — e DEZOITO
/// deles eram utilitarios que o Qt e o `CMake` geram sozinhos:
///
/// ```text
/// all_qmllint            kinein-vectis_autogen        kinein-vectis_copy_qml
/// all_aotstats           kinein-vectis_qmlimportscan  ... e mais treze
/// ```
///
/// Nao era so' confuso: era ERRADO. `target_link_libraries` num target
/// `utility` falha. A IDE oferecia dezoito caminhos que terminam em erro e um
/// que funciona, sem distinguir.
///
/// `interfaceLibrary` fica de fora por um motivo diferente e igualmente
/// concreto: ela SO' aceita visibilidade `INTERFACE`, e o plano que a IDE
/// escreve usa `PRIVATE`. Oferece-la seria oferecer outro erro.
const LINKABLE_KINDS: [&str; 5] = [
    "EXECUTABLE",
    "STATIC_LIBRARY",
    "SHARED_LIBRARY",
    "MODULE_LIBRARY",
    "OBJECT_LIBRARY",
];

/// `true` quando da' para linkar uma biblioteca neste target.
///
/// O file-api escreve `STATIC_LIBRARY`; a UI e o scanner de fonte usam
/// `staticLibrary`. Comparar sem `_` e sem caixa cobre as duas grafias sem
/// precisar de tabela de traducao.
#[must_use]
fn is_linkable_kind(kind: &str) -> bool {
    let normalizado = kind.replace('_', "").to_ascii_uppercase();
    LINKABLE_KINDS
        .iter()
        .any(|aceito| aceito.replace('_', "") == normalizado)
}

/// Le os nomes de target direto do `CMakeLists.txt`, sem configurar nada.
///
/// POR QUE ISTO EXISTE (2026-09-04). Relato de uso do autor: "o `SQLite` eu nao
/// consegui ativar". O painel de bibliotecas pedia que ele DIGITASSE o nome do
/// alvo do `CMake` que vai linkar, e enquanto o campo estivesse vazio o plano
/// nao aparecia. A IDE cobrava do autor uma informacao que esta' escrita no
/// projeto dele.
///
/// O `list_targets` so' responde DEPOIS de um configure, porque le' o file-api.
/// Num projeto recem-aberto isso e' vazio, e e' justamente ai' que o autor quer
/// escolher bibliotecas. Este scanner cobre esse buraco lendo a fonte.
///
/// LIMITES, e eles sao reais — por isso o resultado vem marcado como `source`
/// e a UI diz de onde veio:
///
/// ```text
/// add_executable(${NOME} ...)   PULADO: o nome e' variavel, e resolve-lo
///                               exigiria interpretar CMake
/// ALIAS / IMPORTED              PULADOS: nao sao alvos aos quais se linka
/// nome dentro de if()           incluido mesmo se a condicao for falsa —
///                               este scanner nao avalia condicao
/// ```
///
/// Nada disto e' aproximacao perigosa: o pior caso e' oferecer um nome a mais
/// numa lista, e o autor ve' a lista antes de escolher.
#[must_use]
pub fn targets_from_source(root: &Path) -> Vec<CmakeTargetInfo> {
    let mut encontrados = Vec::new();
    let mut vistos = std::collections::BTreeSet::new();
    for arquivo in cmake_lists_files(root) {
        let Ok(texto) = fs::read_to_string(&arquivo) else {
            continue;
        };
        for linha in texto.lines() {
            let limpa = linha.split('#').next().unwrap_or("");
            for (chamada, tipo) in [
                ("add_executable(", "executable"),
                ("add_library(", "library"),
            ] {
                let Some(posicao) = limpa.to_ascii_lowercase().find(chamada) else {
                    continue;
                };
                let resto = &limpa[posicao + chamada.len()..];
                let nome = resto
                    .split(|c: char| c.is_whitespace() || c == ')')
                    .find(|parte| !parte.is_empty())
                    .unwrap_or("");
                // Nome vazio, com variavel, ou alias/importado: nao serve.
                let maiusculo = resto.to_ascii_uppercase();
                if nome.is_empty()
                    || nome.contains('$')
                    || maiusculo.contains(" ALIAS ")
                    || maiusculo.contains(" IMPORTED")
                    // INTERFACE so' aceita visibilidade INTERFACE, e o plano
                    // que a IDE escreve usa PRIVATE. Ver `LINKABLE_KINDS`.
                    || maiusculo.contains(" INTERFACE")
                {
                    continue;
                }
                if vistos.insert(nome.to_owned()) {
                    encontrados.push(CmakeTargetInfo {
                        name: nome.to_owned(),
                        kind: tipo.to_owned(),
                        ..CmakeTargetInfo::default()
                    });
                }
            }
        }
        if encontrados.len() >= MAX_TARGETS {
            break;
        }
    }
    encontrados.truncate(MAX_TARGETS);
    encontrados
}

/// Os `CMakeLists.txt` do projeto, sem entrar em diretorio de build.
///
/// Profundidade limitada de proposito: um projeto com dezenas de milhares de
/// diretorios nao pode travar a abertura do painel de bibliotecas.
pub(crate) fn cmake_lists_files(root: &Path) -> Vec<PathBuf> {
    const PROFUNDIDADE_MAXIMA: usize = 3;
    const IGNORADOS: [&str; 6] = [".kinein", ".git", "build", "target", "node_modules", "out"];

    let mut arquivos = Vec::new();
    let mut fila = vec![(root.to_path_buf(), 0usize)];
    while let Some((diretorio, nivel)) = fila.pop() {
        let candidato = diretorio.join("CMakeLists.txt");
        if candidato.is_file() {
            arquivos.push(candidato);
        }
        if nivel >= PROFUNDIDADE_MAXIMA {
            continue;
        }
        let Ok(entradas) = fs::read_dir(&diretorio) else {
            continue;
        };
        for entrada in entradas.flatten() {
            if !entrada.path().is_dir() {
                continue;
            }
            let nome = entrada.file_name().to_string_lossy().into_owned();
            if nome.starts_with('.') || IGNORADOS.contains(&nome.as_str()) {
                continue;
            }
            fila.push((entrada.path(), nivel + 1));
        }
    }
    arquivos.sort();
    arquivos
}

/// Le os targets do reply `codemodel-v2` do ultimo configure, com o MODELO
/// de cada um (`cmake/model.rs`): fontes, artefatos, linguagens, flags,
/// sysroot, dependencias.
///
/// Sem reply (nunca configurado com a query) retorna vazio; a UI explica.
#[must_use]
pub fn list_targets(root: &Path) -> Vec<CmakeTargetInfo> {
    let Some(modelo) = model::CmakeModel::load(&build_dir(root)) else {
        return Vec::new();
    };
    modelo
        .targets
        .iter()
        .map(|t| (target_kind_name(&t.kind), t))
        // Utilitario gerado pelo Qt/CMake nao aceita `target_link_libraries`;
        // ver `LINKABLE_KINDS`. Devolve-lo seria oferecer um caminho que
        // termina em erro.
        .filter(|(kind, _)| is_linkable_kind(kind))
        .take(MAX_TARGETS)
        .map(|(kind, t)| {
            let mut includes: Vec<&str> = Vec::new();
            let mut defines: Vec<&str> = Vec::new();
            let mut languages: Vec<String> = Vec::new();
            for g in &t.compile_groups {
                includes.extend(g.includes.iter().map(String::as_str));
                defines.extend(g.defines.iter().map(String::as_str));
                if !languages.contains(&g.language) {
                    languages.push(g.language.clone());
                }
            }
            includes.sort_unstable();
            includes.dedup();
            defines.sort_unstable();
            defines.dedup();
            CmakeTargetInfo {
                name: t.name.clone(),
                kind,
                artifacts: t
                    .artifacts
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                sources: t.sources.iter().filter(|s| !s.generated).count() as u64,
                generated_sources: t.sources.iter().filter(|s| s.generated).count() as u64,
                languages,
                standard: t
                    .compile_groups
                    .first()
                    .and_then(|g| g.standard.clone()),
                includes: includes.len() as u64,
                defines: defines.len() as u64,
                sysroot: t
                    .compile_groups
                    .iter()
                    .find_map(|g| g.sysroot.clone()),
                dependencies: t.dependencies.clone(),
                source_dir: Some(t.source_dir.display().to_string()),
            }
        })
        .collect()
}

/// Converte o `type` do file-api (`EXECUTABLE`, `STATIC_LIBRARY`, ...) para
/// o kind camelCase do protocolo (`executable`, `staticLibrary`, ...).
fn target_kind_name(raw: &str) -> String {
    let mut kind = String::with_capacity(raw.len());
    let mut uppercase_next = false;
    for ch in raw.chars() {
        if ch == '_' {
            uppercase_next = true;
            continue;
        }
        if uppercase_next {
            kind.extend(ch.to_uppercase());
            uppercase_next = false;
        } else {
            kind.extend(ch.to_lowercase());
        }
    }
    kind
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        configure_command, list_presets, list_targets, status, targets_from_source,
        write_file_api_query,
    };
    use crate::toolchain::Toolchain;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cmake-tests")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[test]
    fn status_reflects_configure_artifacts() {
        let root = temp_root("status");
        let before = status(&root);
        assert!(!before.configured);
        assert!(!before.has_compile_commands);

        std::fs::create_dir_all(before.build_dir.clone()).unwrap();
        std::fs::write(before.build_dir.join("CMakeCache.txt"), "#\n").unwrap();
        std::fs::write(before.build_dir.join("compile_commands.json"), "[]\n").unwrap();
        let after = status(&root);
        assert!(after.configured);
        assert!(after.has_compile_commands);
    }

    #[test]
    fn configure_command_pins_build_dir_and_exports_cdb() {
        let root = temp_root("command");
        let command = configure_command(&root, Some("dev"), &Toolchain::resolve(&root, &[]));
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        assert_eq!(arguments[0], "--preset");
        assert_eq!(arguments[1], "dev");
        assert!(arguments.contains(&"-S".to_owned()));
        assert!(arguments.contains(&"-DCMAKE_EXPORT_COMPILE_COMMANDS=ON".to_owned()));
        let build_index = arguments.iter().position(|arg| arg == "-B").unwrap();
        assert!(arguments[build_index + 1].ends_with(".kinein/build"));
    }

    #[test]
    fn file_api_query_is_written_inside_build_dir() {
        let root = temp_root("query");
        write_file_api_query(&root).unwrap();
        assert!(
            root.join(".kinein/build/.cmake/api/v1/query/codemodel-v2")
                .is_file()
        );
    }

    #[test]
    fn presets_merge_both_files_and_skip_hidden() {
        let root = temp_root("presets");
        std::fs::write(
            root.join("CMakePresets.json"),
            r#"{ "version": 6, "configurePresets": [
                { "name": "base", "hidden": true },
                { "name": "release", "displayName": "Release" }
            ] }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("CMakeUserPresets.json"),
            r#"{ "version": 6, "configurePresets": [ { "name": "dev-local" } ] }"#,
        )
        .unwrap();

        let presets = list_presets(&root).unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, "release");
        assert_eq!(presets[0].display_name.as_deref(), Some("Release"));
        assert_eq!(presets[1].name, "dev-local");

        std::fs::write(root.join("CMakeUserPresets.json"), "nao é json").unwrap();
        assert!(list_presets(&root).is_err());
    }

    /// O relato de uso de 2026-09-04: dezoito dos dezenove "alvos" eram
    /// utilitarios do Qt, e `target_link_libraries` neles FALHA.
    #[test]
    fn utilitario_gerado_nao_entra_na_lista_de_onde_linkar() {
        let root = temp_root("targets-linkaveis");
        let reply = root.join(".kinein/build/.cmake/api/v1/reply");
        std::fs::create_dir_all(&reply).unwrap();
        std::fs::write(
            reply.join("codemodel-v2-abc.json"),
            r#"{ "configurations": [ { "targets": [
                { "name": "app", "jsonFile": "t-app.json" },
                { "name": "all_qmllint", "jsonFile": "t-lint.json" },
                { "name": "core", "jsonFile": "t-core.json" },
                { "name": "so_interface", "jsonFile": "t-iface.json" }
            ] } ] }"#,
        )
        .unwrap();
        for (arquivo, tipo) in [
            ("t-app.json", "EXECUTABLE"),
            ("t-lint.json", "UTILITY"),
            ("t-core.json", "STATIC_LIBRARY"),
            ("t-iface.json", "INTERFACE_LIBRARY"),
        ] {
            std::fs::write(reply.join(arquivo), format!(r#"{{ "type": "{tipo}" }}"#)).unwrap();
        }

        let nomes: Vec<_> = list_targets(&root).into_iter().map(|t| t.name).collect();
        assert_eq!(
            nomes,
            vec!["app".to_owned(), "core".to_owned()],
            "utilitario ou INTERFACE entrou na lista de onde linkar"
        );
    }

    /// O buraco que o relato de uso de 2026-09-04 expos: projeto recem-aberto
    /// nao tem file-api, e era ai' que o autor queria escolher biblioteca.
    #[test]
    fn targets_lidos_da_fonte_quando_nao_houve_configure() {
        let root = temp_root("targets-fonte");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.24)\n\
             project(demo CXX)\n\
             add_executable(app main.cpp)   # o alvo principal\n\
             add_library(core STATIC core.cpp)\n\
             add_library(core_alias ALIAS core)\n\
             add_executable(${GERADO} gerado.cpp)\n\
             # add_executable(comentado x.cpp)\n",
        )
        .unwrap();

        let lidos = targets_from_source(&root);
        let nomes: Vec<_> = lidos.iter().map(|t| t.name.as_str()).collect();

        assert_eq!(nomes, vec!["app", "core"], "leu: {nomes:?}");
        assert_eq!(lidos[0].kind, "executable");
        assert_eq!(lidos[1].kind, "library");
    }

    /// Diretorio de build tem `CMakeLists.txt` gerado; entrar nele traria
    /// alvos internos do proprio `CMake` para a lista do autor.
    #[test]
    fn a_varredura_nao_entra_em_diretorio_de_build() {
        let root = temp_root("targets-fonte-build");
        std::fs::write(
            root.join("CMakeLists.txt"),
            "add_executable(app main.cpp)\n",
        )
        .unwrap();
        for ignorado in [".kinein", "build", "target", ".git"] {
            let pasta = root.join(ignorado);
            std::fs::create_dir_all(&pasta).unwrap();
            std::fs::write(
                pasta.join("CMakeLists.txt"),
                "add_executable(interno x.cpp)\n",
            )
            .unwrap();
        }

        let nomes: Vec<_> = targets_from_source(&root)
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(
            nomes,
            vec!["app".to_owned()],
            "leu de um diretorio ignorado"
        );
    }

    #[test]
    fn targets_come_from_the_file_api_reply() {
        let root = temp_root("targets");
        let reply = root.join(".kinein/build/.cmake/api/v1/reply");
        std::fs::create_dir_all(&reply).unwrap();
        std::fs::write(
            reply.join("target-demo.json"),
            r#"{ "type": "EXECUTABLE" }"#,
        )
        .unwrap();
        std::fs::write(
            reply.join("target-util.json"),
            r#"{ "type": "STATIC_LIBRARY" }"#,
        )
        .unwrap();
        std::fs::write(
            reply.join("codemodel-v2-abc.json"),
            r#"{ "configurations": [ { "targets": [
                { "name": "demo", "jsonFile": "target-demo.json" },
                { "name": "util", "jsonFile": "target-util.json" }
            ] } ] }"#,
        )
        .unwrap();

        let targets = list_targets(&root);
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].name, "demo");
        assert_eq!(targets[0].kind, "executable");
        assert_eq!(targets[1].kind, "staticLibrary");

        let empty = temp_root("targets-empty");
        assert!(list_targets(&empty).is_empty());
    }
}
