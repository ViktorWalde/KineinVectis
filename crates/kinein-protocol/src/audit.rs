//! Auditoria de seguranca (`audit.run`, L2 fatia 5).
//!
//! A auditoria e um JOB, como o resto do L2: `audit.run` aceita o workspace
//! aberto e o resultado chega por `event.audit.finished`, com os achados
//! saindo antes por `event.audit.diagnostic` na aba Problemas.
//!
//! O que distingue esta fatia das outras do L2 e a REDE. O `cargo-audit`
//! consulta a base de advisories do `RustSec`, e ate aqui a Kinein nunca acessou
//! a rede em runtime — toda integracao roda ferramenta local. Por decisao do
//! autor (2026-08-21) a atualizacao da base e OPT-IN EXPLICITO: sem o usuario
//! ligar, a auditoria roda com `--no-fetch` e usa a copia local, se houver.
//! Uma IDE que comeca a falar com a internet por conta propria muda de
//! categoria, e essa nao e uma mudanca que se faz por conveniencia de fatia.

use serde::{Deserialize, Serialize};

/// Parametros de `audit.run`.
///
/// Vazio de proposito, e com `deny_unknown_fields`: a auditoria e do workspace
/// aberto, e `buildSystem` nao entra porque o `cargo-audit` le o `Cargo.lock` —
/// nao existe auditoria "de `CMake`" para escolher. Campo que nasceria sempre
/// ignorado e peso morto de contrato (licao do `assistant_terminal_width`).
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AuditRunParams {}

/// Estado da base de advisories no momento da auditoria.
///
/// Vai no evento terminal para a UI poder dizer "isto e de 3 semanas atras"
/// em vez de apresentar um resultado velho como se fosse de agora. Auditoria
/// de seguranca com base desatualizada e pior que nenhuma: ela tranquiliza.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditDatabaseInfo {
    /// Quantos advisories a base conhece.
    pub advisory_count: u64,
    /// Data da ultima atualizacao da base, como a ferramenta reporta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    /// `true` quando a base NAO foi atualizada nesta execucao — o padrao, ja
    /// que buscar exige o opt-in de rede.
    pub offline: bool,
}

#[cfg(test)]
mod tests {
    use super::{AuditDatabaseInfo, AuditRunParams};

    #[test]
    fn params_recusam_campo_desconhecido() {
        // `deny_unknown_fields` e o que impede um parametro inventado pela UI
        // de ser aceito em silencio e nunca fazer nada.
        assert!(serde_json::from_str::<AuditRunParams>("{}").is_ok());
        assert!(serde_json::from_str::<AuditRunParams>(r#"{"buildSystem":"cargo"}"#).is_err());
    }

    #[test]
    fn database_info_serializa_em_camel_case() {
        let json = serde_json::to_value(AuditDatabaseInfo {
            advisory_count: 1225,
            last_updated: Some("2026-08-21T08:28:40+02:00".to_owned()),
            offline: true,
        })
        .unwrap();
        assert_eq!(json["advisoryCount"], 1225);
        assert_eq!(json["lastUpdated"], "2026-08-21T08:28:40+02:00");
        assert_eq!(json["offline"], true);
    }
}
