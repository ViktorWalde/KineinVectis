//! Buffer formatting payloads (`format.*`).

use serde::{Deserialize, Serialize};

/// Parameters for `format.text`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FormatTextParams {
    /// Absolute path, inside the workspace, of the file the buffer belongs to.
    pub path: String,
    /// Current UTF-8 buffer content to format.
    pub text: String,
}

/// Um formatter registrado e as extensões que ele atende.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatterCapability {
    /// Identificador estável do formatter (e nome do binário no `PATH`).
    pub id: String,
    /// Extensões, minúsculas e sem ponto, que este formatter atende.
    pub extensions: Vec<String>,
}

/// Result of `format.capabilities`: o catálogo de formatters do core.
///
/// Existe para a UI **não manter uma segunda lista**. Até o protocolo 0.61.0 o
/// `EditorController` decidia sozinho o que era formatável, com duas listas
/// escritas à mão (`formattableLanguage` por linguagem e `formattablePath` por
/// extensão) que nem concordavam entre si — enquanto o core já era a autoridade
/// via `format::formatter_for_path`. Duas fontes para a mesma verdade divergem
/// por construção. Agora o core publica e a UI consome.
///
/// O catálogo é ESTÁTICO (mapa puro de extensão), então a UI pode buscá-lo uma
/// vez e guardar; ele não muda com o workspace nem com a presença do binário.
/// Formatter ausente no `PATH` continua sendo erro de `format.text`, não de
/// capacidade — são perguntas diferentes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatCapabilitiesResult {
    /// Formatters registrados, cada um com suas extensões.
    pub formatters: Vec<FormatterCapability>,
}

/// Result of `format.text`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatTextResult {
    /// Canonical path of the formatted file, echoed back so the UI can drop
    /// stale responses when the active tab changed meanwhile.
    pub path: String,
    /// Formatted buffer content (identical to the input when nothing changed).
    pub text: String,
    /// Whether the formatted text differs from the input.
    pub changed: bool,
    /// Identifier of the tool that produced the result (e.g. `rustfmt`).
    pub formatter: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{FormatTextParams, FormatTextResult};

    #[test]
    fn format_text_params_use_camel_case_and_reject_unknown_fields() {
        let parsed: FormatTextParams =
            serde_json::from_value(json!({ "path": "/w/main.rs", "text": "fn main(){}" })).unwrap();
        assert_eq!(parsed.path, "/w/main.rs");
        assert_eq!(parsed.text, "fn main(){}");

        let rejected = serde_json::from_value::<FormatTextParams>(json!({
            "path": "/w/main.rs",
            "text": "",
            "extra": true,
        }));
        assert!(rejected.is_err());
    }

    #[test]
    fn format_text_result_serializes_camel_case() {
        let value = serde_json::to_value(FormatTextResult {
            path: "/w/src/main.rs".to_owned(),
            text: "fn main() {}\n".to_owned(),
            changed: true,
            formatter: "rustfmt".to_owned(),
        })
        .unwrap();
        assert_eq!(
            value,
            json!({
                "path": "/w/src/main.rs",
                "text": "fn main() {}\n",
                "changed": true,
                "formatter": "rustfmt",
            })
        );
    }
}
