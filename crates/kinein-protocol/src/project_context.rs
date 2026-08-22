//! Contexto de compilacao efetivo de um arquivo (`project.fileContext`, L3).
//!
//! Responde a pergunta que separa uma IDE de um editor com realce: **como
//! ESTE arquivo e compilado, de verdade?** Qual target o inclui, qual
//! compilador, qual padrao, quais defines e includes — e, quando a resposta e
//! aproximada, DIZER que e aproximada em vez de apresentar chute como fato.
//!
//! O alvo desenhado esta na secao 10.2 do roadmap do motor semantico
//! ("Painel Effective Compile Context"). Esta e a v1 dele.

use serde::{Deserialize, Serialize};

/// De onde veio o comando de compilacao mostrado.
///
/// A distincao e o coracao desta fatia. Header normalmente NAO tem entrada
/// propria no banco de comandos, e o `clangd` empresta a de outra unidade por
/// heuristica — que pode escolher errado. Uma IDE que mostra o contexto
/// emprestado como se fosse o do arquivo mente sobre a propria confianca.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandOrigin {
    /// O arquivo tem entrada PROPRIA no banco de comandos.
    Exact,
    /// A entrada veio de outra unidade de traducao (tipico de header).
    Borrowed,
    /// Nao ha banco de comandos, ou nada nele serve para este arquivo.
    None,
}

/// Parametros de `project.fileContext`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileContextParams {
    /// Caminho do arquivo, absoluto ou relativo a raiz do workspace.
    pub path: String,
}

/// Contexto de compilacao efetivo de um arquivo.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContextResult {
    /// Caminho consultado, relativo a raiz quando possivel.
    pub path: String,
    /// De onde veio o comando: proprio, emprestado, ou nenhum.
    pub origin: CommandOrigin,
    /// Arquivo que emprestou o comando, quando `origin` e `borrowed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub borrowed_from: Option<String>,
    /// Target do build system que inclui este arquivo, quando conhecido.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Compilador efetivo (primeiro token do comando).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    /// Padrao da linguagem (`-std=`), como escrito no comando.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<String>,
    /// Macros definidas (`-D`), sem o prefixo.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub defines: Vec<String>,
    /// Diretorios de include (`-I`, `-isystem`), na ordem do comando.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    /// Demais flags, na ordem do comando.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<String>,
    /// Banco de comandos usado, relativo a raiz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    /// Comando completo, para o usuario copiar ou reexecutar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Explicacao curta quando nao ha contexto — o gesto que falta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{CommandOrigin, FileContextParams, FileContextResult};

    #[test]
    fn params_recusam_campo_desconhecido() {
        assert!(serde_json::from_str::<FileContextParams>(r#"{"path":"a.cpp"}"#).is_ok());
        assert!(
            serde_json::from_str::<FileContextParams>(r#"{"path":"a.cpp","target":"x"}"#).is_err()
        );
    }

    #[test]
    fn origem_serializa_em_camel_case() {
        assert_eq!(
            serde_json::to_value(CommandOrigin::Borrowed).unwrap(),
            serde_json::json!("borrowed")
        );
        assert_eq!(
            serde_json::to_value(CommandOrigin::Exact).unwrap(),
            serde_json::json!("exact")
        );
    }

    #[test]
    fn resultado_vazio_nao_polui_o_payload() {
        // Listas vazias e opcionais ausentes somem: a UI distingue "nao ha
        // defines" de "nao ha contexto" pela ORIGEM, nao por campo vazio.
        let json = serde_json::to_value(FileContextResult {
            path: "src/a.cpp".to_owned(),
            origin: CommandOrigin::None,
            borrowed_from: None,
            target: None,
            compiler: None,
            standard: None,
            defines: Vec::new(),
            includes: Vec::new(),
            flags: Vec::new(),
            database: None,
            command: None,
            note: Some("configure o CMake".to_owned()),
        })
        .unwrap();
        assert_eq!(json["origin"], "none");
        assert_eq!(json["note"], "configure o CMake");
        assert!(json.get("defines").is_none());
        assert!(json.get("compiler").is_none());
    }
}
