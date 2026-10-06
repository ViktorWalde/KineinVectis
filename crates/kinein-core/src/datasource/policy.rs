//! Política antes da senha/job, compartilhada por consulta e remoção.

use std::path::Path;

use kinein_protocol::{
    DataSourceConfirmation, DataSourceOperationContext, DataSourceProfile, JsonRpcErrorCode,
    SqlImpactSeverity, SqlStatementImpact,
};

use super::confirm;

/// Uma recusa contém somente a razão pública, nunca credencial ou diagnóstico.
#[derive(Debug)]
pub struct Rejection {
    /// Código tipado, independente do texto exibido.
    pub code: JsonRpcErrorCode,
    /// Orientação fixa e pública para o autor.
    pub message: &'static str,
}

impl Rejection {
    /// A recusa de consulta preserva gravidade/alvo e contexto público.
    #[must_use]
    pub fn query_response(
        self,
        id: Option<serde_json::Value>,
        profile: &DataSourceProfile,
        statements: &[SqlStatementImpact],
        token: Option<&str>,
    ) -> kinein_protocol::JsonRpcResponse {
        kinein_protocol::JsonRpcResponse::failure(
            id,
            kinein_protocol::JsonRpcError::new(
                self.code,
                self.message,
                Some(serde_json::json!({
                    "name": profile.name, "clientContext": token,
                    "severity": super::impact::overall(statements),
                    "confirmationTarget": confirm_target(profile, statements),
                    "requiresConnection": profile.production && confirm::needs_confirmation(statements),
                })),
            ),
        )
    }
    /// Resposta pública correlacionada; senha nunca entra nos detalhes.
    #[must_use]
    pub fn response(
        self,
        id: Option<serde_json::Value>,
        name: &str,
        token: Option<&str>,
    ) -> kinein_protocol::JsonRpcResponse {
        kinein_protocol::JsonRpcResponse::failure(
            id,
            kinein_protocol::JsonRpcError::new(
                self.code,
                self.message,
                Some(serde_json::json!({ "name": name, "clientContext": token })),
            ),
        )
    }
}

/// Confere a intenção da UI contra o destino que o core usaria agora.
///
/// # Errors
/// Projeto/perfil mudou ou o identificador público não tem formato válido.
pub fn check_context(
    root: &Path,
    profile: &DataSourceProfile,
    expected: Option<&DataSourceOperationContext>,
    token: Option<&str>,
) -> Result<(), Rejection> {
    if token.is_some_and(|token| {
        token.is_empty()
            || token.len() > 128
            || !token.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'.' | b'_' | b'-')
            })
    }) {
        return Err(Rejection {
            code: JsonRpcErrorCode::InvalidParams,
            message: "O contexto da operação é inválido.",
        });
    }
    if expected.is_some_and(|expected| {
        Path::new(&expected.workspace) != root || expected.profile != *profile
    }) {
        return Err(Rejection {
            code: JsonRpcErrorCode::DataSourceContextChanged,
            message: "O projeto ou a conexão mudou; peça a operação novamente.",
        });
    }
    Ok(())
}

/// Recusa no despacho e também no caminho do motor, antes de conectar.
///
/// # Errors
/// O perfil não autoriza a operação de escrita/desconhecida.
pub const fn check_read_only(profile: &DataSourceProfile, writing: bool) -> Result<(), Rejection> {
    if writing && profile.read_only {
        Err(read_only())
    } else {
        Ok(())
    }
}

/// Somente leitura recusa inclusive operação desconhecida e confirmação.
///
/// # Errors
/// Escrita proibida ou confirmação insuficiente para este perfil.
pub fn check_query(
    profile: &DataSourceProfile,
    statements: &[SqlStatementImpact],
    confirmed: bool,
    names: Option<&DataSourceConfirmation>,
) -> Result<(), Rejection> {
    let writing = statements.is_empty()
        || statements
            .iter()
            .any(|s| s.severity != SqlImpactSeverity::Read);
    check_read_only(profile, writing)?;
    let removes = confirm::needs_confirmation(statements);
    let needs_warning = removes || profile.production && writing;
    if needs_warning
        && (!confirmed
            || profile.production
                && removes
                && !names.is_some_and(|names| {
                    names.connection == profile.name
                        && names.target == confirm_target(profile, statements)
                }))
    {
        return Err(Rejection {
            code: JsonRpcErrorCode::WriteConfirmationRequired,
            message: "Confira o impacto e confirme esta operação no banco.",
        });
    }
    Ok(())
}

/// Alvo exibido no aviso, com a mesma regra para SQL e coleções `MongoDB`.
#[must_use]
pub fn confirm_target(profile: &DataSourceProfile, statements: &[SqlStatementImpact]) -> String {
    let Some(statement) = statements
        .iter()
        .find(|s| confirm::needs_confirmation(std::slice::from_ref(s)) && !s.targets.is_empty())
    else {
        return profile.name.clone();
    };
    let target = &statement.targets[0];
    if statement.kind.starts_with("mongo") || statement.kind == "dropCollection" {
        return target.clone();
    }
    // Identificadores delimitados podem conter pontos: use o último segmento léxico.
    let mut at = 0;
    let bytes = target.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'"' | b'`' | b'[') {
            let close = if byte == b'[' { b']' } else { byte };
            index = super::sql_syntax::quoted_end(bytes, index, close).0;
        } else {
            if byte == b'.' {
                at = index + 1;
            }
            index += 1;
        }
    }
    let tail = &target[at..];
    if let Some(first) = tail.as_bytes().first().copied()
        && matches!(first, b'"' | b'`' | b'[')
    {
        let close = if first == b'[' {
            ']'
        } else {
            char::from(first)
        };
        return tail[1..tail.len().saturating_sub(1)]
            .replace(&format!("{close}{close}"), &close.to_string());
    }
    tail.to_owned()
}

/// Remove dados somente quando o perfil permite e os nomes de produção conferem.
///
/// # Errors
/// Dados de perfil somente leitura ou confirmação de produção ausente.
pub fn check_destroy(
    profile: &DataSourceProfile,
    data: bool,
    names: Option<&DataSourceConfirmation>,
) -> Result<(), Rejection> {
    if !data || profile.engine == kinein_protocol::DataSourceEngine::Odbc {
        return Ok(());
    }
    if profile.read_only {
        return Err(read_only());
    }
    if profile.production
        && !names.is_some_and(|names| {
            names.connection == profile.name && names.target == profile.database
        })
    {
        return Err(Rejection {
            code: JsonRpcErrorCode::WriteConfirmationRequired,
            message: "Para apagar dados de produção, confirme o nome da conexão e do banco.",
        });
    }
    Ok(())
}

const fn read_only() -> Rejection {
    Rejection {
        code: JsonRpcErrorCode::ReadOnlyViolation,
        message: "Esta conexão está em modo somente leitura; a operação não foi enviada ao banco.",
    }
}
