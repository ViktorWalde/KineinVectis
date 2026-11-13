//! As duas acoes que olham (e limpam) a pasta de build da configuracao ativa.
//!
//! Desde o C1-21 a pasta e' a da configuracao ativa
//! ([`crate::cmake::active_dir`]), a mesma de `cmake.configure`, `build.run` e
//! `run.start`: `.kinein/build`, `.kinein/presets/<preset>` ou o `binaryDir`
//! de um preset. Nenhuma acao daqui aceita caminho do cliente. Uma pasta da
//! IDE e' removida inteira, como antes; a de um preset, so' quando o cache
//! dela e' deste projeto e ela nao e' a raiz nem a contem (decisao do autor
//! de 2026-11-09).

use std::{
    fs,
    path::{Path, PathBuf},
};

use super::error::ConfigActionError;
use super::plan::ActionPlan;
use crate::{cdb, cmake};

/// Maximo de entradas do cache repassadas a UI por preview.
const MAX_CACHE_ENTRIES: usize = 200;

/// Entradas do cache que so poluem a leitura humana.
const INTERNAL_TYPE: &str = "INTERNAL";

/// O configure ja rodou na pasta da configuracao ativa?
#[must_use]
pub(super) fn is_configured(root: &Path) -> bool {
    cmake::status(root).configured
}

/// A pasta da configuracao ativa, ou a recusa com o motivo do configure.
fn active_dir(root: &Path) -> Result<PathBuf, ConfigActionError> {
    cmake::active_dir(root).map_err(|reason| ConfigActionError::NotApplicable { reason })
}

/// Le `CMakeCache.txt` da pasta da configuracao ativa e resume para leitura
/// humana.
pub(super) fn inspect_cache(root: &Path) -> Result<ActionPlan, ConfigActionError> {
    let directory = active_dir(root)?;
    let cache = directory.join("CMakeCache.txt");
    let Ok(body) = fs::read_to_string(&cache) else {
        return Err(ConfigActionError::NotApplicable {
            reason: format!(
                "o CMake ainda não foi configurado em {} (CMakeCache.txt não existe)",
                cdb::label(root, &directory)
            ),
        });
    };

    let mut entries = Vec::new();
    let mut hidden = 0_usize;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }
        let Some((declaration, _)) = trimmed.split_once('=') else {
            continue;
        };
        let kind = declaration.split_once(':').map(|(_, kind)| kind);
        if kind == Some(INTERNAL_TYPE) {
            hidden += 1;
            continue;
        }
        if entries.len() < MAX_CACHE_ENTRIES {
            entries.push(trimmed.to_owned());
        } else {
            hidden += 1;
        }
    }

    let total = entries.len();
    let mut plan = ActionPlan::report(
        format!("{total} entrada(s) do cache em {}", cache.display()),
        entries,
    );
    if hidden > 0 {
        plan = plan.with_note(format!(
            "{hidden} entrada(s) ocultadas: tipo {INTERNAL_TYPE} ou acima do limite \
             de {MAX_CACHE_ENTRIES}."
        ));
    }
    Ok(plan)
}

/// Diz por que a pasta da configuracao ativa merece reparo, quando merece.
///
/// So' quando a CDB em uso e' a dela: a de outra pasta, de `build/` ou do
/// `.clangd` nao se repara removendo esta (C-05 e C1-21, 2026-11-09).
#[must_use]
pub(super) fn stale_reason(root: &Path) -> Option<String> {
    let status = cdb::status(root);
    let directory = cmake::active_dir(root).ok()?;
    let in_use = status
        .file(root)
        .as_deref()
        .and_then(Path::parent)
        .is_some_and(|cdb| same_directory(cdb, &directory));
    if !status.stale || !in_use {
        return None;
    }
    let culprit = status
        .stale_because
        .as_deref()
        .unwrap_or("um arquivo de build");
    Some(format!(
        "a compilation database esta mais velha que {culprit}"
    ))
}

fn same_directory(left: &Path, right: &Path) -> bool {
    left == right || crate::platform::same_file(left, right) == Some(true)
}

