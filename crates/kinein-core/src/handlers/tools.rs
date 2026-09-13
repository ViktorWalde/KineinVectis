//! Handlers de `tools.*` e `environment.scan` (`impl Core`).
//!
//! # Por que este módulo nasceu em 2026-08-30
//!
//! Todo o conteúdo abaixo morava no `lib.rs`, cuja responsabilidade escrita é
//! *"SÓ: struct Core, dispatch, `RequestOutcome`, erro do loop"*
//! (`ARCHITECTURE.md` §4 regra 1: **lógica de domínio nunca entra ali**). Eram
//! ~140 linhas — o corpo inteiro do job de scan, o registro compartilhado e
//! três helpers — num arquivo que já estava em débito na catraca.
//!
//! Quem cobrou foi a catraca, ao reprovar **uma linha** acrescentada por outra
//! fatia. Seguindo a §4 regra 9 ("três suspeitos: a sua mudança, a categoria, o
//! arquivo"), a mudança estava certa (era um `pub mod`, que só pode morar no
//! `lib.rs`) e o arquivo é que carregava responsabilidade de outro dono. Devolvida
//! ela, o `lib.rs` saiu do débito — sem ninguém cortar linha para caber.

use std::sync::{Arc, Mutex};

use kinein_protocol::{
    JobAcceptedResult, JobRisk, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ToolInfo,
    ToolStatus, ToolsDetectResult,
};
use serde_json::{Value, json};

use crate::tools::KNOWN_TOOLS;
use crate::{Core, jobs};

impl Core {
    /// `tools.detect`: varre o PATH agora e publica o resultado no registro.
    pub(crate) fn tools_detect_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let tools = self.detector.detect_all();
        set_tool_registry(&self.tool_registry, tools.clone());
        JsonRpcResponse::success(request_id, json!(ToolsDetectResult { tools }))
    }

    /// `tools.status`: devolve o último resultado conhecido, varrendo só se
    /// ainda não houver nenhum. É o caminho barato que a UI chama com frequência.
    pub(crate) fn tools_status_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        JsonRpcResponse::success(
            request_id,
            json!(ToolsDetectResult {
                tools: self.detected_tools()
            }),
        )
    }

    /// Refaz a deteccao e publica no registro — para quando o disco mudou
    /// por acao da propria IDE (uma toolchain instalada na pasta dela).
    pub(crate) fn refresh_tool_registry(&self) {
        set_tool_registry(&self.tool_registry, self.detector.detect_all());
    }

    /// As ferramentas detectadas, varrendo o PATH so na primeira vez.
    ///
    /// E o mesmo caminho barato do `tools.status`, exposto para quem precisa da
    /// lista sem responder um request — hoje a toolchain, que cruza a escolha
    /// do usuario com o que existe na maquina.
    pub(crate) fn detected_tools(&self) -> Vec<ToolInfo> {
        self.tool_registry_snapshot().unwrap_or_else(|| {
            let tools = self.detector.detect_all();
            set_tool_registry(&self.tool_registry, tools.clone());
            tools
        })
    }

    pub(crate) fn tool_registry_snapshot(&self) -> Option<Vec<ToolInfo>> {
        self.tool_registry
            .lock()
            .ok()
            .and_then(|registry| registry.clone())
    }

    /// `environment.scan`: varredura completa como JOB cancelável, com
    /// progresso por ferramenta. Longo por natureza — nunca síncrono.
    pub(crate) fn environment_scan_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(jobs) = self.jobs.as_ref() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "jobs nao estao habilitados neste loop do core",
                    Some(json!({ "method": "environment.scan" })),
                ),
            );
        };

        let detector = self.detector.clone();
        let registry = Arc::clone(&self.tool_registry);
        let job_id = jobs.spawn(
            "environment.scan",
            "Environment Scan",
            JobRisk::Low,
            false,
            move |ctx| {
                let total = KNOWN_TOOLS.len();
                ctx.emit_event(
                    "event.environment.started",
                    json!({ "jobId": ctx.id(), "tools": total }),
                );
                ctx.report_progress(0.0, Some("iniciando scan de ambiente"));

                let mut tools = Vec::with_capacity(total);
                for (index, spec) in KNOWN_TOOLS.iter().enumerate() {
                    let info = detector.detect(spec);
                    ctx.emit_output(&format!(
                        "{}: {}",
                        info.display_name,
                        tool_status_label(info.status)
                    ));
                    ctx.emit_event(
                        "event.environment.tool",
                        json!({ "jobId": ctx.id(), "tool": info.clone() }),
                    );
                    tools.push(info);
                    ctx.report_progress(
                        progress_fraction(index + 1, total),
                        Some(spec.display_name),
                    );
                }

                set_tool_registry(&registry, tools.clone());
                let summary = ToolScanSummary::from_tools(&tools);
                ctx.emit_event(
                    "event.environment.finished",
                    json!({
                        "jobId": ctx.id(),
                        "success": true,
                        "total": summary.total,
                        "detected": summary.detected,
                        "missing": summary.missing,
                        "failed": summary.failed,
                        "tools": tools,
                    }),
                );

                jobs::JobOutcome::Success
            },
        );

        JsonRpcResponse::success(request_id, json!(JobAcceptedResult { job_id }))
    }
}

fn set_tool_registry(registry: &Arc<Mutex<Option<Vec<ToolInfo>>>>, tools: Vec<ToolInfo>) {
    if let Ok(mut slot) = registry.lock() {
        *slot = Some(tools);
    }
}

const fn tool_status_label(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::NotConfigured => "notConfigured",
        ToolStatus::Missing => "missing",
        ToolStatus::Detected => "detected",
        ToolStatus::Ready => "ready",
        ToolStatus::Running => "running",
        ToolStatus::Failed => "failed",
        ToolStatus::Disabled => "disabled",
    }
}

fn progress_fraction(done: usize, total: usize) -> f64 {
    let done = u32::try_from(done).unwrap_or(u32::MAX);
    let total = u32::try_from(total.max(1)).unwrap_or(u32::MAX);
    f64::from(done) / f64::from(total)
}

struct ToolScanSummary {
    total: u64,
    detected: u64,
    missing: u64,
    failed: u64,
}

impl ToolScanSummary {
    fn from_tools(tools: &[ToolInfo]) -> Self {
        let mut summary = Self {
            total: tools.len() as u64,
            detected: 0,
            missing: 0,
            failed: 0,
        };
        for tool in tools {
            match tool.status {
                ToolStatus::Detected | ToolStatus::Ready => summary.detected += 1,
                ToolStatus::Missing | ToolStatus::NotConfigured => summary.missing += 1,
                ToolStatus::Failed => summary.failed += 1,
                ToolStatus::Running | ToolStatus::Disabled => {}
            }
        }
        summary
    }
}
