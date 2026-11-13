//! Descoberta e diagnóstico da compilation database do C/C++.
//!
//! # Por que este módulo não é do domínio `cmake`
//!
//! A pergunta que ele responde é *"este workspace tem uma compilation database
//! utilizável, onde, e ela está velha?"* — e isso vale para `CMake`, Meson, `bear`
//! sobre um Makefile ou uma CDB escrita à mão. Pendurar no `cmake` obrigaria um
//! ramo por build system, que é exatamente o que o `tools.rs` evita: **detectar
//! é agnóstico.**
//!
//! # O que o clangd já faz sozinho — medido na fonte (2026-08-30)
//!
//! O clangd procura `compile_commands.json` **nos diretórios pai do arquivo
//! editado e em subdiretórios `build/`**: ao editar `$SRC/gui/window.cpp` ele
//! busca em `$SRC/gui/`, `$SRC/gui/build/`, `$SRC/`, `$SRC/build/`, …
//! (<https://clangd.llvm.org/installation>).
//!
//! **A consequência é que este módulo NÃO existe para apontar o clangd.** Um
//! projeto Meson com `build/compile_commands.json` já funciona hoje, sem uma
//! linha nossa. Ele existe para o que o clangd não faz: **dizer ao usuário** se
//! há CDB, de onde ela veio e se envelheceu. Sem isso, um projeto sem CDB dá
//! erro de include em tudo e a IDE fica muda.
//!
//! # Qual CDB vale (C-05, consolidação C §16, 2026-11-09)
//!
//! Este módulo é o ÚNICO dono da resposta: contexto, verificador, tidy e o
//! `--compile-commands-dir` do clangd (`lsp/server.rs`) leem a mesma CDB. Até
//! 2026-11-09 a lista era fixa (`.kinein/build`, raiz, `build`, `builddir`,
//! `out/build`): a CDB de um preset configurado fora da IDE, em
//! `build/<preset>/`, não era achada, e a de `builddir/` era mostrada sem que o
//! clangd a recebesse. A ordem agora: o `.clangd` da raiz que fixa a base → a
//! configuração escolhida → `.kinein/build` → raiz e `build/` → a pasta dos
//! presets, o `binaryDir` ou `.kinein/presets/<preset>` (uma: ela; várias: a
//! pessoa escolhe) → `builddir/` e `out/build/`. Desde o C1-20 a configuração
//! escolhida é a ativa, e a CDB dela é a da pasta dela (consolidação C §17).

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use kinein_protocol::{CdbAbsence, CdbAbsenceKind, CdbCandidate, CdbOrigin};

use crate::cmake_presets::PresetDirectory;
// O rotulo mora na folha `cmake_configuration`: `run` e `cmake` o usam sem
// depender da CDB.
pub use crate::cmake_configuration::label;

mod clangd;
mod directory;
pub(crate) mod verification;

/// An exact, unambiguous unit and the CDB bytes from which it was derived.
pub(crate) struct VerificationContext {
    pub unit: kinein_protocol::CompileUnit,
    pub cdb: PathBuf,
    pub bytes: Vec<u8>,
}

/// Nome do arquivo, fixado pela especificação da JSON Compilation Database.
const FILE: &str = "compile_commands.json";

/// O build dir da IDE: a única CDB que sabemos casada com o configure dela.
const IDE_DIRECTORY: &str = ".kinein/build";

/// Pastas que o clangd também acha sozinho a partir de um arquivo da raiz;
/// `""` é a própria raiz (muita gente cria ali um symlink para o clangd achar).
const SEARCHED_BY_CLANGD: [&str; 2] = ["", "build"];

/// Convenções de build system que o clangd não procura (`builddir` é o nome
/// comum do Meson).
const CONVENTIONS: [&str; 2] = ["builddir", "out/build"];

/// Fontes que podem invalidar uma CDB: se alguma for mais nova que ela, as
/// flags no disco descrevem um projeto que não existe mais.
const INVALIDATING_SOURCES: &[&str] = &[
    "CMakeLists.txt",
    "CMakePresets.json",
    "CMakeUserPresets.json",
    "meson.build",
    "Makefile",
];

