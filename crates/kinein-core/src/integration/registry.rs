//! O registro de integrações: derivado de `KNOWN_TOOLS`.
//!
//! Roadmap 28 §2: "lê o registro existente; NÃO é registro dinâmico novo". Não
//! há descoberta em disco nem carga de plugin — a lista é compilada junto ao
//! binário. As capacidades são mapeadas do `id` da ferramenta.

use kinein_protocol::IntegrationDescriptor;

use crate::tools::KNOWN_TOOLS;

/// Um descritor por ferramenta conhecida, na mesma ordem de `KNOWN_TOOLS`.
pub(super) fn descriptors() -> Vec<IntegrationDescriptor> {
    KNOWN_TOOLS
        .iter()
        .map(|spec| IntegrationDescriptor {
            id: spec.id.to_owned(),
            display_name: spec.display_name.to_owned(),
            capabilities: vec![capability_for(spec.id).to_owned()],
        })
        .collect()
}

/// A capacidade que cada ferramenta oferece. É a resposta a "o que esta
/// integração faz por mim", e é o eixo pelo qual a UI agrupa o inventário.
fn capability_for(id: &str) -> &'static str {
    match id {
        "cargo" | "cmake" | "ninja" => "build",
        "rustc" | "clang" | "clangxx" | "gcc" | "gxx" => "compiler",
        "clangd" | "rust-analyzer" => "languageServer",
        "gdb" | "lldb" | "lldb-dap" => "debugger",
        "cppcheck" => "analyzer",
        "git" => "versionControl",
        "ripgrep" | "fd" => "search",
        "rustup" => "toolchain",
        "claude" | "codex" => "agent",
        _ => "tool",
    }
}

#[cfg(test)]
mod tests {
    use super::{capability_for, descriptors};
    use crate::tools::KNOWN_TOOLS;

    #[test]
    fn todo_tool_conhecido_ganha_um_descritor_com_capacidade() {
        let ds = descriptors();
        assert_eq!(ds.len(), KNOWN_TOOLS.len());
        // Cada capacidade e' uma so' (v1); nenhuma cai no fallback generico,
        // porque toda ferramenta de KNOWN_TOOLS tem um dominio claro — se
        // alguem adicionar uma tool nova e esquecer o mapa, este teste cai.
        for d in &ds {
            assert_eq!(d.capabilities.len(), 1);
            assert_ne!(
                capability_for(&d.id),
                "tool",
                "{} caiu no fallback: mapeie a capacidade",
                d.id
            );
        }
    }
}
