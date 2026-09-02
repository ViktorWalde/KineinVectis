//! As duas acoes que criam configure presets em `CMakePresets.json`.
//!
//! O arquivo e JSON, entao a edicao e feita no MODELO (`serde_json`) e nao no
//! texto: acrescentar um preset a mao dentro de um array com virgulas e
//! comentarios seria adivinhacao. O preco e que o arquivo sai **reformatado**
//! (2 espacos, chaves em ordem alfabetica, que e a ordem do mapa do
//! `serde_json` sem a feature `preserve_order`). Isso e visivel no diff do
//! preview antes de qualquer escrita — e o motivo de a acao ter preview.
//!
//! `"version": 6` e o mesmo schema que o wizard de projeto da IDE ja escreve
//! (`workspace/create.rs`), e exige `CMake` >= 3.25.

use serde_json::{Map, Value, json};

use super::error::ConfigActionError;
use super::plan::{ActionPlan, PlannedFile, read_optional};

/// Arquivo editado por este modulo.
pub(super) const CMAKE_PRESETS: &str = "CMakePresets.json";

/// Versao do schema de presets escrita pela IDE.
const PRESETS_SCHEMA_VERSION: u64 = 6;

/// Um preset ja declarado neste `CMakePresets.json`?
#[must_use]
pub(super) fn has_preset(text: &str, name: &str) -> bool {
    serde_json::from_str::<Value>(text)
        .ok()
        .as_ref()
        .and_then(|value| value.get("configurePresets"))
        .and_then(Value::as_array)
        .is_some_and(|presets| {
            presets
                .iter()
                .any(|preset| preset.get("name").and_then(Value::as_str) == Some(name))
        })
}

/// Planeja a criacao de um configure preset com o `CMAKE_BUILD_TYPE` pedido.
pub(super) fn create_configure_preset(
    root: &std::path::Path,
    name: &str,
    display_name: &str,
    build_type: &str,
) -> Result<ActionPlan, ConfigActionError> {
    let before = read_optional(root, CMAKE_PRESETS)?;
    let mut document = match before.as_deref() {
        None => empty_document(),
        Some(text) => serde_json::from_str::<Value>(text).map_err(|error| {
            ConfigActionError::UnsupportedShape {
                path: CMAKE_PRESETS.to_owned(),
                reason: format!("JSON invalido ({error})"),
            }
        })?,
    };

    let Some(object) = document.as_object_mut() else {
        return Err(ConfigActionError::UnsupportedShape {
            path: CMAKE_PRESETS.to_owned(),
            reason: "a raiz do arquivo nao e um objeto JSON".to_owned(),
        });
    };
    object
        .entry("version")
        .or_insert_with(|| json!(PRESETS_SCHEMA_VERSION));

    let presets = object
        .entry("configurePresets")
        .or_insert_with(|| json!([]));
    let Some(presets) = presets.as_array_mut() else {
        return Err(ConfigActionError::UnsupportedShape {
            path: CMAKE_PRESETS.to_owned(),
            reason: "configurePresets nao e um array".to_owned(),
        });
    };
    if presets
        .iter()
        .any(|preset| preset.get("name").and_then(Value::as_str) == Some(name))
    {
        return Err(ConfigActionError::NotApplicable {
            reason: format!("o preset {name} ja existe em {CMAKE_PRESETS}"),
        });
    }
    presets.push(preset_value(name, display_name, build_type));

    let mut after = serde_json::to_string_pretty(&document).map_err(|error| {
        ConfigActionError::UnsupportedShape {
            path: CMAKE_PRESETS.to_owned(),
            reason: format!("falha serializando o resultado ({error})"),
        }
    })?;
    after.push('\n');

    let created = before.is_none();
    let mut plan = ActionPlan::edit(
        format!("Cria o configure preset '{name}' ({build_type})"),
        PlannedFile {
            path: CMAKE_PRESETS.to_owned(),
            before,
            after,
        },
    );
    if created {
        plan = plan.with_note(format!(
            "{CMAKE_PRESETS} sera criado com \"version\": {PRESETS_SCHEMA_VERSION} \
             (exige CMake 3.25 ou mais novo)."
        ));
    } else {
        plan = plan.with_note(
            "O arquivo e reescrito formatado (2 espacos, chaves em ordem alfabetica); \
             o diff acima ja mostra o resultado exato."
                .to_owned(),
        );
    }
    Ok(plan)
}

fn empty_document() -> Value {
    let mut root = Map::new();
    root.insert("version".to_owned(), json!(PRESETS_SCHEMA_VERSION));
    root.insert("configurePresets".to_owned(), json!([]));
    Value::Object(root)
}

fn preset_value(name: &str, display_name: &str, build_type: &str) -> Value {
    json!({
        "name": name,
        "displayName": display_name,
        "generator": "Ninja",
        "binaryDir": format!("${{sourceDir}}/build/{name}"),
        "cacheVariables": {
            "CMAKE_BUILD_TYPE": build_type,
            "CMAKE_EXPORT_COMPILE_COMMANDS": "ON",
        },
    })
}
