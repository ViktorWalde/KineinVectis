//! O `.clangd` de um projeto ESP-IDF: o fragmento que impede o include-cleaner
//! do clangd de trocar os headers do `FreeRTOS` (52 §14.41).
//!
//! O `FreeRTOS` do ESP-IDF exige `freertos/FreeRTOS.h` antes dos outros headers
//! dele (`task.h` faz `#error` sem ele), e os headers nao declaram isso ao
//! clangd. Sem o fragmento, "remove #include directive" e "fix all includes"
//! quebram o build. Medido com o esp-clangd 21.1.3 e o ESP-IDF v6.1, onde todo
//! header do `FreeRTOS` mora sob `include/freertos/`.

use std::path::Path;

use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_optional};

/// O arquivo de configuracao do clangd na raiz do projeto.
pub(super) const CLANGD: &str = ".clangd";

/// O fragmento acrescentado; o clangd le varios fragmentos separados por `---`.
const FRAGMENT: &str = r"# Kinein (ESP-IDF): o FreeRTOS exige freertos/FreeRTOS.h antes dos outros
# headers dele; sem isto o include-cleaner sugere troca-lo e o build quebra.
Diagnostics:
  Includes:
    IgnoreHeader:
      - 'freertos/.*'
";

/// O `.clangd` ja' ignora os headers do `FreeRTOS` no include-cleaner.
#[must_use]
pub(super) fn ignores_freertos(text: &str) -> bool {
    text.contains("IgnoreHeader")
        && text
            .lines()
            .any(|line| !line.trim_start().starts_with('#') && line.contains("freertos/"))
}

/// Acrescenta o fragmento ao `.clangd`, ou cria o arquivo com ele.
///
/// # Errors
/// `NotApplicable` quando o arquivo ja' ignora os headers do `FreeRTOS`, ou
/// quando nao pode ser lido.
pub(super) fn freertos_includes(root: &Path) -> Result<ActionPlan, ConfigActionError> {
    let before = read_optional(root, CLANGD)?;
    let after = match before.as_deref() {
        Some(text) if ignores_freertos(text) => {
            return Err(ConfigActionError::NotApplicable {
                reason: format!("o {CLANGD} já ignora os headers do FreeRTOS"),
            });
        }
        Some(text) if !text.trim().is_empty() => {
            let separator = if text.ends_with('\n') { "" } else { "\n" };
            format!("{text}{separator}---\n{FRAGMENT}")
        }
        _ => FRAGMENT.to_owned(),
    };
    let summary = if before.is_some() {
        format!("Acrescenta ao {CLANGD} o fragmento que mantém os headers do FreeRTOS")
    } else {
        format!("Cria o {CLANGD} com o fragmento que mantém os headers do FreeRTOS")
    };
    Ok(ActionPlan::edit(
        summary,
        PlannedFile {
            path: CLANGD.to_owned(),
            before,
            after,
        },
    )
    .with_note(
        "O clangd relê o arquivo quando o servidor reinicia: use 'LSP: Reiniciar servidor' \
         depois de aplicar."
            .to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("{}-clangd-action-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn creates_the_file_or_appends_a_separate_fragment() {
        let root = scratch("plan");
        let created = freertos_includes(&root).unwrap();
        assert_eq!(created.files[0].before, None);
        assert_eq!(created.files[0].after, FRAGMENT);
        let user = "CompileFlags:\n  CompilationDatabase: build";
        std::fs::write(root.join(CLANGD), user).unwrap();
        let appended = freertos_includes(&root).unwrap();
        let after = &appended.files[0].after;
        assert!(after.starts_with(user), "the user's text stays as it was");
        assert_eq!(&after[user.len()..], format!("\n---\n{FRAGMENT}"));
        std::fs::write(root.join(CLANGD), after).unwrap();
        assert!(matches!(
            freertos_includes(&root),
            Err(ConfigActionError::NotApplicable { .. })
        ));
    }

    #[test]
    fn only_an_active_freertos_ignore_counts_as_applied() {
        assert!(ignores_freertos(FRAGMENT));
        assert!(ignores_freertos(
            "Diagnostics:\n  Includes:\n    IgnoreHeader: [\"freertos/.*\", \"esp_.*\"]\n"
        ));
        assert!(!ignores_freertos(
            "Diagnostics:\n  UnusedIncludes: Strict\n"
        ));
        assert!(!ignores_freertos(
            "Diagnostics:\n  Includes:\n    IgnoreHeader: ['esp_.*']\n# freertos/ later\n"
        ));
    }
}
