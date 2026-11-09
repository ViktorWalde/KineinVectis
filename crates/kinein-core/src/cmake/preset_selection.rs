//! Selecao unica de preset para configure explicito e build automatico.
#![allow(
    clippy::redundant_pub_crate,
    reason = "Keep shared preset selection internal to the core"
)]

use std::path::Path;

use crate::cmake_configuration::{active_configuration, pending};

pub(crate) type PresetChoice = (Option<String>, Option<&'static str>);

/// O preset do configure e o `presetSource`: o pedido (`"kit"`), senao o da
/// configuracao ativa (C1-20). A tentativa pendente da mesma configuracao,
/// na pasta dela, repete-se como `"incompleteConfigure"`; a de outro preset
/// que divide a pasta perde para a configuracao ativa.
pub(crate) fn choose_preset(root: &Path, request: Option<String>) -> Result<PresetChoice, String> {
    if let Some(preset) = request.filter(|p| !p.trim().is_empty()) {
        return Ok((Some(preset), Some("kit")));
    }
    let configuration = active_configuration(root).map_err(|refusal| refusal.reason())?;
    if let Some(attempt) = pending(&configuration.directory)?
        && attempt.preset == configuration.preset
    {
        return Ok((attempt.preset, Some("incompleteConfigure")));
    }
    let source = match configuration.source {
        "noPresets" => None,
        source => Some(source),
    };
    Ok((configuration.preset, source))
}

#[cfg(test)]
mod tests {
    use super::choose_preset;
    use crate::cmake::{
        active_kit, begin_configure, complete_configure, record_preset, select_configuration,
        selected_configuration,
    };

    /// Dois presets com o mesmo `binaryDir`: a tentativa pendente de um nao
    /// decide o preset nem o kit do outro.
    #[test]
    fn an_attempt_of_another_preset_in_a_shared_folder_is_not_repeated() {
        let root = std::env::temp_dir()
            .join("kinein-preset-selection")
            .join(format!("{} pasta comum", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("CMakePresets.json"),
            r#"{"version":3,"configurePresets":[
                {"name":"a","binaryDir":"${sourceDir}/build"},
                {"name":"b","binaryDir":"${sourceDir}/build"}]}"#,
        )
        .unwrap();
        let shared = root.join("build");
        // Enquanto `a` configura, `b` recusa na trava da pasta comum, sem
        // tocar o registro nem a tentativa.
        let running = begin_configure(&root, &shared, Some("a"), "kit-a").unwrap();
        assert_eq!(
            begin_configure(&root, &shared, Some("b"), "kit-b")
                .err()
                .map(|error| error.kind()),
            Some(std::io::ErrorKind::WouldBlock)
        );
        assert_eq!(selected_configuration(&root), Ok(Some("a".to_owned())));
        // O configure de `a` foi interrompido: a tentativa fica na pasta.
        drop(running);
        assert_eq!(
            choose_preset(&root, None),
            Ok((Some("a".to_owned()), Some("incompleteConfigure")))
        );
        assert_eq!(active_kit(&root), "kit-a");

        select_configuration(&root, Some("b")).unwrap();
        assert_eq!(
            choose_preset(&root, None),
            Ok((Some("b".to_owned()), Some("configuration")))
        );
        assert_eq!(active_kit(&root), "b");

        // `a` configurou a pasta com sucesso; o kit dela nao vale para `b`.
        let done = begin_configure(&root, &shared, Some("a"), "kit-a").unwrap();
        std::fs::write(shared.join("CMakeCache.txt"), "").unwrap();
        record_preset(&shared, Some("a"));
        complete_configure(done).unwrap();
        assert_eq!(active_kit(&root), "kit-a");
        select_configuration(&root, Some("b")).unwrap();
        assert_eq!(active_kit(&root), "b");
        assert_eq!(
            choose_preset(&root, None),
            Ok((Some("b".to_owned()), Some("configuration")))
        );
    }
}