/// Diagnóstico da compilation database de um workspace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CdbStatus {
    /// Diretório da CDB em uso, relativo ao root (`"."` é o root) ou absoluto
    /// fora dele; `None` quando não há.
    pub directory: Option<String>,
    /// A CDB é mais velha que algum arquivo de build que a define.
    pub stale: bool,
    /// Motivo: arquivo de build mais novo ou configure da IDE incompleto.
    pub stale_because: Option<String>,
    /// A regra que escolheu a CDB em uso.
    pub origin: Option<CdbOrigin>,
    /// O preset cujo `binaryDir` contém a CDB em uso.
    pub preset: Option<String>,
    /// Toda pasta com CDB achada, na ordem de precedência.
    pub candidates: Vec<CdbCandidate>,
    /// Por que não há CDB em uso apesar da evidência de uma; com vários
    /// presets com CDB e nenhuma escolha, a IDE pergunta em vez de escolher.
    pub absence: Option<CdbAbsence>,
    /// O preset de `.kinein/configuration.json`.
    pub selected: Option<String>,
    /// O `.clangd` da raiz decide a base: a IDE não passa
    /// `--compile-commands-dir`, que o sobreporia.
    pub clangd_decides: bool,
}

impl CdbStatus {
    /// Não há compilation database alcançável neste workspace.
    #[must_use]
    pub const fn ausente() -> Self {
        Self {
            directory: None,
            stale: false,
            stale_because: None,
            origin: None,
            preset: None,
            candidates: Vec::new(),
            absence: None,
            selected: None,
            clangd_decides: false,
        }
    }

    /// O `compile_commands.json` em uso.
    #[must_use]
    pub fn file(&self, root: &Path) -> Option<PathBuf> {
        self.directory
            .as_deref()
            .map(|directory| directory_path(root, directory).join(FILE))
    }

    /// A pasta que a IDE passa ao clangd: a da CDB em uso, salvo quando o
    /// `.clangd` decide.
    #[must_use]
    pub fn clangd_directory(&self, root: &Path) -> Option<PathBuf> {
        if self.clangd_decides {
            return None;
        }
        self.directory
            .as_deref()
            .map(|directory| directory_path(root, directory))
    }
}

fn directory_path(root: &Path, label: &str) -> PathBuf {
    if label.is_empty() || label == "." {
        root.to_path_buf()
    } else {
        root.join(crate::platform::from_portable(label))
    }
}

/// Uma pasta com CDB e a regra que a achou.
#[derive(Debug, Clone)]
struct Found {
    directory: PathBuf,
    origin: CdbOrigin,
    preset: Option<String>,
}

/// O que a ordem decidiu.
enum Choice {
    Use(Found),
    /// Sem CDB em uso, com o motivo quando há evidência de uma.
    Absent(Option<CdbAbsence>),
}

const fn absent(kind: CdbAbsenceKind, reason: String) -> Choice {
    Choice::Absent(Some(CdbAbsence { kind, reason }))
}

/// Os arquivos da CDB que a abertura lê, para o `projectProblem` (`0.223.0`,
/// INF-18): o `.clangd` da raiz e a CDB da pasta da configuração ativa.
#[must_use]
pub fn read_files(root: &Path) -> Vec<PathBuf> {
    let mut files = vec![root.join(clangd::FILE)];
    if let Ok(directory) = crate::cmake_configuration::active_dir(root) {
        files.push(directory.join(FILE));
    }
    files
}

