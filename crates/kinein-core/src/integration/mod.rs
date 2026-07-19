//! Plataforma de integrações (L1): o inventário tipado das integrações e a
//! saúde de cada uma.
//!
//! v1 é READ-ONLY. Três invariantes do roadmap 28 §2 governam este domínio:
//! 1. REUSA `tools.rs`, não cria um segundo detector.
//! 2. Sem registro dinâmico: os descritores vêm de `KNOWN_TOOLS`, compilados
//!    junto — nunca código de terceiro em runtime.
//! 3. A UI é burra: lista e (depois) configura; nunca inicia processo.

pub mod config;
mod health;
mod registry;

use kinein_protocol::{IntegrationInfo, IntegrationListResult};

use crate::tools::ToolDetector;

/// Uma integração com este `id` existe no inventário compilado?
///
/// Config de id desconhecido é recusada no handler: aceitar gravaria valores
/// órfãos que nenhuma vertical lê (o typo só apareceria quando "sumisse").
#[must_use]
pub fn is_known(id: &str) -> bool {
    registry::descriptors()
        .iter()
        .any(|descriptor| descriptor.id == id)
}

/// Ids cuja SAÚDE derivada mudou entre duas detecções.
///
/// Compara a `IntegrationHealth` derivada (o mesmo `from_tool_info` do
/// `integration.list`), não o `ToolInfo` cru — só o que a UI enxerga conta.
/// `old = None` (primeira detecção do processo) devolve vazio de propósito:
/// "passou a existir" não é "mudou", e emitiria um evento por ferramenta em
/// todo boot.
#[must_use]
pub fn health_changes(
    old: Option<&[kinein_protocol::ToolInfo]>,
    new: &[kinein_protocol::ToolInfo],
) -> Vec<String> {
    let Some(old) = old else {
        return Vec::new();
    };
    registry::descriptors()
        .into_iter()
        .filter_map(|descriptor| {
            let id = descriptor.id;
            let before = health::from_tool_info(&id, old.iter().find(|tool| tool.id == id));
            let after = health::from_tool_info(&id, new.iter().find(|tool| tool.id == id));
            (before != after).then_some(id)
        })
        .collect()
}

/// Lista toda integração conhecida com a sua saúde atual.
///
/// A detecção acontece UMA vez (`detect_all`) e é casada por `id` com cada
/// descritor — nenhum probe é executado duas vezes, e não há detector próprio.
#[must_use]
pub fn list(detector: &ToolDetector) -> IntegrationListResult {
    let detected = detector.detect_all();
    let integrations = registry::descriptors()
        .into_iter()
        .map(|descriptor| {
            let info = detected.iter().find(|tool| tool.id == descriptor.id);
            let health = health::from_tool_info(&descriptor.id, info);
            IntegrationInfo { descriptor, health }
        })
        .collect();
    IntegrationListResult { integrations }
}

#[cfg(test)]
mod tests {
    use kinein_protocol::ToolStatus;

    use crate::integration;
    use crate::tools::{KNOWN_TOOLS, ToolDetector};

    #[test]
    fn lista_uma_integracao_por_ferramenta_conhecida() {
        // Invariante 2: o inventario e' exatamente KNOWN_TOOLS, nada dinamico.
        let resultado = integration::list(&ToolDetector::from_environment());
        assert_eq!(resultado.integrations.len(), KNOWN_TOOLS.len());
        for info in &resultado.integrations {
            assert_eq!(info.descriptor.id, info.health.id);
            assert!(!info.descriptor.capabilities.is_empty());
        }
    }

    #[test]
    fn saude_vem_da_deteccao_de_ferramentas() {
        // Invariante 1: com um PATH vazio, TODA integracao fica nao-instalada —
        // a saude e' a mesma resposta que tools.rs daria, nao um segundo
        // detector com regra propria.
        let vazio = std::env::temp_dir().join("kinein-integration-sem-nada");
        let _ = std::fs::create_dir_all(&vazio);
        let detector = ToolDetector::with_search_path(&vazio);

        let resultado = integration::list(&detector);
        assert!(!resultado.integrations.is_empty());
        for info in &resultado.integrations {
            assert!(
                !info.health.installed,
                "{} deveria faltar",
                info.descriptor.id
            );
        }

        // E o casamento por id: uma ferramenta detectada aparece instalada.
        let detectados = detector.detect_all();
        for tool in &detectados {
            let esperado = tool.status == ToolStatus::Detected;
            let achado = resultado
                .integrations
                .iter()
                .find(|i| i.descriptor.id == tool.id)
                .expect("todo tool detectado tem descritor");
            assert_eq!(achado.health.installed, esperado);
        }
    }
}
