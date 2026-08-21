//! Handler de `audit.run` (`impl Core`).
//!
//! Fino: valida, le o opt-in de rede, spawna o job e delega ao dominio
//! `crate::audit`. Os achados saem por `event.audit.diagnostic` e o resultado
//! por `event.audit.finished`.

use kinein_protocol::{AuditRunParams, JobAcceptedResult, JobRisk, JsonRpcResponse, ProjectKind};
use serde_json::{Value, json};

use super::build::{emit_build_event, jobs_unavailable_response, unsupported_kind_response};
use crate::rpc::parse_params;
use crate::{Core, audit, build::BuildEvent, jobs::JobOutcome};

impl Core {
    pub(crate) fn audit_run_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<AuditRunParams>(
            request_id.as_ref(),
            params,
            "audit.run nao aceita parametros",
        ) {
            return *response;
        }
        let (root, kind) = match self.runner_workspace(request_id.as_ref(), "audit.run", None) {
            Ok(context) => context,
            Err(response) => return *response,
        };
        // Auditoria de advisory le o Cargo.lock: so existe para Cargo. Nao ha
        // equivalente "de CMake" a oferecer, entao o erro e sincrono e claro.
        if kind != ProjectKind::RustCargo {
            return unsupported_kind_response(request_id, "audit", kind);
        }

        // O OPT-IN DE REDE, lido ANTES do spawn (o job roda em outra thread e
        // nao enxerga o Core). Decisao do autor em 2026-08-21: sem isto ligado
        // a IDE nao busca a base de advisories — ela usa a copia local, se
        // houver, e explica quando nao houver.
        let rede_autorizada = rede_autorizada(
            self.integration_config_value("cargo-audit", "allowNetwork")
                .as_deref(),
        );

        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "audit.run");
        };

        let job_id = jobs.spawn("audit", "Audit", JobRisk::Medium, true, move |ctx| {
            let cancel = ctx.cancellation();
            let mut on_output = |line: &str| ctx.emit_output(line);
            emit_build_event(
                ctx,
                "audit",
                &BuildEvent::Started {
                    command: if rede_autorizada {
                        "cargo audit".to_owned()
                    } else {
                        "cargo audit --no-fetch (rede nao autorizada)".to_owned()
                    },
                },
            );
            match audit::run_audit(&root, rede_autorizada, &cancel, &mut on_output) {
                Ok(report) => {
                    // O lockfile e lido UMA vez para posicionar todos os
                    // achados: abrir e reler por advisory seria IO por nada.
                    let lockfile =
                        std::fs::read_to_string(root.join("Cargo.lock")).unwrap_or_default();
                    for mut achado in report.findings {
                        achado.line = pacote_do_achado(&achado.message)
                            .and_then(|pacote| audit::linha_do_pacote(&lockfile, &pacote));
                        emit_build_event(ctx, "audit", &BuildEvent::Diagnostic(achado));
                    }
                    ctx.emit_event(
                        "event.audit.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": true,
                            "vulnerabilities": report.vulnerabilities,
                            "database": report.database,
                        }),
                    );
                    JobOutcome::Success
                }
                Err(error) => {
                    ctx.emit_event(
                        "event.audit.finished",
                        json!({
                            "jobId": ctx.id(),
                            "success": false,
                            "error": error.to_string(),
                        }),
                    );
                    JobOutcome::Failed
                }
            }
        });

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }
}

/// Interpreta o valor de `allowNetwork` da configuracao da integracao.
///
/// Funcao propria porque isto e um PORTAO, e portao se testa direto. So um
/// "true" explicito abre a rede: ausente, vazio, "false", "0" ou qualquer
/// outra coisa mantem a auditoria offline. A regra e deliberadamente
/// assimetrica — na duvida, NAO conectar. Uma config mal digitada nao pode
/// virar consentimento.
///
/// Isto existe porque a mutacao `.is_some()` SOBREVIVEU na primeira rodada de
/// provas desta fatia: nada testava a leitura do opt-in, e um `allowNetwork`
/// gravado como "false" teria ligado a rede.
fn rede_autorizada(valor: Option<&str>) -> bool {
    valor.is_some_and(|valor| valor.trim().eq_ignore_ascii_case("true"))
}

/// Extrai o nome do pacote da mensagem montada pelo dominio (`... (nome ver)`).
///
/// O nome viaja dentro da mensagem porque o `BuildDiagnostic` do funil nao tem
/// campo para pacote — e acrescentar um campo que so a auditoria usaria
/// engordaria o contrato de todo mundo por causa de uma fatia.
fn pacote_do_achado(message: &str) -> Option<String> {
    let inicio = message.rfind(" (")? + 2;
    let fim = message.rfind(')')?;
    if fim <= inicio {
        return None;
    }
    let dentro = &message[inicio..fim];
    Some(
        dentro
            .split_whitespace()
            .next()
            .unwrap_or(dentro)
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn so_um_true_explicito_abre_a_rede() {
        // O portao da decisao do autor (2026-08-21). Assimetrico de proposito:
        // na duvida, offline. Config mal digitada nao vira consentimento.
        assert!(super::rede_autorizada(Some("true")));
        assert!(super::rede_autorizada(Some("TRUE")), "case-insensitive");
        assert!(
            super::rede_autorizada(Some(" true ")),
            "espaco nao atrapalha"
        );

        assert!(!super::rede_autorizada(None), "ausente = offline");
        assert!(!super::rede_autorizada(Some("false")));
        assert!(!super::rede_autorizada(Some("")));
        assert!(!super::rede_autorizada(Some("0")));
        assert!(!super::rede_autorizada(Some("sim")));
        assert!(!super::rede_autorizada(Some("yes")));
        // O caso que a mutacao pegou: QUALQUER valor presente nao pode bastar.
        assert!(!super::rede_autorizada(Some("qualquer coisa")));
    }

    #[test]
    fn pacote_do_achado_le_o_nome_entre_parenteses() {
        assert_eq!(
            super::pacote_do_achado("RUSTSEC-2017-0008: unmaintained (serial 0.4.0)").as_deref(),
            Some("serial")
        );
        // Sem versao ainda funciona.
        assert_eq!(
            super::pacote_do_achado("aviso (pacote)").as_deref(),
            Some("pacote")
        );
        // Mensagem que nao segue o formato nao vira nome inventado.
        assert_eq!(super::pacote_do_achado("sem parenteses"), None);
        assert_eq!(super::pacote_do_achado("vazio ()"), None);
    }
}