/// Procura a CDB e diz se ela envelheceu.
///
/// Nunca falha: um workspace sem CDB é um estado legítimo (projeto Rust, ou
/// C/C++ ainda não configurado), não um erro.
#[must_use]
pub fn status(root: &Path) -> CdbStatus {
    let presets = crate::cmake_configuration::preset_dirs(root);
    let clangd = clangd::base(root);
    let selected = crate::cmake::selected_configuration(root);
    let found = discover(root, &presets, &clangd);
    let mut status = CdbStatus {
        candidates: found
            .iter()
            .map(|found| CdbCandidate {
                directory: label(root, &found.directory),
                origin: found.origin,
                preset: found.preset.clone(),
            })
            .collect(),
        selected: selected.clone().ok().flatten(),
        clangd_decides: !matches!(clangd, clangd::Base::Unset),
        ..CdbStatus::ausente()
    };
    match choose(root, &found, &clangd, selected) {
        Choice::Absent(absence) => status.absence = absence,
        Choice::Use(found) => {
            let file = found.directory.join(FILE);
            (status.stale, status.stale_because) = stale(root, &file);
            // A tentativa pendente vale na pasta em que está (C1-20).
            if let Some(reason) = crate::cmake::incomplete_in(&found.directory) {
                status.stale = true;
                status.stale_because = Some(reason);
            }
            status.directory = Some(label(root, &found.directory));
            status.origin = Some(found.origin);
            status.preset = found.preset;
        }
    }
    status
}

/// Toda pasta com CDB, na ordem de precedência e sem repetir a mesma pasta.
fn discover(root: &Path, presets: &[PresetDirectory], clangd: &clangd::Base) -> Vec<Found> {
    let relative = |text: &str| directory_path(root, text);
    let mut ordered: Vec<(PathBuf, CdbOrigin)> = Vec::new();
    if let clangd::Base::Directory(directory) = clangd {
        ordered.push((directory.clone(), CdbOrigin::Clangd));
    }
    ordered.push((relative(IDE_DIRECTORY), CdbOrigin::Ide));
    ordered.extend(
        SEARCHED_BY_CLANGD
            .iter()
            .map(|text| (relative(text), CdbOrigin::Directory)),
    );
    ordered.extend(
        presets
            .iter()
            .map(|preset| (preset.directory.clone(), CdbOrigin::Preset)),
    );
    ordered.extend(
        CONVENTIONS
            .iter()
            .map(|text| (relative(text), CdbOrigin::Directory)),
    );
    let mut found: Vec<Found> = Vec::new();
    for (directory, origin) in ordered {
        if !directory.join(FILE).is_file()
            || found
                .iter()
                .any(|seen| same_directory(&seen.directory, &directory))
        {
            continue;
        }
        let preset = presets
            .iter()
            .find(|preset| same_directory(&preset.directory, &directory))
            .map(|preset| preset.name.clone());
        found.push(Found {
            directory,
            origin,
            preset,
        });
    }
    found
}

fn choose(
    root: &Path,
    found: &[Found],
    clangd: &clangd::Base,
    selected: Result<Option<String>, String>,
) -> Choice {
    let with_cdb = |directory: &Path| find(found, directory);
    match clangd {
        clangd::Base::Unset => {}
        clangd::Base::Directory(directory) => {
            return with_cdb(directory).cloned().map_or_else(
                || {
                    absent(
                        CdbAbsenceKind::Clangd,
                        format!(
                            "o .clangd da raiz fixa a base em {}, sem compile_commands.json",
                            label(root, directory)
                        ),
                    )
                },
                |found| {
                    Choice::Use(Found {
                        origin: CdbOrigin::Clangd,
                        ..found
                    })
                },
            );
        }
        clangd::Base::Manual(reason) => {
            return absent(
                CdbAbsenceKind::Clangd,
                format!(
                    "o .clangd da raiz decide a base ({reason}); a IDE não a interpreta nem a sobrepõe"
                ),
            );
        }
    }
    let ide = root.join(IDE_DIRECTORY);
    match selected {
        Err(reason) => return absent(CdbAbsenceKind::ChosenConfiguration, reason),
        Ok(Some(preset)) => return chosen(root, &preset, found),
        Ok(None) => {}
    }
    if let Some(found) = with_cdb(&ide) {
        return Choice::Use(found.clone());
    }
    if let Some(found) = SEARCHED_BY_CLANGD
        .iter()
        .find_map(|text| with_cdb(&directory_path(root, text)))
    {
        return Choice::Use(found.clone());
    }
    let from_presets: Vec<&Found> = found
        .iter()
        .filter(|found| found.origin == CdbOrigin::Preset)
        .collect();
    match from_presets.as_slice() {
        [] => {}
        [only] => return Choice::Use((*only).clone()),
        several => {
            let names: Vec<&str> = several
                .iter()
                .filter_map(|found| found.preset.as_deref())
                .collect();
            return absent(
                CdbAbsenceKind::ChoiceRequired,
                format!(
                    "{} configurações de presets têm compile_commands.json ({}): escolha uma",
                    names.len(),
                    names.join(", ")
                ),
            );
        }
    }
    CONVENTIONS
        .iter()
        .find_map(|text| with_cdb(&directory_path(root, text)))
        .map_or(Choice::Absent(None), |found| Choice::Use(found.clone()))
}

