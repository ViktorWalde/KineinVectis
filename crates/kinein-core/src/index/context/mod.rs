//! O CONTEXTO DE COMPILADOR por arquivo: como cada arquivo do projeto e'
//! compilado ou executado.
//!
//! A segunda metade da exigencia do autor (2026-09-12): *"integracao profunda
//! de leitura do contexto do codigo/compilador"*. O indice diz o que ha' em
//! cada arquivo; este modulo diz COM QUE cada arquivo e' compilado:
//!
//! ```text
//! C/C++    a unidade de compilacao da compile_commands.json: compilador,
//!          diretorio, -std, -I/-isystem/-iquote, -D, saida. A CDB e' a fonte
//!          — e' a mesma que o clangd le; a IDE nao inventa flags
//! Rust     o pacote e o alvo do `cargo metadata --no-deps` que POSSUEM o
//!          arquivo: src_path exato > diretorio mais longo > lib em empate
//! Python   o interpretador na precedencia do roadmaps/29 §4.1: VIRTUAL_ENV,
//!          .venv/, venv/, poetry (`poetry env info -p`), sistema (avisando)
//! ```
//!
//! Tudo que roda processo (cargo, poetry, `python --version`) e' INJETADO por
//! [`Ferramentas`]: o teste passa `None` e nada da maquina entra.
//!
//! Um leitor por responsabilidade: [`cdb`] (a `compile_commands.json`),
//! [`cargo`] (o `cargo metadata`), `crate::python::env` (o interpretador), e o modelo
//! por alvo do `CMake` (`crate::cmake::model`, o file-api do build dir da
//! IDE: que targets compilam o arquivo, e a unidade quando nao ha' CDB). Este
//! arquivo e' o modelo e a consulta por arquivo.

mod cargo;
mod cdb;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use kinein_protocol::{CargoUnit, CompileUnit, ContextSummary, FileContext, PythonEnv};

use self::cargo::AlvoCargo;
use self::cdb::Unidade;
use crate::cmake::model::CmakeModel;

/// O que o contexto pode EXECUTAR, e so' se estiver aqui.
#[derive(Debug, Clone, Default)]
pub struct Ferramentas {
    /// O `cargo` a rodar para `metadata`; `None` = nao rodar.
    pub cargo: Option<PathBuf>,
    /// O que o resolvedor do interpretador Python pode executar e ler.
    pub python: crate::python::env::PythonTools,
}

/// O contexto carregado uma vez por workspace (e recarregado quando o build
/// muda), consultado por arquivo.
#[derive(Debug, Clone, Default)]
pub struct CompileContext {
    cdb_directory: Option<String>,
    cdb_stale_because: Option<String>,
    unidades: HashMap<PathBuf, Unidade>,
    pacotes: u64,
    alvos: Vec<AlvoCargo>,
    python: Option<PythonEnv>,
    cmake: Option<CmakeModel>,
}

impl CompileContext {
    /// Carrega o contexto de `root`.
    #[must_use]
    pub fn load(root: &Path, ferramentas: &Ferramentas) -> Self {
        let mut ctx = Self::default();
        let cdb = crate::cdb::status(root);
        if let Some(dir) = cdb.directory.as_deref() {
            let caminho = root.join(dir).join("compile_commands.json");
            if let Ok(texto) = std::fs::read_to_string(&caminho) {
                ctx.unidades = cdb::parse_cdb(&texto);
                ctx.cdb_directory = Some(dir.to_owned());
                ctx.cdb_stale_because = cdb
                    .stale_because
                    .or_else(|| cdb::cmakelists_mais_novo(root, &caminho, &ctx.unidades));
            }
        }
        if root.join("Cargo.toml").is_file() {
            if let Some(cargo) = ferramentas.cargo.as_deref() {
                if let Some((pacotes, alvos)) = cargo::cargo_metadata(cargo, root) {
                    ctx.pacotes = pacotes;
                    ctx.alvos = alvos;
                }
            }
        }
        ctx.python = crate::python::env::python_env(root, &ferramentas.python);
        ctx.cmake = CmakeModel::load(&crate::cmake::build_dir(root));
        ctx
    }

