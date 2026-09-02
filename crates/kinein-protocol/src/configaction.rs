//! Configuration action payloads (`configAction.*`).
//!
//! Uma Configuration Action e uma RECEITA de configuracao de projeto aplicada
//! de forma segura: a IDE explica, mostra o diff, o usuario consente e so
//! entao o core escreve (spec 9.1 §10). O contrato reflete esse ciclo em tres
//! metodos — `list`, `preview`, `apply` — e nunca em um so.
//!
//! A lista e CONTEXTUAL, filtrada pelo build system ativo do workspace
//! (spec 9.2 §1): projeto `CMake` nao ve acao Cargo e vice-versa.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::BuildSystem;

/// Build system que uma acao configura.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigActionScope {
    /// Acao de `CMake` (`CMakeLists.txt`, `CMakePresets.json`, build dir).
    Cmake,
    /// Acao de Cargo (`Cargo.toml`, run configs).
    Cargo,
}

impl ConfigActionScope {
    /// Build system que habilita este escopo.
    #[must_use]
    pub const fn build_system(self) -> BuildSystem {
        match self {
            Self::Cmake => BuildSystem::Cmake,
            Self::Cargo => BuildSystem::Cargo,
        }
    }
}

/// Risco declarado da acao (`ARCHITECTURE.md` §7: todo comando e classificado).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigActionRisk {
    /// Nao muda arquivo do projeto.
    Low,
    /// Edita um arquivo de configuracao do projeto.
    Medium,
    /// Destroi artefato (build dir) ou muda o contrato de compilacao.
    High,
}

/// O que `configAction.apply` faz de verdade.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigActionEffect {
    /// Reescreve um arquivo de texto; o preview mostra o diff exato.
    Edit,
    /// So le: o preview E o resultado, e `apply` nao escreve nada.
    Inspect,
    /// Devolve um `jobId` e reusa um job existente do core.
    Job,
    /// Efeito nao textual dentro do core (remover build dir, salvar run config).
    Delegate,
}

/// Disponibilidade da acao no workspace atual (spec 9.2 §8).
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConfigActionState {
    /// Pode ser aplicada.
    Available,
    /// Pode ser aplicada e o estado do projeto pede por ela.
    Recommended,
    /// O build system existe, mas falta condicao (target, arquivo, configure).
    PartiallyAvailable,
    /// O efeito ja esta no projeto ou nao faz sentido aqui.
    Unavailable,
    /// O build system dela nao esta ativo neste workspace.
    HiddenByScope,
}

/// Um parametro que a acao pede antes do preview.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionParamInfo {
    /// Chave usada em `configAction.preview`/`apply`.
    pub name: String,
    /// Rotulo curto para a UI.
    pub label: String,
    /// Sem valor, a acao nao pode ser planejada.
    pub required: bool,
    /// Exemplo mostrado no campo vazio.
    pub placeholder: String,
}

/// Link de documentacao de uma acao (spec 9.2 §12).
///
/// No MVP o campo existe com `url_key` resolvido pela UI; a resolucao para URL
/// real e o Documentation Registry sao pos-MVP (spec 9.2 §28).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionDocLink {
    /// Origem (`officialDoc`, `kineinGuide`, ...).
    pub kind: String,
    /// Titulo mostrado no preview lateral.
    pub title: String,
    /// Chave estavel do comando documentado (`cmake.target_link_libraries`).
    pub url_key: String,
}

/// Uma Configuration Action anunciada por `configAction.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionInfo {
    /// Identificador estavel (`cmake.addExecutable`).
    pub id: String,
    /// Titulo curto para a lista.
    pub title: String,
    /// Descricao de UMA linha (spec 9.1 §11.3).
    pub description: String,
    /// Build system da acao.
    pub scope: ConfigActionScope,
    /// Categoria para agrupar a lista.
    pub category: String,
    /// Risco declarado.
    pub risk: ConfigActionRisk,
    /// Arquivos que a acao toca, relativos a raiz.
    pub affects: Vec<String>,
    /// O que `apply` faz.
    pub effect: ConfigActionEffect,
    /// Disponibilidade medida no workspace atual.
    pub state: ConfigActionState,
    /// Por que o estado e esse, quando nao e `available`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Parametros pedidos antes do preview.
    pub params: Vec<ConfigActionParamInfo>,
    /// Documentacao associada.
    pub docs: Vec<ConfigActionDocLink>,
}

/// Parameters for `configAction.list`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigActionListParams {
    /// Inclui as acoes escondidas pelo escopo (spec 9.2 §25, "show actions
    /// outside active scope"). Padrao: `false`.
    #[serde(default)]
    pub include_hidden_by_scope: bool,
}

/// Result payload for `configAction.list`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionListResult {
    /// Acoes visiveis, na ordem do catalogo.
    pub actions: Vec<ConfigActionInfo>,
    /// Build systems detectados no workspace (o que define o escopo).
    pub active_build_systems: Vec<BuildSystem>,
}