/// A configuração escolhida: a CDB da pasta dela (C1-20). Até 2026-11-09 o
/// `.kinein/build` configurado pela IDE com o mesmo preset vencia o
/// `binaryDir`; esse `.kinein/build` agora só vale no automático. Sem CDB na
/// pasta, nenhuma outra a substitui.
fn chosen(root: &Path, preset: &str, found: &[Found]) -> Choice {
    let with_cdb = |directory: &Path| find(found, directory);
    let listed = match crate::cmake::list_presets(root) {
        Ok(presets) => presets.iter().any(|entry| entry.name == preset),
        Err(reason) => return absent(CdbAbsenceKind::ChosenConfiguration, reason),
    };
    if !listed {
        return absent(
            CdbAbsenceKind::ChosenConfiguration,
            format!(
                "a configuração escolhida (preset {preset}) não existe mais entre os presets: \
                 escolha outra"
            ),
        );
    }
    match crate::cmake::configuration_dir(root, Some(preset)) {
        Ok(directory) => with_cdb(&directory).map_or_else(
            || {
                absent(
                    CdbAbsenceKind::ChosenConfiguration,
                    format!(
                        "a configuração escolhida (preset {preset}) não tem compile_commands.json \
                         em {}: configure",
                        label(root, &directory)
                    ),
                )
            },
            |found| {
                Choice::Use(Found {
                    origin: CdbOrigin::Configuration,
                    preset: Some(preset.to_owned()),
                    ..found.clone()
                })
            },
        ),
        Err(reason) => absent(
            CdbAbsenceKind::ChosenConfiguration,
            format!("a configuração escolhida (preset {preset}) não tem pasta de build: {reason}"),
        ),
    }
}

fn find<'a>(found: &'a [Found], directory: &Path) -> Option<&'a Found> {
    found
        .iter()
        .find(|found| same_directory(&found.directory, directory))
}

/// Mesma pasta pelo caminho real; sem resolver, pela grafia.
fn same_directory(left: &Path, right: &Path) -> bool {
    left == right || crate::platform::same_file(left, right) == Some(true)
}

/// A CDB é mais velha que alguma fonte que a define?
///
/// Compara mtime. Se o mtime da CDB não puder ser lido, a resposta é "não
/// envelheceu": **um diagnóstico que não sabe não deve gritar.** Aviso falso
/// ensina a ignorar aviso (`ARCHITECTURE.md` §4 regra 11).
fn stale(root: &Path, cdb: &Path) -> (bool, Option<String>) {
    let Some(written) = modified(cdb) else {
        return (false, None);
    };
    for source in INVALIDATING_SOURCES {
        let Some(changed) = modified(&root.join(source)) else {
            continue;
        };
        if changed > written {
            return (true, Some((*source).to_owned()));
        }
    }
    (false, None)
}

fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, thread, time::Duration};

    use kinein_protocol::{CdbAbsenceKind, CdbCandidate, CdbOrigin};

    use super::{CdbStatus, status};

    /// O tipo e o motivo da ausência, para comparar os dois de uma vez.
    fn absence(found: &CdbStatus) -> Option<(CdbAbsenceKind, &str)> {
        found
            .absence
            .as_ref()
            .map(|absence| (absence.kind, absence.reason.as_str()))
    }

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cdb-tests")
            .join(format!("{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_cdb(dir: &std::path::Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("compile_commands.json"), "[]\n").unwrap();
    }

    #[test]
    fn workspace_sem_cdb_nao_e_erro() {
        let root = temp_root("ausente");

        assert_eq!(status(&root), CdbStatus::ausente());
    }

    #[test]
    fn encontra_a_cdb_na_raiz_e_no_build() {
        let root = temp_root("na-raiz");
        write_cdb(&root);
        assert_eq!(status(&root).directory.as_deref(), Some("."));

        let root = temp_root("no-build");
        write_cdb(&root.join("build"));
        assert_eq!(status(&root).directory.as_deref(), Some("build"));

        // `builddir` e' nome comum de Meson e NAO e' procurado pelo clangd.
        let root = temp_root("no-builddir");
        write_cdb(&root.join("builddir"));
        assert_eq!(status(&root).directory.as_deref(), Some("builddir"));
    }

    /// A CDB da IDE vence: e' a unica que sabemos casada com o ultimo configure.
    #[test]
    fn a_cdb_da_ide_tem_precedencia_sobre_as_convencoes() {
        let root = temp_root("precedencia");
        write_cdb(&root.join("build"));
        write_cdb(&root);
        write_cdb(&root.join(".kinein").join("build"));

        assert_eq!(status(&root).directory.as_deref(), Some(".kinein/build"));
    }

    #[test]
    fn cdb_mais_velha_que_o_cmakelists_e_reportada_com_o_culpado() {
        let root = temp_root("velha");
        write_cdb(&root.join("build"));
        // mtime tem granularidade; garante ordem observavel.
        thread::sleep(Duration::from_millis(20));
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let resultado = status(&root);

        assert!(resultado.stale);
        assert_eq!(resultado.stale_because.as_deref(), Some("CMakeLists.txt"));
    }

    #[test]
    fn cdb_mais_nova_que_as_fontes_nao_e_velha() {
        let root = temp_root("nova");
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();
        thread::sleep(Duration::from_millis(20));
        write_cdb(&root.join("build"));

        let resultado = status(&root);

        assert!(!resultado.stale);
        assert_eq!(resultado.stale_because, None);
    }

    /// Sem CDB nao existe "velha": stale so faz sentido sobre algo que existe.
    #[test]
    fn ausente_nunca_e_reportada_como_velha() {
        let root = temp_root("ausente-nao-e-velha");
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let resultado = status(&root);

        assert_eq!(resultado.directory, None);
        assert!(!resultado.stale);
    }

    fn presets(root: &std::path::Path, body: &str) {
        fs::write(root.join("CMakePresets.json"), body).unwrap();
    }

    const TWO_PRESETS: &str = r#"{"version":6,"configurePresets":[
        {"name":"debug","binaryDir":"${sourceDir}/build/debug"},
        {"name":"release","binaryDir":"${sourceDir}/build/release"},
        {"name":"hidden","hidden":true,"binaryDir":"${sourceDir}/build/hidden"}]}"#;

    /// C-05: a CDB de um preset configurado fora da IDE e' achada, com origem.
    #[test]
    fn the_only_preset_with_a_database_is_used_with_its_origin() {
        let root = temp_root("preset-unico");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join("build/hidden"));

        let found = status(&root);

        assert_eq!(found.directory.as_deref(), Some("build/debug"));
        assert_eq!(found.origin, Some(CdbOrigin::Preset));
        assert_eq!(found.preset.as_deref(), Some("debug"));
        assert_eq!(absence(&found), None);
        assert_eq!(
            found.file(&root),
            Some(root.join("build/debug/compile_commands.json"))
        );
        assert_eq!(
            found.clangd_directory(&root),
            Some(root.join("build/debug"))
        );
    }

    /// Varios presets com CDB e nenhuma escolha: a IDE nao escolhe (autor,
    /// 2026-11-09), e as candidatas vao para a tela.
    #[test]
    fn several_preset_databases_require_a_choice() {
        let root = temp_root("varios-presets");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join("build/release"));

        let found = status(&root);

        assert_eq!(found.directory, None);
        assert_eq!(
            absence(&found),
            Some((
                CdbAbsenceKind::ChoiceRequired,
                "2 configurações de presets têm compile_commands.json (debug, release): escolha uma"
            ))
        );
        assert_eq!(
            found.candidates,
            [
                CdbCandidate {
                    directory: "build/debug".to_owned(),
                    origin: CdbOrigin::Preset,
                    preset: Some("debug".to_owned()),
                },
                CdbCandidate {
                    directory: "build/release".to_owned(),
                    origin: CdbOrigin::Preset,
                    preset: Some("release".to_owned()),
                },
            ]
        );
        assert_eq!(found.clangd_directory(&root), None);
    }

    #[test]
    fn the_chosen_configuration_wins_and_never_falls_back() {
        let root = temp_root("escolhida");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join("build/release"));
        write_cdb(&root.join(".kinein/build"));
        crate::cmake::select_configuration(&root, Some("release")).unwrap();

        let found = status(&root);
        assert_eq!(found.directory.as_deref(), Some("build/release"));
        assert_eq!(found.origin, Some(CdbOrigin::Configuration));
        assert_eq!(found.selected.as_deref(), Some("release"));

        fs::remove_file(root.join("build/release/compile_commands.json")).unwrap();
        let found = status(&root);
        assert_eq!(found.directory, None);
        assert_eq!(
            absence(&found),
            Some((
                CdbAbsenceKind::ChosenConfiguration,
                "a configuração escolhida (preset release) não tem compile_commands.json em \
                 build/release: configure"
            ))
        );

        presets(
            &root,
            r#"{"version":6,"configurePresets":[{"name":"debug","binaryDir":"b"}]}"#,
        );
        let found = status(&root);
        assert_eq!(found.directory, None);
        let (kind, reason) = absence(&found).unwrap();
        assert_eq!(kind, CdbAbsenceKind::ChosenConfiguration);
        assert!(
            reason.contains("não existe mais entre os presets"),
            "{reason}"
        );

        fs::write(root.join(".kinein/configuration.json"), "{").unwrap();
        let found = status(&root);
        assert_eq!(found.directory, None);
        let (kind, reason) = absence(&found).unwrap();
        assert_eq!(kind, CdbAbsenceKind::ChosenConfiguration);
        assert!(
            reason.starts_with("registro da configuração escolhida"),
            "{reason}"
        );

        crate::cmake::select_configuration(&root, None).unwrap();
        assert_eq!(status(&root).origin, Some(CdbOrigin::Ide));
    }

    /// A configuracao escolhida usa a pasta dela, mesmo com um `.kinein/build`
    /// que a IDE configurou com o mesmo preset antes do C-04 (C1-19): ate' o
    /// `0.208.0` esse `.kinein/build` vencia.
    #[test]
    fn the_chosen_configuration_uses_its_own_directory_even_beside_an_old_ide_build() {
        let root = temp_root("escolhida-ide");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        let ide = root.join(".kinein/build");
        write_cdb(&ide);
        fs::write(ide.join("CMakeCache.txt"), "").unwrap();
        crate::cmake::record_preset(&ide, Some("debug"));
        crate::cmake::select_configuration(&root, Some("debug")).unwrap();

        let found = status(&root);

        assert_eq!(found.directory.as_deref(), Some("build/debug"));
        assert_eq!(found.origin, Some(CdbOrigin::Configuration));
        assert_eq!(found.preset.as_deref(), Some("debug"));

        // No automatico, o `.kinein/build` anotado continua valendo.
        crate::cmake::select_configuration(&root, None).unwrap();
        let found = status(&root);
        assert_eq!(found.directory.as_deref(), Some(".kinein/build"));
        assert_eq!(found.origin, Some(CdbOrigin::Ide));
    }

    /// A tentativa pendente deixa obsoleta a CDB da pasta em que esta, e so'
    /// ela (C1-20).
    #[test]
    fn a_pending_attempt_marks_only_the_database_of_its_folder() {
        let root = temp_root("tentativa-pasta");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join("build/release"));
        crate::cmake::select_configuration(&root, Some("debug")).unwrap();
        let pending = |preset: &str| {
            fs::write(
                root.join("build")
                    .join(preset)
                    .join(".kinein-configure-pending"),
                format!(r#"{{"preset":"{preset}","kit":""}}"#),
            )
            .unwrap();
        };
        pending("release");
        let found = status(&root);
        assert_eq!(found.directory.as_deref(), Some("build/debug"));
        assert!(!found.stale, "{:?}", found.stale_because);

        pending("debug");
        let found = status(&root);
        assert!(found.stale);
        assert_eq!(
            found.stale_because.as_deref(),
            Some("configure não concluído (preset debug); repita ou escolha outra configuração")
        );
    }

    /// A raiz e `build/` vem antes dos presets: o clangd as acharia sozinho.
    #[test]
    fn databases_clangd_finds_alone_come_before_preset_directories() {
        let root = temp_root("build-antes");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join("build/release"));
        write_cdb(&root.join("build"));

        let found = status(&root);

        assert_eq!(found.directory.as_deref(), Some("build"));
        assert_eq!(found.origin, Some(CdbOrigin::Directory));
        assert_eq!(absence(&found), None);
        assert_eq!(found.candidates.len(), 3);
    }

    #[test]
    fn a_conventional_directory_that_is_a_preset_binary_dir_names_the_preset() {
        let root = temp_root("builddir-preset");
        presets(
            &root,
            r#"{"version":6,"configurePresets":[{"name":"meson-like","binaryDir":"builddir"}]}"#,
        );
        write_cdb(&root.join("builddir"));

        let found = status(&root);

        assert_eq!(found.directory.as_deref(), Some("builddir"));
        assert_eq!(found.origin, Some(CdbOrigin::Preset));
        assert_eq!(found.preset.as_deref(), Some("meson-like"));
    }

    #[test]
    fn a_binary_dir_outside_the_root_is_absolute() {
        let root = temp_root("fora");
        let outside = root.with_extension("fora build");
        drop(fs::remove_dir_all(&outside));
        let body = format!(
            r#"{{"version":6,"configurePresets":[{{"name":"out","binaryDir":"{}"}}]}}"#,
            outside.display()
        );
        presets(&root, &body);
        write_cdb(&outside);

        let found = status(&root);

        assert_eq!(found.directory, Some(outside.display().to_string()));
        assert_eq!(
            found.file(&root),
            Some(outside.join("compile_commands.json"))
        );
        drop(fs::remove_dir_all(&outside));
    }

    /// O `.clangd` que fixa a base vence (autor, 2026-11-09), e a IDE nao
    /// passa a opcao que o sobreporia.
    #[test]
    fn a_root_clangd_file_that_fixes_the_base_wins() {
        let root = temp_root("clangd");
        presets(&root, TWO_PRESETS);
        write_cdb(&root.join("build/debug"));
        write_cdb(&root.join(".kinein/build"));
        crate::cmake::select_configuration(&root, Some("debug")).unwrap();
        fs::write(
            root.join(".clangd"),
            "CompileFlags:\n  CompilationDatabase: build/release\n",
        )
        .unwrap();

        let found = status(&root);
        assert_eq!(found.directory, None);
        assert!(found.clangd_decides);
        assert_eq!(
            absence(&found),
            Some((
                CdbAbsenceKind::Clangd,
                "o .clangd da raiz fixa a base em build/release, sem compile_commands.json"
            ))
        );

        write_cdb(&root.join("build/release"));
        let found = status(&root);
        assert_eq!(found.directory.as_deref(), Some("build/release"));
        assert_eq!(found.origin, Some(CdbOrigin::Clangd));
        assert_eq!(found.preset.as_deref(), Some("release"));
        assert_eq!(found.clangd_directory(&root), None);

        fs::write(
            root.join(".clangd"),
            "If:\n  PathMatch: src/.*\nCompileFlags:\n  CompilationDatabase: build/debug\n",
        )
        .unwrap();
        let found = status(&root);
        assert_eq!(found.directory, None);
        assert!(found.clangd_decides);
        let (kind, reason) = absence(&found).unwrap();
        assert_eq!(kind, CdbAbsenceKind::Clangd);
        assert!(reason.contains("fragmento condicional"), "{reason}");
    }
}