    /// Os numeros para o `index.status`.
    #[must_use]
    pub fn summary(&self) -> ContextSummary {
        ContextSummary {
            cdb_directory: self.cdb_directory.clone(),
            cdb_entries: self.unidades.len() as u64,
            cdb_stale: self.cdb_stale_because.is_some(),
            cdb_stale_because: self.cdb_stale_because.clone(),
            cargo_packages: self.pacotes,
            cargo_targets: self.alvos.len() as u64,
            cmake_targets: self.cmake.as_ref().map_or(0, |m| m.targets.len() as u64),
            python_interpreter: self.python.as_ref().map(|p| p.interpreter.clone()),
            python_origin: self.python.as_ref().map(|p| p.origin.clone()),
        }
    }

    /// Como `path` e' compilado/executado.
    #[must_use]
    pub fn for_file(&self, root: &Path, path: &Path, language: &str) -> FileContext {
        let absoluto = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let mut ctx = FileContext {
            path: absoluto.display().to_string(),
            language: language.to_owned(),
            unit: None,
            cargo: None,
            python: None,
            targets: Vec::new(),
            source: None,
            hint: None,
        };
        match language {
            "c" | "cpp" => self.contexto_c(&absoluto, &mut ctx),
            "rust" => self.contexto_rust(&absoluto, &mut ctx),
            "python" => {
                ctx.python.clone_from(&self.python);
                if let Some(p) = &self.python {
                    ctx.source = Some(format!("interpretador por {}", p.origin));
                } else {
                    ctx.hint = Some(
                        "nenhum interpretador Python resolvido: sem VIRTUAL_ENV, .venv/, venv/, \
                         poetry nem python3 no PATH"
                            .to_owned(),
                    );
                }
            }
            _ => {}
        }
        ctx
    }

    fn contexto_c(&self, absoluto: &Path, ctx: &mut FileContext) {
        let chave = std::fs::canonicalize(absoluto).unwrap_or_else(|_| absoluto.to_path_buf());
        // O inverso: que targets do CMake compilam este arquivo (file-api).
        if let Some(modelo) = &self.cmake {
            ctx.targets = modelo
                .targets_for(&chave)
                .iter()
                .map(|t| t.name.clone())
                .collect();
        }
        if let Some(u) = self
            .unidades
            .get(&chave)
            .or_else(|| self.unidades.get(absoluto))
        {
            ctx.unit = Some(CompileUnit {
                compiler: u.compiler.clone(),
                directory: u.directory.clone(),
                standard: u.standard.clone(),
                includes: u.includes.clone(),
                defines: u.defines.clone(),
                output: u.output.clone(),
                arguments: u.arguments.clone(),
            });
            ctx.source = self
                .cdb_directory
                .as_ref()
                .map(|d| format!("compile_commands.json em {d}"));
            if let Some(porque) = &self.cdb_stale_because {
                ctx.hint = Some(format!(
                    "a compile_commands.json e' mais velha que {porque}: reconfigure para as flags \
                     voltarem a valer"
                ));
            }
            return;
        }
        if self.unidade_do_file_api(&chave, ctx) {
            return;
        }
        let ext = absoluto
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        let e_cabecalho = matches!(ext.as_str(), "h" | "hh" | "hpp" | "hxx" | "ipp");
        // Cabecalho nunca tem unidade, com ou sem CDB: a dica de configurar
        // seria falsa para ele.
        ctx.hint = Some(
            match (&self.cdb_directory, e_cabecalho, &self.cdb_stale_because) {
                (_, true, _) => {
                    "cabecalho: nao tem unidade de compilacao propria; o clangd deduz as \
                                  flags pela unidade que o inclui"
                        .to_owned()
                }
                (None, false, _) => {
                    "sem compile_commands.json: configure o projeto (cmake) para a IDE saber \
                            como cada arquivo e' compilado"
                        .to_owned()
                }
                (Some(dir), false, Some(porque)) => format!(
                    "arquivo fora da compile_commands.json em {dir}, que e' mais velha que {porque}: \
                 reconfigure"
                ),
                (Some(dir), false, None) => {
                    format!("arquivo fora da compile_commands.json em {dir}: nenhum alvo o compila")
                }
            },
        );
    }