/// Mudanca planejada em UM arquivo.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionFileChange {
    /// Caminho relativo a raiz do workspace.
    pub path: String,
    /// Conteudo atual; ausente quando o arquivo sera CRIADO.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Conteudo que sera gravado.
    pub after: String,
}

/// Parameters for `configAction.preview`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigActionPreviewParams {
    /// Id da acao.
    pub id: String,
    /// Valores dos parametros declarados por `ConfigActionParamInfo`.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
}

/// Result payload for `configAction.preview`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionPreviewResult {
    /// Id da acao previsualizada.
    pub id: String,
    /// Titulo da acao.
    pub title: String,
    /// Uma linha dizendo o que sera feito.
    pub summary: String,
    /// Arquivos afetados, com antes e depois. Vazio para `inspect`/`job`.
    pub files: Vec<ConfigActionFileChange>,
    /// Linhas de relatorio das acoes `inspect`.
    pub report: Vec<String>,
    /// Avisos que o usuario precisa ler antes de aplicar.
    pub notes: Vec<String>,
}

/// Snapshot que precisa continuar no disco na hora de aplicar.
///
/// E a MESMA barreira do `fs.write` (`ARCHITECTURE.md` §7.1): a UI devolve o
/// `before` que viu no preview, e o core recusa a escrita se o disco mudou
/// entre o consentimento e a aplicacao.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigActionExpectedFile {
    /// Caminho relativo a raiz, como veio no preview.
    pub path: String,
    /// Conteudo visto no preview; ausente significa "o arquivo nao existia".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Parameters for `configAction.apply`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigActionApplyParams {
    /// Id da acao.
    pub id: String,
    /// Os mesmos valores usados no preview.
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    /// Snapshots do preview; lista vazia dispensa a comparacao.
    #[serde(default)]
    pub expected: Vec<ConfigActionExpectedFile>,
}

/// Result payload for `configAction.apply`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigActionApplyResult {
    /// Id da acao aplicada.
    pub id: String,
    /// Frase curta do que aconteceu, para a barra de status.
    pub message: String,
    /// Caminhos relativos efetivamente escritos.
    pub files: Vec<String>,
    /// Job criado, quando o efeito e `job`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ConfigActionApplyParams, ConfigActionEffect, ConfigActionFileChange, ConfigActionInfo,
        ConfigActionPreviewParams, ConfigActionRisk, ConfigActionScope, ConfigActionState,
    };

    #[test]
    fn action_info_serializes_camel_case_and_omits_absent_reason() {
        let value = serde_json::to_value(ConfigActionInfo {
            id: "cmake.addExecutable".to_owned(),
            title: "Adicionar executavel".to_owned(),
            description: "Cria um target de executavel.".to_owned(),
            scope: ConfigActionScope::Cmake,
            category: "CMake Basic".to_owned(),
            risk: ConfigActionRisk::Medium,
            affects: vec!["CMakeLists.txt".to_owned()],
            effect: ConfigActionEffect::Edit,
            state: ConfigActionState::Available,
            reason: None,
            params: Vec::new(),
            docs: Vec::new(),
        })
        .unwrap();

        assert_eq!(value["id"], "cmake.addExecutable");
        assert_eq!(value["scope"], "cmake");
        assert_eq!(value["state"], "available");
        assert!(value.get("reason").is_none());
    }

    #[test]
    fn file_change_omits_before_when_the_file_will_be_created() {
        let value = serde_json::to_value(ConfigActionFileChange {
            path: "CMakePresets.json".to_owned(),
            before: None,
            after: "{}\n".to_owned(),
        })
        .unwrap();

        assert!(value.get("before").is_none());
        assert_eq!(value["after"], "{}\n");
    }

    #[test]
    fn preview_params_default_to_empty_and_reject_unknown_fields() {
        let parsed: ConfigActionPreviewParams =
            serde_json::from_value(json!({ "id": "cargo.check" })).unwrap();
        assert!(parsed.params.is_empty());

        let rejected = serde_json::from_value::<ConfigActionPreviewParams>(json!({
            "id": "cargo.check",
            "extra": 1,
        }));
        assert!(rejected.is_err());
    }

    #[test]
    fn apply_params_carry_the_preview_snapshot() {
        let parsed: ConfigActionApplyParams = serde_json::from_value(json!({
            "id": "cmake.addExecutable",
            "params": { "name": "demo", "sources": "src/main.cpp" },
            "expected": [{ "path": "CMakeLists.txt", "content": "antigo\n" }],
        }))
        .unwrap();

        assert_eq!(parsed.params["name"], "demo");
        assert_eq!(parsed.expected[0].content.as_deref(), Some("antigo\n"));
    }
}
