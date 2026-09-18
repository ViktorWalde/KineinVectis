//! Handlers for `toolchain.*` requests (`impl Core`): ler e fixar qual
//! executavel cumpre cada papel no workspace aberto.
//!
//! Fino, como a §4 regra 2 manda: valida params, chama o dominio e formata a
//! resposta. Quem cruza escolha com maquina e o [`crate::toolchain`]; quem
//! detecta continua sendo o `ToolDetector`.

use std::path::Path;
use std::process::Command;

use kinein_protocol::{
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, ToolchainGetParams, ToolchainResult,
    ToolchainRole, ToolchainSetKitParams, ToolchainSetParams,
};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, toolchain};

impl Core {
    /// Roteia os metodos `toolchain.*`; `None` quando o metodo nao e deles.
    pub(crate) fn toolchain_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "toolchain.get" => Some(self.toolchain_get_response(request_id, params)),
            "toolchain.set" => Some(self.toolchain_set_response(request_id, params)),
            "toolchain.setKit" => Some(self.toolchain_set_kit_response(request_id, params)),
            _ => None,
        }
    }

    fn toolchain_get_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "toolchain.get");
        };
        let parsed = match parse_params::<ToolchainGetParams>(
            request_id.as_ref(),
            params,
            "toolchain.get aceita apenas preset",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let resolvida = toolchain::Toolchain::resolve_kit(
            &root,
            &self.detected_tools(),
            parsed.preset.as_deref().unwrap_or_default(),
        );
        JsonRpcResponse::success(request_id, json!(self.toolchain_result(&resolvida)))
    }

    fn toolchain_set_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "toolchain.set");
        };
        let parsed = match parse_params::<ToolchainSetParams>(
            request_id.as_ref(),
            params,
            "toolchain.set requer role (id ausente volta para automatico)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match toolchain::set(
            &root,
            &self.detected_tools(),
            parsed.role,
            parsed.id.as_deref(),
            parsed.preset.as_deref().unwrap_or_default(),
        ) {
            Ok(resolvida) => {
                // Trocar o compilador muda o `--query-driver` do clangd; vale
                // na proxima subida do servidor cpp (ou num `lsp.restart`).
                self.configure_clangd_from_toolchain(&root);
                JsonRpcResponse::success(request_id, json!(self.toolchain_result(&resolvida)))
            }
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }

    /// `toolchain.setKit` — sysroot e triple do alvo de um kit.
    fn toolchain_set_kit_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "toolchain.setKit");
        };
        let parsed = match parse_params::<ToolchainSetKitParams>(
            request_id.as_ref(),
            params,
            "toolchain.setKit aceita preset, sysroot, targetTriple, chip, remoteTarget, debugServer, toolchainFile e svdFile",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match toolchain::set_kit(
            &root,
            &self.detected_tools(),
            parsed.preset.as_deref().unwrap_or_default(),
            toolchain::KitUpdate {
                sysroot: parsed.sysroot.as_deref(),
                target_triple: parsed.target_triple.as_deref(),
                chip: parsed.chip.as_deref(),
                remote_target: parsed.remote_target.as_deref(),
                debug_server: parsed.debug_server.as_deref(),
                toolchain_file: parsed.toolchain_file.as_deref(),
                svd_file: parsed.svd_file.as_deref(),
            },
        ) {
            Ok(resolvida) => {
                self.configure_clangd_from_toolchain(&root);
                JsonRpcResponse::success(request_id, json!(self.toolchain_result(&resolvida)))
            }
            Err(message) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(JsonRpcErrorCode::InvalidParams, message, None),
            ),
        }
    }

    /// O resultado do kit mais o que so' um PROCESSO responde (`integracoes/39`,
    /// 2026-09-12): os alvos Rust instalados e se o compilador cross efetivo
    /// traz o sistema alvo. Os dois processos sao os DETECTADOS (PATH vazio
    /// nos testes = nada roda), curtos e sincronos.
    fn toolchain_result(&self, resolvida: &toolchain::Toolchain) -> ToolchainResult {
        let mut resultado = resolvida.to_result();
        let ferramentas = self.detected_tools();
        let rustup = ferramentas
            .iter()
            .find(|t| t.id == "rustup")
            .and_then(|t| t.path.clone());
        resultado.rust_targets = rustup
            .as_deref()
            .and_then(|p| alvos_rust_instalados(Path::new(p)));
        if resolvida.sysroot().is_none() {
            resultado.sysroot_hint = resolvida
                .effective_program(ToolchainRole::CCompiler)
                .and_then(|(id, path)| dica_de_sysroot(id, Path::new(path)));
        }
        resultado
    }
}

/// `rustup target list --installed`, uma linha por alvo. `None` se o rustup
/// nao respondeu (o chamador nao distingue "sem rustup" de "rustup quebrado":
/// nos dois casos a IDE nao sabe os alvos, e diz isso).
fn alvos_rust_instalados(rustup: &Path) -> Option<Vec<String>> {
    let saida = Command::new(rustup)
        .args(["target", "list", "--installed"])
        .output()
        .ok()?;
    if !saida.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&saida.stdout)
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}

/// Um compilador cross de Linux (`*-linux-gnu*`) sem `usr/include` no sysroot
/// que ele mesmo declara (`-print-sysroot`) nao compila programa de usuario
/// nenhum — e' como as distros empacotam o `gcc-aarch64-linux-gnu` (medido no
/// Fedora 44 em 2026-09-12: `/usr/aarch64-linux-gnu/sys-root` vazio). Bare
/// metal (`-none-eabi`, `-elf`) traz a libc no proprio pacote e nao entra aqui.
fn dica_de_sysroot(id: &str, cc: &Path) -> Option<String> {
    if !id.contains("-linux-gnu") {
        return None;
    }
    let saida = Command::new(cc).arg("-print-sysroot").output().ok()?;
    let sysroot = String::from_utf8_lossy(&saida.stdout).trim().to_owned();
    let tem_cabecalhos = !sysroot.is_empty() && Path::new(&sysroot).join("usr/include").is_dir();
    if tem_cabecalhos {
        return None;
    }
    let onde = if sysroot.is_empty() {
        "sem sysroot declarado".to_owned()
    } else {
        format!("sysroot `{sysroot}` sem usr/include")
    };
    Some(format!(
        "o compilador cross `{id}` nao traz o sistema alvo ({onde}): aponte um sysroot no kit — \
         copiado da placa (rsync de /usr/include e /usr/lib), o tarball da Bootlin, ou o SDK \
         Yocto/Buildroot"
    ))
}
