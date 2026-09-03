//! Toolchain payloads (`toolchain.*`).
//!
//! Uma TOOLCHAIN e a resposta a uma pergunta que ate 2026-09-02 nao tinha dono
//! neste projeto: **qual executavel roda cada papel?** Compilador, gerador e os
//! proprios `cmake`/`cargo` eram "o que estiver no `PATH`", entao trocar de
//! compilador exigia editar arquivo a mao ou exportar `CC`/`CXX` antes de abrir
//! a IDE — o "kit" que `CLion` e Qt Creator expoem nao existia
//! (`roadmaps/29` §5d, B2 do TR2).
//!
//! O vocabulario de PAPEIS e fechado de proposito. Papel novo e' entrada nova
//! aqui e no catalogo do core, nao string livre vinda da UI: um papel que o
//! core nao sabe usar nao configura nada, e o usuario ficaria com um seletor
//! que nao faz efeito.

use serde::{Deserialize, Serialize};

/// Papel que um executavel cumpre na construcao do projeto.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolchainRole {
    /// Compilador C (`CMAKE_C_COMPILER`).
    CCompiler,
    /// Compilador C++ (`CMAKE_CXX_COMPILER`).
    CxxCompiler,
    /// Gerador do `CMake` (`-G`).
    Generator,
    /// O proprio `cmake`.
    Cmake,
    /// O proprio `cargo`.
    Cargo,
}

impl ToolchainRole {
    /// Chave estavel usada no arquivo e no protocolo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CCompiler => "cCompiler",
            Self::CxxCompiler => "cxxCompiler",
            Self::Generator => "generator",
            Self::Cmake => "cmake",
            Self::Cargo => "cargo",
        }
    }

    /// Todos os papeis, na ordem em que a UI os mostra.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::CCompiler,
            Self::CxxCompiler,
            Self::Generator,
            Self::Cmake,
            Self::Cargo,
        ]
    }
}

/// Um executavel que pode cumprir um papel neste computador.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainCandidate {
    /// Papel que ele cumpre.
    pub role: ToolchainRole,
    /// Identificador estavel do candidato (`clang`, `gxx`, `ninja`, ...).
    ///
    /// Para compiladores e ferramentas e o mesmo `id` do `tools.detect`; para
    /// o gerador e o nome que o `CMake` aceita em `-G`, normalizado.
    pub id: String,
    /// Rotulo curto para a UI.
    pub label: String,
    /// Caminho absoluto do binario, quando ha um.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Versao reportada pelo binario, quando o probe respondeu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// O que esta escolhido para um papel.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainSelection {
    /// Papel escolhido.
    pub role: ToolchainRole,
    /// Id do candidato escolhido; ausente e' AUTOMATICO (o que estiver no
    /// `PATH`), que e o comportamento historico e continua sendo o padrao.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Caminho que o core vai usar de fato, ja resolvido.
    ///
    /// Ausente quando a escolha e automatica e o binario nao foi encontrado —
    /// a UI mostra isso como "nao detectado" em vez de fingir que ha kit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
}

/// Result payload de todo metodo `toolchain.*`.
///
/// Os tres metodos respondem o MESMO shape, como as run configs: a UI nunca
/// calcula estado derivado nem precisa casar respostas diferentes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainResult {
    /// Kit lido/escrito; vazio = o kit padrao do workspace.
    #[serde(default)]
    pub preset: String,
    /// Raiz do sistema alvo, quando escolhida.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Triple do alvo, quando escolhido.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
    /// Arquivo de toolchain que o PRESET declara (`toolchainFile`), quando ha.
    ///
    /// E informacao, nao escolha: quem manda nele e o `CMakePresets.json`, e a
    /// IDE mostra para o usuario nao procurar no lugar errado quando o
    /// compilador efetivo nao for o que ele escolheu aqui.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_toolchain_file: Option<String>,
    /// Escolha atual de cada papel, na ordem de [`ToolchainRole::all`].
    pub selections: Vec<ToolchainSelection>,
    /// O que existe nesta maquina para cada papel.
    pub candidates: Vec<ToolchainCandidate>,
}

/// Parameters for `toolchain.get`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainGetParams {
    /// Kit to read. `None` = the workspace default kit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// Parameters for `toolchain.set`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainSetParams {
    /// Papel a mudar.
    pub role: ToolchainRole,
    /// Id do candidato; ausente volta para automatico.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Kit a mudar. `None` = o kit padrao do workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// Parameters for `toolchain.setKit` — sysroot e alvo de cross-compilacao.
///
/// Campo ausente NAO e o mesmo que campo vazio: ausente preserva o valor
/// atual, string vazia limpa. Sem isso, mexer no sysroot apagaria o target.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainSetKitParams {
    /// Kit a mudar. `None` = o kit padrao do workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// Raiz do sistema alvo (`CMAKE_SYSROOT`). `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Triple do alvo (`--target` do cargo, `CMAKE_SYSTEM_*`). `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ToolchainCandidate, ToolchainResult, ToolchainRole, ToolchainSelection, ToolchainSetParams,
    };

    #[test]
    fn roles_serialize_as_the_stable_key() {
        let value = serde_json::to_value(ToolchainRole::CxxCompiler).unwrap();
        assert_eq!(value, "cxxCompiler");
        assert_eq!(ToolchainRole::CxxCompiler.as_str(), "cxxCompiler");
        assert_eq!(ToolchainRole::all().len(), 5);
    }

    #[test]
    fn selection_omits_the_automatic_choice_and_the_missing_path() {
        let value = serde_json::to_value(ToolchainResult {
            preset: String::new(),
            sysroot: None,
            target_triple: None,
            preset_toolchain_file: None,
            selections: vec![ToolchainSelection {
                role: ToolchainRole::Cmake,
                id: None,
                resolved_path: None,
            }],
            candidates: vec![ToolchainCandidate {
                role: ToolchainRole::Cmake,
                id: "cmake".to_owned(),
                label: "CMake".to_owned(),
                path: Some("/usr/bin/cmake".to_owned()),
                version: None,
            }],
        })
        .unwrap();

        assert!(value["selections"][0].get("id").is_none());
        assert!(value["selections"][0].get("resolvedPath").is_none());
        assert_eq!(value["candidates"][0]["path"], "/usr/bin/cmake");
        assert!(value["candidates"][0].get("version").is_none());
    }

    #[test]
    fn set_params_accept_the_automatic_choice_and_reject_unknown_fields() {
        let automatico: ToolchainSetParams =
            serde_json::from_value(json!({ "role": "generator" })).unwrap();
        assert_eq!(automatico.role, ToolchainRole::Generator);
        assert_eq!(automatico.id, None);

        assert!(
            serde_json::from_value::<ToolchainSetParams>(json!({
                "role": "generator",
                "extra": 1,
            }))
            .is_err()
        );
    }
}
