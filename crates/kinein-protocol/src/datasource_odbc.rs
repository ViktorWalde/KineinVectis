//! Descoberta ODBC e autorizacao explicita de codigo nativo (`0.157.0`).

use serde::{Deserialize, Serialize};

/// Um DSN do gerenciador; nenhum atributo de conexao ou segredo e' exposto.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceOdbcSource {
    /// Nome exato que sera' passado a `SQLConnect`.
    pub dsn: String,
    /// Driver indicado pelo gerenciador.
    pub driver: String,
    /// Identidade publica que vincula a autorizacao ao driver atual.
    pub identity: String,
}

/// Listar nao conecta nem carrega um driver.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataSourceOdbcSourcesParams {}

/// DSN existentes no usuario/sistema, sem duplicar nomes.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct DataSourceOdbcSourcesResult {
    /// Fontes conhecidas pelo gerenciador.
    pub sources: Vec<DataSourceOdbcSource>,
}

/// Gesto explicito; nao existe autorizacao salva no perfil.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceOdbcAuthorizeParams {
    /// Perfil do projeto aberto.
    pub name: String,
    /// Identidade exibida no aviso, conferida novamente no core.
    pub identity: String,
    /// Projeto que originou o aviso; resposta antiga nao autoriza o projeto novo.
    pub workspace: String,
}

/// A aprovacao vale somente enquanto a sessao e a identidade forem as mesmas.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceOdbcAuthorizeResult {
    /// Perfil autorizado.
    pub name: String,
    /// Identidade aprovada.
    pub identity: String,
    /// Projeto que recebeu a aprovacao.
    pub workspace: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn consent_requires_workspace_and_rejects_extra_authority_or_credentials() {
        let valid = json!({ "name": "outro", "identity": "hash", "workspace": "/projeto" });
        assert!(serde_json::from_value::<DataSourceOdbcAuthorizeParams>(valid.clone()).is_ok());
        for field in ["password", "allow", "driver"] {
            let mut extra = valid.clone();
            extra[field] = json!("indevido");
            assert!(serde_json::from_value::<DataSourceOdbcAuthorizeParams>(extra).is_err());
        }
        assert!(
            serde_json::from_value::<DataSourceOdbcAuthorizeParams>(
                json!({ "name": "outro", "identity": "hash" })
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<DataSourceOdbcSourcesParams>(
                json!({ "password": "indevido" })
            )
            .is_err()
        );
    }
}
