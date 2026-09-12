//! O MODELO POR ALVO do `CMake`, lido do file-api: que arquivos cada target
//! compila, com que flags, e o que ele produz.
//!
//! E' a "CDB em memoria" do pedido do autor (2026-09-12, `roadmaps/42` §8
//! item 4), na forma que o `CMake` oferece de verdade: a resposta
//! `codemodel-v2` traz, por target, as fontes e os `compileGroups` (linguagem,
//! `languageStandard`, includes, defines, `compileCommandFragments`, sysroot),
//! os artefatos e as dependencias; a `toolchains-v1` (`CMake` >= 3.20,
//! cmake-file-api(7)) traz o compilador de cada linguagem com versao e includes
//! implicitos. Tudo LIDO do reply que o configure escreveu — a IDE nao parseia
//! `CMakeLists.txt`.
//!
//! Onde entra: `cmake.targets.list` (o alvo com fontes, artefatos, flags),
//! `index.context` (o arquivo -> os alvos que o compilam; e a unidade de
//! compilacao quando nao ha' `compile_commands.json`), e o P2 (que ELF gravar).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Um target do codemodel, com o que a IDE consome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// `name::@hash`, como o file-api identifica (para as dependencias).
    pub id: String,
    /// Nome do target.
    pub name: String,
    /// `type` cru do file-api (`EXECUTABLE`, `STATIC_LIBRARY`, `UTILITY`...).
    pub kind: String,
    /// Artefatos, absolutos (o ELF/biblioteca que o build produz).
    pub artifacts: Vec<PathBuf>,
    /// Pasta de fonte do target, absoluta.
    pub source_dir: PathBuf,
    /// Fontes, absolutas, com a marca de gerada e o grupo de compilacao.
    pub sources: Vec<Source>,
    /// Grupos de compilacao (um por combinacao de linguagem+flags).
    pub compile_groups: Vec<CompileGroup>,
    /// Nomes dos targets de que este depende (ids resolvidos).
    pub dependencies: Vec<String>,
    /// Linguagem do link (`CXX`, `C`), quando ha' link.
    pub link_language: Option<String>,
}

/// Uma fonte de um target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// Caminho absoluto.
    pub path: PathBuf,
    /// Gerada pelo build (moc, rcc, ...).
    pub generated: bool,
    /// Indice em `compile_groups`, quando a fonte e' compilada.
    pub compile_group: Option<usize>,
}

/// Um grupo de compilacao: as flags que um conjunto de fontes compartilha.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileGroup {
    /// `C`, `CXX`, `ASM`...
    pub language: String,
    /// `languageStandard.standard` (`23`, `17`), quando o file-api o traz.
    pub standard: Option<String>,
    /// Includes, absolutos, na ordem.
    pub includes: Vec<String>,
    /// Defines como `NOME` ou `NOME=valor`.
    pub defines: Vec<String>,
    /// `compileCommandFragments`, na ordem (`-std=gnu++23`, `-Wall`...).
    pub fragments: Vec<String>,
    /// `sysroot.path`, quando ha' `CMAKE_SYSROOT`.
    pub sysroot: Option<String>,
}

/// O compilador de uma linguagem, da `toolchains-v1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toolchain {
    /// `C`, `CXX`...
    pub language: String,
    /// Caminho do compilador.
    pub path: String,
    /// `GNU`, `Clang`...
    pub id: Option<String>,
    /// Versao do compilador.
    pub version: Option<String>,
    /// Includes implicitos do compilador.
    pub implicit_includes: Vec<String>,
}

/// O modelo inteiro de um build dir configurado com a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmakeModel {
    /// Build dir lido, absoluto.
    pub build_dir: PathBuf,
    /// Pasta de fonte do projeto, absoluta.
    pub source_dir: PathBuf,
    /// Targets, na ordem do codemodel.
    pub targets: Vec<Target>,
    /// Compiladores por linguagem, quando a `toolchains-v1` foi respondida.
    pub toolchains: Vec<Toolchain>,
    /// Arquivo -> indices em `targets` que o compilam.
    por_arquivo: HashMap<PathBuf, Vec<usize>>,
}