    /// Sem unidade na CDB, mas o file-api sabe como o arquivo e' compilado: a
    /// "CDB em memoria" — o grupo de compilacao do target + o compilador da
    /// `toolchains-v1`. So' vale para o que o codemodel COMPILA (cabecalho e'
    /// listado, nao compilado). `true` quando preencheu.
    fn unidade_do_file_api(&self, chave: &Path, ctx: &mut FileContext) -> bool {
        let Some(modelo) = &self.cmake else {
            return false;
        };
        let Some((target, grupo)) = modelo.compile_group_for(chave) else {
            return false;
        };
        let compilador = modelo.compiler_for(&grupo.language);
        let standard = grupo
            .fragments
            .iter()
            .find_map(|f| f.strip_prefix("-std=").map(str::to_owned))
            .or_else(|| grupo.standard.clone());
        ctx.unit = Some(CompileUnit {
            compiler: compilador.map_or_else(
                || format!("({} do kit)", grupo.language),
                |c| c.path.clone(),
            ),
            directory: modelo.build_dir.display().to_string(),
            standard,
            includes: grupo.includes.clone(),
            defines: grupo.defines.clone(),
            output: None,
            arguments: grupo.fragments.clone(),
        });
        ctx.source = Some(format!(
            "file-api codemodel-v2 (target {}), sem compile_commands.json",
            target.name
        ));
        true
    }

    fn contexto_rust(&self, absoluto: &Path, ctx: &mut FileContext) {
        let Some(alvo) = self.alvo_de(absoluto) else {
            ctx.hint = Some(if self.alvos.is_empty() {
                "sem `cargo metadata`: o cargo nao rodou (ausente, ou o Cargo.toml nao esta' na raiz)"
                    .to_owned()
            } else {
                "arquivo fora de todo alvo do cargo: nenhum `mod` chega ate' ele, ou e' de um pacote \
                 fora do workspace"
                    .to_owned()
            });
            return;
        };
        ctx.cargo = Some(CargoUnit {
            package: alvo.package.clone(),
            target: alvo.target.clone(),
            kind: alvo.kind.clone(),
            edition: alvo.edition.clone(),
            manifest: alvo.manifest.clone(),
            src_path: alvo.src_path.display().to_string(),
            features: alvo.features.clone(),
        });
        ctx.source = Some("cargo metadata --no-deps".to_owned());
    }

    /// `src_path` EXATO, senao o diretorio de `src_path` mais longo que prefixa
    /// o arquivo, e `lib` em empate (um `src/x.rs` pertence ao lib e ao bin do
    /// mesmo pacote; o lib e' o que o rust-analyzer tambem prefere). O
    /// `build.rs` (`custom-build`) so' casa exato: mora na raiz do pacote e,
    /// por prefixo, seria dono de tudo.
    fn alvo_de(&self, absoluto: &Path) -> Option<&AlvoCargo> {
        if let Some(exato) = self.alvos.iter().find(|a| a.src_path == absoluto) {
            return Some(exato);
        }
        self.alvos
            .iter()
            .filter(|a| a.kind != "custom-build")
            .filter_map(|a| {
                let dir = a.src_path.parent()?;
                absoluto
                    .starts_with(dir)
                    .then(|| (dir.components().count(), u8::from(a.kind == "lib"), a))
            })
            .max_by_key(|(profundidade, e_lib, _)| (*profundidade, *e_lib))
            .map(|(_, _, a)| a)
    }
}
