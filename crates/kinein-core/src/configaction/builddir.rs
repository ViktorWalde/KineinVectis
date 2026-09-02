//! As duas acoes que olham (e limpam) o diretorio de build da IDE.
//!
//! O diretorio e o UNICO e imutavel do projeto — `<root>/.kinein/build`,
//! definido por [`crate::cmake::build_dir`] e compartilhado por `cmake.configure`,
//! `build.run` e `run.start`. Nenhuma acao daqui aceita caminho do cliente:
//! isso e o que torna "reparar build dir" uma remocao segura.

use std::{fs, path::Path};

use super::error::ConfigActionError;
use super::plan::ActionPlan;
use crate::{cdb, cmake};

/// Maximo de entradas do cache repassadas a UI por preview.
const MAX_CACHE_ENTRIES: usize = 200;

/// Entradas do cache que so poluem a leitura humana.
const INTERNAL_TYPE: &str = "INTERNAL";

/// O configure ja rodou neste workspace?
#[must_use]
pub(super) fn is_configured(root: &Path) -> bool {
    cmake::status(root).configured
}

/// Le `CMakeCache.txt` do build dir da IDE e resume para leitura humana.
pub(super) fn inspect_cache(root: &Path) -> Result<ActionPlan, ConfigActionError> {
    let cache = cmake::build_dir(root).join("CMakeCache.txt");
    let Ok(body) = fs::read_to_string(&cache) else {
        return Err(ConfigActionError::NotApplicable {
            reason: "o CMake ainda nao foi configurado por esta IDE \
                     (.kinein/build/CMakeCache.txt nao existe)"
                .to_owned(),
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

/// Diz por que o build dir merece reparo, quando merece.
#[must_use]
pub(super) fn stale_reason(root: &Path) -> Option<String> {
    let status = cdb::status(root);
    if !status.stale {
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

/// Planeja a remocao do build dir para que o proximo configure nasca limpo.
pub(super) fn repair_plan(root: &Path) -> Result<ActionPlan, ConfigActionError> {
    let build_dir = cmake::build_dir(root);
    if !build_dir.is_dir() {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{} nao existe; nao ha o que reparar", build_dir.display()),
        });
    }
    let mut plan = ActionPlan::effect(format!("Remove {} inteiro", build_dir.display()));
    plan = plan.with_note(
        "O diretorio de build e artefato derivado: nada de codigo-fonte mora nele. \
         Depois disso, rode 'Configurar' para reconstruir o cache e a compilation \
         database."
            .to_owned(),
    );
    if let Some(reason) = stale_reason(root) {
        plan = plan.with_note(format!("Motivo detectado: {reason}."));
    }
    Ok(plan)
}

/// Remove o build dir da IDE.
pub(super) fn repair(root: &Path) -> Result<String, ConfigActionError> {
    let build_dir = cmake::build_dir(root);
    if !build_dir.is_dir() {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("{} nao existe; nao ha o que reparar", build_dir.display()),
        });
    }
    fs::remove_dir_all(&build_dir).map_err(|source| {
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
        dir.canonicalize().unwrap()
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
        assert!(error.to_string().contains("nao foi configurado"));
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

        assert!(message.contains(".kinein/build"));
        assert!(!root.join(".kinein/build").exists());
        assert!(repair(&root).is_err());
        assert!(repair_plan(&root).is_err());
    }
}
