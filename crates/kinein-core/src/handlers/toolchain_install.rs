//! Handlers do provedor de instalacao de toolchain (`impl Core`).
//!
//! `toolchain.installable` (o catalogo com o estado desta maquina, e o que o
//! projeto aberto recomenda) e `toolchain.install` (o JOB que baixa, confere
//! o SHA-256 e desempacota na pasta da IDE — `integracoes/39` §5).
//!
//! Fino: valida params, pergunta ao dominio, formata. Quem baixa e' o
//! [`crate::toolchain::install`]; quem passa a enxergar a toolchain depois e'
//! o detector, que le a pasta da IDE a cada busca.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    InstallableFirmware, InstallableToolchain, JobRisk, JsonRpcError, JsonRpcErrorCode,
    JsonRpcResponse, ToolchainInstallParams, ToolchainInstallableResult, ToolchainInstalledEvent,
};
use serde_json::{Value, json};

use crate::jobs::JobOutcome;
use crate::rpc::{jobs_unavailable_response, parse_params};
use crate::toolchain::install::{self, Entrada, InstallEvent, Kind};
use crate::{Core, tools};

impl Core {
    /// Roteia `toolchain.installable` e `toolchain.install`.
    pub(crate) fn toolchain_install_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "toolchain.installable" => Some(self.toolchain_installable_response(request_id)),
            "toolchain.install" => Some(self.toolchain_install_response(request_id, params)),
            _ => None,
        }
    }

    /// A pasta da IDE: a do detector (um teste aponta a sua), senao a padrao
    /// sob `$HOME`.
    pub(crate) fn toolchain_install_root(&self) -> Option<PathBuf> {
        self.detector
            .install_root()
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os("HOME").map(|h| tools::install_root(Path::new(&h))))
    }

    /// Nao exige workspace: o catalogo e a pasta sao desta maquina. Com um
    /// workspace, a familia do `project.model` marca o que e' recomendado.
    fn toolchain_installable_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let Some(raiz) = self.toolchain_install_root() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InternalError,
                    "sem $HOME: nao sei onde a IDE instala toolchains",
                    None,
                ),
            );
        };
        let root = self.workspace_root();
        let project_family = root.as_deref().and_then(|root| {
            let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
            crate::project::model(root, toolchain.chip()).target.family
        });
        let recomendada = project_family
            .as_deref()
            .and_then(install::catalog::familia_do_catalogo);
        // Um firmware MicroPython so' e' recomendado a um projeto MicroPython
        // da mesma familia; a uma app C do ESP-IDF, nunca.
        let micropython = root.as_deref().is_some_and(crate::project::e_micropython);
        let toolchains =
            install::CATALOGO
                .iter()
                .map(|e| installable(e, &raiz, recomendada))
                .chain(install::FIRMWARE.iter().map(|e| {
                    installable(e, &raiz, project_family.as_deref().filter(|_| micropython))
                }))
                .collect();
        JsonRpcResponse::success(
            request_id,
            json!(ToolchainInstallableResult {
                install_root: raiz.display().to_string(),
                toolchains,
                project_family,
            }),
        )
    }

    fn toolchain_install_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<ToolchainInstallParams>(
            request_id.as_ref(),
            params,
            "toolchain.install requer o campo id",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(entrada) = install::entrada(&parsed.id) else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    format!(
                        "`{}` nao esta' no catalogo de toolchains instalaveis (toolchain.installable lista os ids)",
                        parsed.id
                    ),
                    None,
                ),
            );
        };
        let Some(raiz) = self.toolchain_install_root() else {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InternalError, "sem $HOME", None),
            );
        };
        if install::is_installed(&raiz, entrada) {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidRequest,
                    format!(
                        "{} {} ja' esta' instalada em {}",
                        entrada.label,
                        entrada.version,
                        install::install_dir(&raiz, entrada).display()
                    ),
                    None,
                ),
            );
        }
        // O `tar` do sistema (GPL, processo): sem ele nao ha' como desempacotar,
        // e isso se diz ANTES de baixar 300 MB. Um firmware e' um arquivo so'
        // e nao precisa dele.
        let tar = if entrada.kind == Kind::Firmware {
            None
        } else {
            match self.detector.find_in_path("tar") {
                Some(tar) => Some(tar),
                None => {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::ToolNotFound,
                            "`tar` nao esta' nesta maquina — o pacote `tar` da distro desempacota a toolchain",
                            Some(json!({ "tool": "tar" })),
                        ),
                    );
                }
            }
        };
        let Some(jobs) = self.jobs.as_ref() else {
            return jobs_unavailable_response(request_id, "toolchain.install");
        };
        let titulo = format!("Instalar {} {}", entrada.label, entrada.version);
        let job_id = jobs.spawn("toolchain", titulo, JobRisk::Medium, true, move |ctx| {
            instalar(ctx, entrada, &raiz, tar.as_deref())
        });
        JsonRpcResponse::success(request_id, json!({ "jobId": job_id }))
    }
}

