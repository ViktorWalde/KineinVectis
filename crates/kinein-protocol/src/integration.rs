//! Integration platform payloads (`integration.list`).
//!
//! Uma "integração" na Kinein é um DESCRITOR TIPADO compilado junto ao core —
//! nunca código de terceiro carregado em runtime (`ARCHITECTURE.md` §2.1, e o
//! roadmap 28 §2). Esta é a v1: o inventário READ-ONLY. Diz o que existe e se
//! cada integração está saudável; configurar e ligar/desligar vêm depois.

use serde::{Deserialize, Serialize};

/// Descrição estática de uma integração.
///
/// Derivada do registro de ferramentas já conhecido pelo core (`KNOWN_TOOLS`),
/// não de um registro dinâmico novo.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationDescriptor {
    /// Identificador estável, igual ao `id` da ferramenta (ex.: `clangd`).
    pub id: String,
    /// Nome legível exibido pela UI.
    pub display_name: String,
    /// O que a integração oferece (ex.: `build`, `languageServer`, `debugger`).
    pub capabilities: Vec<String>,
}

/// Saúde de uma integração, DERIVADA da detecção de ferramentas.
///
/// Não há detector próprio: o campo vem do `tools.rs` que já é agnóstico de
/// distro (roadmap 28, invariante 1).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationHealth {
    /// Identificador da integração a que esta saúde pertence.
    pub id: String,
    /// A ferramenta foi encontrada e respondeu ao probe de versão?
    pub installed: bool,
    /// Versão reportada, quando instalada.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Caminho absoluto do binário detectado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Explicação legível quando não está saudável ("por que não").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Um descritor com a sua saúde atual.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationInfo {
    /// A descrição estática.
    pub descriptor: IntegrationDescriptor,
    /// A saúde medida agora.
    pub health: IntegrationHealth,
}

/// Payload de resposta de `integration.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationListResult {
    /// Toda integração conhecida pelo core, com a sua saúde.
    pub integrations: Vec<IntegrationInfo>,
}

#[cfg(test)]
mod tests {
    use crate::{IntegrationDescriptor, IntegrationHealth, IntegrationInfo, IntegrationListResult};

    #[test]
    fn integration_info_serializes_camel_case_and_omits_empty_fields() {
        let info = IntegrationInfo {
            descriptor: IntegrationDescriptor {
                id: "clangd".to_owned(),
                display_name: "clangd".to_owned(),
                capabilities: vec!["languageServer".to_owned()],
            },
            health: IntegrationHealth {
                id: "clangd".to_owned(),
                installed: false,
                version: None,
                path: None,
                detail: Some("nao encontrado no PATH".to_owned()),
            },
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["descriptor"]["displayName"], "clangd");
        assert_eq!(json["health"]["installed"], false);
        // Campos vazios sao omitidos; o campo com valor aparece em camelCase.
        assert!(json["health"].get("version").is_none());
        assert_eq!(json["health"]["detail"], "nao encontrado no PATH");
    }

    #[test]
    fn list_result_round_trips() {
        let result = IntegrationListResult {
            integrations: vec![],
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: IntegrationListResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, back);
    }
}
