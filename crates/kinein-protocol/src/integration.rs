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

/// Escopo de um valor de configuração de integração (0.62.0).
///
/// `Workspace` sobrepõe `Global` para o mesmo par integração+chave; `reset`
/// remove a sobreposição do escopo pedido — reversível por construção: o
/// estado "sem sobreposição" é sempre alcançável de volta.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntegrationConfigScope {
    /// Vale para todo workspace do usuário.
    Global,
    /// Vale só para o workspace aberto.
    Workspace,
}

/// Um valor de configuração armazenado para uma integração.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationConfigEntry {
    /// Chave livre (contrato genérico chave→valor; a semântica é da vertical).
    pub key: String,
    /// Valor textual armazenado.
    pub value: String,
    /// De onde o valor veio.
    pub scope: IntegrationConfigScope,
}

/// Parâmetros de `integration.config.get`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntegrationConfigGetParams {
    /// Integração consultada (`id` do descritor).
    pub id: String,
}

/// Parâmetros de `integration.config.set`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntegrationConfigSetParams {
    /// Integração alvo.
    pub id: String,
    /// Chave a gravar.
    pub key: String,
    /// Valor a gravar.
    pub value: String,
    /// Escopo da gravação.
    pub scope: IntegrationConfigScope,
}

/// Parâmetros de `integration.config.reset`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IntegrationConfigResetParams {
    /// Integração alvo.
    pub id: String,
    /// Chave cuja sobreposição sai.
    pub key: String,
    /// Escopo de onde a sobreposição sai.
    pub scope: IntegrationConfigScope,
}

/// Resposta de `integration.config.get`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationConfigGetResult {
    /// Integração consultada.
    pub id: String,
    /// Valores armazenados nos dois escopos (workspace sobrepõe global).
    pub entries: Vec<IntegrationConfigEntry>,
}

/// Resposta de `integration.config.reset`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationConfigResetResult {
    /// Integração alvo.
    pub id: String,
    /// Chave pedida.
    pub key: String,
    /// `true` quando havia sobreposição e ela foi removida; `false` quando o
    /// reset foi um no-op (já estava no default) — e aí NENHUM evento é emitido.
    pub removed: bool,
}

/// O que mudou numa integração (`event.integration.changed`).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IntegrationChangeKind {
    /// A saúde (instalada/versão/caminho) mudou após uma nova detecção.
    Health,
    /// Um valor de configuração mudou (set efetivo ou reset que removeu).
    Config,
}

/// Payload de `event.integration.changed`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationChangedEvent {
    /// Integração que mudou.
    pub id: String,
    /// Que face dela mudou.
    pub kind: IntegrationChangeKind,
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
    fn config_payloads_use_camel_case_and_reject_unknown_fields() {
        use crate::{
            IntegrationChangeKind, IntegrationChangedEvent, IntegrationConfigEntry,
            IntegrationConfigResetParams, IntegrationConfigScope, IntegrationConfigSetParams,
        };
        use serde_json::json;

        let set: IntegrationConfigSetParams = serde_json::from_value(json!({
            "id": "clangd", "key": "args", "value": "--log=error", "scope": "workspace"
        }))
        .unwrap();
        assert_eq!(set.scope, IntegrationConfigScope::Workspace);

        let invalid = serde_json::from_value::<IntegrationConfigSetParams>(json!({
            "id": "clangd", "key": "args", "value": "x", "scope": "workspace", "extra": 1
        }));
        assert!(invalid.is_err());

        let reset: IntegrationConfigResetParams = serde_json::from_value(json!({
            "id": "clangd", "key": "args", "scope": "global"
        }))
        .unwrap();
        assert_eq!(reset.scope, IntegrationConfigScope::Global);

        let entry = serde_json::to_value(IntegrationConfigEntry {
            key: "args".to_owned(),
            value: "--log=error".to_owned(),
            scope: IntegrationConfigScope::Global,
        })
        .unwrap();
        assert_eq!(entry["scope"], "global");

        let event = serde_json::to_value(IntegrationChangedEvent {
            id: "clangd".to_owned(),
            kind: IntegrationChangeKind::Config,
        })
        .unwrap();
        assert_eq!(event["kind"], "config");
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
