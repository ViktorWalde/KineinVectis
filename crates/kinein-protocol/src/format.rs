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