impl Core {
    /// Um firmware do catalogo, INSTALADO, como `flash.rs` o grava
    /// (`runConfig.flashProposal { firmware }`). `Err` diz o que falta: id
    /// desconhecido, nao e' firmware, ainda nao baixado.
    pub(crate) fn firmware_image(&self, id: &str) -> Result<crate::flash::FirmwareImage, String> {
        let e = install::entrada(id)
            .ok_or_else(|| format!("`{id}` nao esta' no catalogo (toolchain.installable)"))?;
        let fw = e
            .firmware
            .filter(|_| e.kind == Kind::Firmware)
            .ok_or_else(|| format!("`{id}` e' uma toolchain, nao um firmware"))?;
        let raiz = self.toolchain_install_root().ok_or("sem $HOME")?;
        let file = install::installed_path(&raiz, e);
        if !file.is_file() {
            return Err(format!(
                "{} {} ainda nao foi baixado: instale-o pelo catalogo (toolchain.install)",
                e.label, e.version
            ));
        }
        Ok(crate::flash::FirmwareImage {
            label: e.label.to_owned(),
            engine: fw.engine.to_owned(),
            offset: fw.offset.map(str::to_owned),
            chip: fw.chip.map(str::to_owned),
            size_bytes: std::fs::metadata(&file).map_or(e.size_bytes, |m| m.len()),
            file,
        })
    }
}

/// O que a tela mostra ANTES do clique: tudo do catalogo, e o estado aqui.
fn installable(e: &Entrada, raiz: &Path, recomendada: Option<&str>) -> InstallableToolchain {
    InstallableToolchain {
        id: e.id.to_owned(),
        label: e.label.to_owned(),
        version: e.version.to_owned(),
        family: e.family.to_owned(),
        url: e.url.to_owned(),
        size_bytes: e.size_bytes,
        sha256: e.sha256.to_owned(),
        license: e.license.to_owned(),
        source: e.source.to_owned(),
        install_dir: install::install_dir(raiz, e).display().to_string(),
        installed: install::is_installed(raiz, e),
        recommended: recomendada == Some(e.family),
        kind: e.kind.as_str().to_owned(),
        firmware: e.firmware.map(|fw| InstallableFirmware {
            board: fw.board.to_owned(),
            engine: fw.engine.to_owned(),
            offset: fw.offset.map(str::to_owned),
            chip: fw.chip.map(str::to_owned),
            file: install::installed_path(raiz, e).display().to_string(),
        }),
    }
}

/// O job: cada passo na saida, o progresso em bytes, e o evento no fim.
fn instalar(
    ctx: &crate::jobs::JobContext,
    e: &Entrada,
    raiz: &Path,
    tar: Option<&Path>,
) -> JobOutcome {
    let cancel = ctx.cancellation();
    let mut ultimo_percentual: u64 = u64::MAX;
    let mut sink = |evento: InstallEvent| match evento {
        InstallEvent::Downloading {
            url,
            expected_bytes,
        } => {
            ctx.emit_output(&format!(
                "baixando {url} ({} MiB) para {}",
                expected_bytes / (1024 * 1024),
                install::install_dir(raiz, e).display()
            ));
            ctx.report_progress(0.0, Some("baixando"));
        }
        InstallEvent::Progress { received, total } => {
            // Uma linha por ponto percentual, nao por bloco de 256 KiB.
            let percentual = (received * 100).checked_div(total).unwrap_or(0);
            if percentual != ultimo_percentual {
                ultimo_percentual = percentual;
                #[allow(clippy::cast_precision_loss)]
                ctx.report_progress(
                    (received as f64 / total.max(1) as f64) * 0.9,
                    Some(&format!("baixando: {percentual}%")),
                );
            }
        }
        InstallEvent::Verified { sha256 } => {
            ctx.emit_output(&format!("sha256 confere: {sha256}"));
            ctx.report_progress(0.92, Some("checksum confere; desempacotando"));
        }
        InstallEvent::Extracting { command } => ctx.emit_output(&format!("$ {command}")),
    };
    let resultado = install::install(e, raiz, tar, &cancel, &mut sink);
    let (success, error, path) = match &resultado {
        Ok(destino) => {
            ctx.emit_output(&if e.kind == Kind::Firmware {
                format!(
                    "baixado em {} — grave pelo painel de Embarcados (Gravar, firmware)",
                    destino.display()
                )
            } else {
                format!(
                    "instalada em {} — o detector ja' a enxerga; fixe-a no kit se quiser",
                    destino.display()
                )
            });
            (true, None, destino.display().to_string())
        }
        Err(erro) => {
            ctx.emit_output(&erro.to_string());
            (
                false,
                Some(erro.to_string()),
                install::install_dir(raiz, e).display().to_string(),
            )
        }
    };
    ctx.emit_event(
        "event.toolchain.installed",
        json!(ToolchainInstalledEvent {
            job_id: ctx.id().to_owned(),
            id: e.id.to_owned(),
            version: e.version.to_owned(),
            path,
            success,
            error,
        }),
    );
    if success {
        JobOutcome::Success
    } else {
        JobOutcome::Failed
    }
}