impl CmakeModel {
    /// Le o reply de `build_dir`. `None` sem reply (nunca configurado com a
    /// query) ou com codemodel ilegivel.
    #[must_use]
    pub fn load(build_dir: &Path) -> Option<Self> {
        let reply_dir = build_dir
            .join(".cmake")
            .join("api")
            .join("v1")
            .join("reply");
        let codemodel = read_json(&reply_com_prefixo(&reply_dir, "codemodel-v2")?)?;
        // `paths.source` e' obrigatorio no file-api real; um reply sem ele
        // (fixture minima) resolve os relativos contra o proprio build dir.
        let source_dir = codemodel
            .pointer("/paths/source")
            .and_then(Value::as_str)
            .map_or_else(|| build_dir.to_path_buf(), PathBuf::from);
        let build_abs = codemodel
            .pointer("/paths/build")
            .and_then(Value::as_str)
            .map_or_else(|| build_dir.to_path_buf(), PathBuf::from);
        let mut targets = Vec::new();
        for entrada in codemodel
            .pointer("/configurations/0/targets")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let Some(json_file) = entrada.get("jsonFile").and_then(Value::as_str) else {
                continue;
            };
            let Some(mut detalhe) = read_json(&reply_dir.join(json_file)) else {
                continue;
            };
            // O nome vem no detalhe E na entrada do codemodel; a entrada
            // basta quando o detalhe (uma fixture minima) nao o traz.
            if detalhe.get("name").is_none() {
                if let (Some(obj), Some(nome)) = (detalhe.as_object_mut(), entrada.get("name")) {
                    obj.insert("name".to_owned(), nome.clone());
                }
            }
            if let Some(target) = ler_target(&detalhe, &source_dir, &build_abs) {
                targets.push(target);
            }
        }
        // Dependencias vem por id; o nome e' o que a tela mostra.
        let nomes: HashMap<String, String> = targets
            .iter()
            .map(|t| (t.id.clone(), t.name.clone()))
            .collect();
        for target in &mut targets {
            target.dependencies = target
                .dependencies
                .iter()
                .filter_map(|id| nomes.get(id).cloned())
                .collect();
        }
        let mut por_arquivo: HashMap<PathBuf, Vec<usize>> = HashMap::new();
        for (i, target) in targets.iter().enumerate() {
            for fonte in &target.sources {
                por_arquivo.entry(fonte.path.clone()).or_default().push(i);
            }
        }
        let toolchains = reply_com_prefixo(&reply_dir, "toolchains-v1")
            .and_then(|p| read_json(&p))
            .map(|v| ler_toolchains(&v))
            .unwrap_or_default();
        Some(Self {
            build_dir: build_abs,
            source_dir,
            targets,
            toolchains,
            por_arquivo,
        })
    }

    /// Os targets que compilam (ou listam) `path`.
    #[must_use]
    pub fn targets_for(&self, path: &Path) -> Vec<&Target> {
        self.por_arquivo
            .get(path)
            .map(|indices| indices.iter().map(|i| &self.targets[*i]).collect())
            .unwrap_or_default()
    }

    /// O primeiro grupo de compilacao que compila `path`, com o target dono.
    #[must_use]
    pub fn compile_group_for(&self, path: &Path) -> Option<(&Target, &CompileGroup)> {
        self.targets_for(path).into_iter().find_map(|t| {
            let fonte = t.sources.iter().find(|s| s.path == path)?;
            let grupo = t.compile_groups.get(fonte.compile_group?)?;
            Some((t, grupo))
        })
    }

    /// O compilador da `toolchains-v1` para uma linguagem do file-api.
    #[must_use]
    pub fn compiler_for(&self, language: &str) -> Option<&Toolchain> {
        self.toolchains.iter().find(|t| t.language == language)
    }
}

/// O primeiro `<prefixo>-*.json` do reply.
fn reply_com_prefixo(reply_dir: &Path, prefixo: &str) -> Option<PathBuf> {
    let mut candidatos: Vec<PathBuf> = fs::read_dir(reply_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                n.starts_with(prefixo)
                    && Path::new(n)
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("json"))
            })
        })
        .collect();
    candidatos.sort();
    candidatos.pop()
}

fn read_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// `base/caminho` sem os `.` que o file-api escreve (`paths.source: "."`).
fn absoluto(base: &Path, caminho: &str) -> PathBuf {
    let p = Path::new(caminho);
    let mut saida = if p.is_absolute() {
        PathBuf::new()
    } else {
        base.to_path_buf()
    };
    for componente in p.components() {
        if componente != std::path::Component::CurDir {
            saida.push(componente);
        }
    }
    saida
}

fn strings(valor: Option<&Value>, campo: &str) -> Vec<String> {
    valor
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|v| v.get(campo).and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

fn ler_target(detalhe: &Value, source_dir: &Path, build_dir: &Path) -> Option<Target> {
    let name = detalhe.get("name").and_then(Value::as_str)?.to_owned();
    let id = detalhe
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(&name)
        .to_owned();
    let kind = detalhe
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_owned();
    let artifacts = strings(detalhe.get("artifacts"), "path")
        .iter()
        .map(|p| absoluto(build_dir, p))
        .collect();
    let target_source = detalhe
        .pointer("/paths/source")
        .and_then(Value::as_str)
        .map_or_else(|| source_dir.to_path_buf(), |p| absoluto(source_dir, p));
    let sources = detalhe
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|s| {
            Some(Source {
                path: absoluto(source_dir, s.get("path").and_then(Value::as_str)?),
                generated: s.get("isGenerated").and_then(Value::as_bool) == Some(true),
                compile_group: s
                    .get("compileGroupIndex")
                    .and_then(Value::as_u64)
                    .and_then(|i| usize::try_from(i).ok()),
            })
        })
        .collect();
    let compile_groups = detalhe
        .get("compileGroups")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|g| CompileGroup {
            language: g
                .get("language")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            standard: g
                .pointer("/languageStandard/standard")
                .and_then(Value::as_str)
                .map(str::to_owned),
            includes: strings(g.get("includes"), "path")
                .iter()
                .map(|p| absoluto(source_dir, p).display().to_string())
                .collect(),
            defines: strings(g.get("defines"), "define"),
            fragments: strings(g.get("compileCommandFragments"), "fragment"),
            sysroot: g
                .pointer("/sysroot/path")
                .and_then(Value::as_str)
                .map(str::to_owned),
        })
        .collect();
    Some(Target {
        id,
        name,
        kind,
        artifacts,
        source_dir: target_source,
        sources,
        compile_groups,
        dependencies: strings(detalhe.get("dependencies"), "id"),
        link_language: detalhe
            .pointer("/link/language")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn ler_toolchains(valor: &Value) -> Vec<Toolchain> {
    valor
        .get("toolchains")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| {
            Some(Toolchain {
                language: t.get("language").and_then(Value::as_str)?.to_owned(),
                path: t
                    .pointer("/compiler/path")
                    .and_then(Value::as_str)?
                    .to_owned(),
                id: t
                    .pointer("/compiler/id")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                version: t
                    .pointer("/compiler/version")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                implicit_includes: t
                    .pointer("/compiler/implicit/includeDirectories")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            })
        })
        .collect()
}