/// A pasta que o reparo remove inteira: a da configuracao ativa, se existe.
/// Uma pasta da IDE, como antes; a de um preset, so' com `CMakeCache.txt`
/// cujo `CMAKE_HOME_DIRECTORY` e' a raiz, e que nao seja a raiz nem a contenha
/// (o build in-source e' recusado). No `.kinein`, sem link nem entrada que
/// nao seja pasta no caminho (INF-20). Conferido na previa e de novo ao
/// aplicar.
fn removable(root: &Path) -> Result<PathBuf, ConfigActionError> {
    let directory = active_dir(root)?;
    let refuse = |reason: String| Err(ConfigActionError::NotApplicable { reason });
    if !directory.is_dir() {
        return refuse(format!(
            "{} não existe; não há o que reparar",
            directory.display()
        ));
    }
    // INF-20: a pasta no `.kinein` passa pelo guarda dele; com
    // `.kinein/presets` ligado, o reparo apagava a pasta de fora.
    if let Some(problem) = cmake::configuration_dir_problem(&directory) {
        return refuse(problem);
    }
    if cmake::owned_by_ide(root, &directory) {
        return Ok(directory);
    }
    let preset_folder = |why: &str| {
        refuse(format!(
            "{} é a pasta de build do preset e {why}: a IDE não a remove",
            directory.display()
        ))
    };
    let (Ok(real_root), Ok(real_directory)) = (
        crate::platform::canonicalize(root),
        crate::platform::canonicalize(&directory),
    ) else {
        return preset_folder("não se resolve no disco");
    };
    if real_root.starts_with(&real_directory) {
        return preset_folder("contém a raiz do projeto (build in-source)");
    }
    match cache_home(&directory) {
        None => preset_folder("não tem CMakeCache.txt com CMAKE_HOME_DIRECTORY"),
        Some(home) if !same_directory(&home, root) => preset_folder(&format!(
            "o cache dela é de outro projeto ({})",
            home.display()
        )),
        Some(_) => Ok(directory),
    }
}

/// O `CMAKE_HOME_DIRECTORY` do cache: a pasta de fontes que o configurou.
fn cache_home(directory: &Path) -> Option<PathBuf> {
    let body = fs::read_to_string(directory.join("CMakeCache.txt")).ok()?;
    body.lines().find_map(|line| {
        let (declaration, value) = line.trim().split_once('=')?;
        (declaration.split(':').next() == Some("CMAKE_HOME_DIRECTORY"))
            .then(|| PathBuf::from(value.trim()))
    })
}

/// Planeja a remocao da pasta para que o proximo configure nasca limpo.
pub(super) fn repair_plan(root: &Path) -> Result<ActionPlan, ConfigActionError> {
    let build_dir = removable(root)?;
    let mut plan = ActionPlan::effect(format!("Remove {} inteiro", build_dir.display()));
    plan = plan.with_note(
        "O diretório de build e artefato derivado: nada de codigo-fonte mora nele. \
         Depois disso, rode 'Configurar' para reconstruir o cache e a compilation \
         database."
            .to_owned(),
    );
    if let Some(reason) = stale_reason(root) {
        plan = plan.with_note(format!("Motivo detectado: {reason}."));
    }
    Ok(plan)
}

/// Remove a pasta da configuracao ativa, com a mesma conferencia da previa.
pub(super) fn repair(root: &Path) -> Result<String, ConfigActionError> {
    let build_dir = removable(root)?;
    cmake::remove_configuration_dir(&build_dir).map_err(|source| {
        ConfigActionError::Fs(crate::fsops::FsError::Io {
            path: build_dir.display().to_string(),
            source,
        })
    })?;
    Ok(format!("{} removido", build_dir.display()))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{inspect_cache, is_configured, repair, repair_plan};

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-configaction-builddir")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        crate::platform::canonicalize(&dir).unwrap()
    }

    fn configure(root: &Path, cache: &str) {
        let build = root.join(".kinein/build");
        std::fs::create_dir_all(&build).unwrap();
        std::fs::write(build.join("CMakeCache.txt"), cache).unwrap();
    }

    #[test]
    fn inspect_refuses_before_the_first_configure() {
        let root = temp_root("sem-cache");
        assert!(!is_configured(&root));
        let error = inspect_cache(&root).unwrap_err();
        assert!(error.to_string().contains("não foi configurado"));
    }

    #[test]
    fn inspect_hides_internal_entries_and_comments() {
        let root = temp_root("cache");
        configure(
            &root,
            "# comentario\n//outro\n\nCMAKE_BUILD_TYPE:STRING=Debug\n\
             CMAKE_CACHEFILE_DIR:INTERNAL=/tmp\nCMAKE_CXX_COMPILER:FILEPATH=/usr/bin/c++\n",
        );

        let plan = inspect_cache(&root).unwrap();

        assert_eq!(
            plan.report,
            vec![
                "CMAKE_BUILD_TYPE:STRING=Debug".to_owned(),
                "CMAKE_CXX_COMPILER:FILEPATH=/usr/bin/c++".to_owned(),
            ]
        );
        assert!(plan.notes[0].contains("1 entrada(s) ocultadas"));
        assert!(plan.files.is_empty(), "inspect nunca escreve");
    }

    #[test]
    fn repair_removes_the_build_dir_and_then_refuses() {
        let root = temp_root("reparo");
        configure(&root, "CMAKE_BUILD_TYPE:STRING=Debug\n");
        assert!(repair_plan(&root).is_ok());

        let message = repair(&root).unwrap();

        // A mensagem mostra o caminho como o sistema o escreve.
        let build_dir = std::path::Path::new(".kinein").join("build");
        assert!(
            message.contains(&build_dir.display().to_string()),
            "{message}"
        );
        assert!(!root.join(".kinein/build").exists());
        assert!(repair(&root).is_err());
        assert!(repair_plan(&root).is_err());
    }
}
