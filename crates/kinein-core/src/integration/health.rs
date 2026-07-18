//! Saúde de uma integração, DERIVADA de um `ToolInfo` já produzido pelo
//! `tools.rs`. Não há detecção aqui — só a tradução do resultado da detecção
//! para o vocabulário da plataforma (roadmap 28, invariante 1).

use kinein_protocol::{IntegrationHealth, ToolInfo, ToolStatus};

/// Traduz o `ToolInfo` casado por `id` (ou a ausência dele) em saúde.
pub(super) fn from_tool_info(id: &str, info: Option<&ToolInfo>) -> IntegrationHealth {
    info.map_or_else(
        // Descritor sem ferramenta correspondente nao deveria acontecer (os
        // descritores VEM de KNOWN_TOOLS), mas nao inventamos saude: reportamos
        // ausencia com o motivo, em vez de mentir "instalado".
        || IntegrationHealth {
            id: id.to_owned(),
            installed: false,
            version: None,
            path: None,
            detail: Some("sem detecção correspondente".to_owned()),
        },
        |info| IntegrationHealth {
            id: id.to_owned(),
            installed: info.status == ToolStatus::Detected,
            version: info.version.clone(),
            path: info.path.clone(),
            detail: info.message.clone(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::from_tool_info;
    use kinein_protocol::{ToolInfo, ToolStatus};

    fn tool(status: ToolStatus) -> ToolInfo {
        ToolInfo {
            id: "cargo".to_owned(),
            display_name: "Cargo".to_owned(),
            status,
            path: Some("/usr/bin/cargo".to_owned()),
            version: Some("cargo 1.0.0".to_owned()),
            suggested_install: None,
            message: None,
        }
    }

    #[test]
    fn detected_vira_instalado_com_versao_e_caminho() {
        let h = from_tool_info("cargo", Some(&tool(ToolStatus::Detected)));
        assert!(h.installed);
        assert_eq!(h.version.as_deref(), Some("cargo 1.0.0"));
        assert_eq!(h.path.as_deref(), Some("/usr/bin/cargo"));
    }

    #[test]
    fn qualquer_status_diferente_de_detected_e_nao_instalado() {
        for status in [
            ToolStatus::Missing,
            ToolStatus::Failed,
            ToolStatus::NotConfigured,
        ] {
            assert!(!from_tool_info("cargo", Some(&tool(status))).installed);
        }
    }

    #[test]
    fn sem_deteccao_reporta_ausencia_com_motivo() {
        let h = from_tool_info("cargo", None);
        assert!(!h.installed);
        assert!(h.detail.is_some());
    }
}
