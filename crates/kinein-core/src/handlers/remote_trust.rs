//! `remote.hostKey` / `remote.trustHost` (`impl Core`, `0.153.0`): confiar no
//! servidor na primeira conexao, pela tela — o porque esta' no
//! `remote/trust.rs`.
//!
//! Os dois rodam sincronamente, como o `remote.resolve`: so' sao pedidos
//! depois de uma sonda que ALCANCOU o alvo e parou no host key, entao o
//! `ssh-keyscan` responde em milissegundos (o `-T 4` e' o teto).

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use kinein_protocol::{
    JsonRpcResponse, RemoteHostKey, RemoteHostKeyResult, RemoteNameParams, RemoteTarget,
    RemoteTrustHostParams, RemoteTrustHostResult,
};
use serde_json::{Value, json};

use crate::Core;
use crate::remote::trust;
use crate::rpc::{no_workspace_response, parse_params};

use super::remote::{error_response, target_or_error};

/// Uma chave escaneada: a linha que vai para o `known_hosts` e o que a pessoa ve'.
struct ScannedKey {
    line: String,
    key: RemoteHostKey,
}

/// O que o servidor oferece agora: onde gravar e as chaves com impressao.
struct Scan {
    host: String,
    file: PathBuf,
    keys: Vec<ScannedKey>,
}

impl Core {
    /// Roteia a confianca no host key.
    pub(crate) fn remote_trust_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "remote.hostKey" => Some(self.remote_host_key_response(request_id, params)),
            "remote.trustHost" => Some(self.remote_trust_host_response(request_id, params)),
            _ => None,
        }
    }

    fn remote_host_key_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteNameParams>(
            request_id.as_ref(),
            params,
            "remote.hostKey requer name",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let target = match self.trust_target(request_id.as_ref(), "remote.hostKey", &parsed.name) {
            Ok(target) => target,
            Err(response) => return *response,
        };
        match self.scan_host(&target) {
            Ok(scan) => JsonRpcResponse::success(
                request_id,
                json!(RemoteHostKeyResult {
                    name: target.name,
                    host: scan.host,
                    keys: scan.keys.into_iter().map(|scanned| scanned.key).collect(),
                }),
            ),
            Err(reason) => error_response(request_id, reason),
        }
    }

    fn remote_trust_host_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteTrustHostParams>(
            request_id.as_ref(),
            params,
            "remote.trustHost requer name e fingerprints",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let target = match self.trust_target(request_id.as_ref(), "remote.trustHost", &parsed.name)
        {
            Ok(target) => target,
            Err(response) => return *response,
        };
        let scan = match self.scan_host(&target) {
            Ok(scan) => scan,
            Err(reason) => return error_response(request_id, reason),
        };
        // So' o que a pessoa VIU: uma chave que nao estava na tela nao entra.
        let lines: Vec<String> = scan
            .keys
            .iter()
            .filter(|scanned| parsed.fingerprints.contains(&scanned.key.fingerprint))
            .map(|scanned| scanned.line.clone())
            .collect();
        if lines.is_empty() {
            return error_response(
                request_id,
                "a chave que o servidor oferece agora não é a que você viu — nada foi gravado; \
                 confira de novo",
            );
        }
        match append_lines(&scan.file, &lines) {
            Ok(()) => JsonRpcResponse::success(
                request_id,
                json!(RemoteTrustHostResult {
                    name: target.name,
                    file: scan.file.display().to_string(),
                    recorded: lines.len(),
                }),
            ),
            Err(reason) => error_response(request_id, reason),
        }
    }

    fn trust_target(
        &self,
        request_id: Option<&Value>,
        method: &str,
        name: &str,
    ) -> Result<RemoteTarget, Box<JsonRpcResponse>> {
        let Some(root) = self.workspace_root() else {
            return Err(Box::new(no_workspace_response(request_id.cloned(), method)));
        };
        target_or_error(&root, request_id, name)
    }

    /// `ssh -G` (onde gravar, o host e a porta efetivos), `ssh-keyscan` (as
    /// chaves) e `ssh-keygen -lf` (a impressao de cada uma).
    fn scan_host(&self, target: &RemoteTarget) -> Result<Scan, String> {
        let tool = |name: &str| {
            self.detector
                .find_in_path(name)
                .ok_or_else(|| format!("não achei `{name}` no PATH — instale o openssh-client"))
        };
        let ssh = tool("ssh")?;
        let keyscan = tool("ssh-keyscan")?;
        let keygen = tool("ssh-keygen")?;
        let home = self.sdk_home().unwrap_or_default();
        let effective = Command::new(&ssh)
            .args(trust::effective_args(target))
            .output()
            .map_err(|error| format!("não pude rodar `ssh -G`: {error}"))?;
        if !effective.status.success() {
            return Err(format!("o `ssh` não aceitou o alvo `{}`", target.name));
        }
        let config = trust::parse_effective(&String::from_utf8_lossy(&effective.stdout), &home);
        let scanned = Command::new(&keyscan)
            .args(trust::keyscan_args(&config))
            .stderr(Stdio::null())
            .output()
            .map_err(|error| format!("não pude rodar `ssh-keyscan`: {error}"))?;
        let mut keys: Vec<ScannedKey> = trust::key_lines(&String::from_utf8_lossy(&scanned.stdout))
            .into_iter()
            .filter_map(|line| fingerprint(&keygen, &line).map(|key| ScannedKey { line, key }))
            .collect();
        if keys.is_empty() {
            return Err(format!(
                "o servidor {} não respondeu com chave nenhuma — sem sshd na porta {}, ou ele está segurando conexões \
                 por alguns segundos depois de uma autenticação recusada (OpenSSH 9.8+); tente de novo",
                config.host_name, config.port
            ));
        }
        keys.sort_by_key(|scanned| trust::strength(&scanned.key.kind));
        Ok(Scan {
            host: trust::known_hosts_name(&config.host_name, config.port),
            file: config
                .known_hosts
                .unwrap_or_else(|| home.join(".ssh").join("known_hosts")),
            keys,
        })
    }
}

/// A impressao de uma linha de chave, pelo `ssh-keygen -lf -`.
fn fingerprint(keygen: &Path, line: &str) -> Option<RemoteHostKey> {
    let mut child = Command::new(keygen)
        .args(["-l", "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child
        .stdin
        .take()?
        .write_all(format!("{line}\n").as_bytes())
        .ok()?;
    let output = child.wait_with_output().ok()?;
    trust::parse_fingerprint(String::from_utf8_lossy(&output.stdout).trim())
}

/// Acrescenta as linhas ao `known_hosts`, criando `~/.ssh` (0700) se faltar.
fn append_lines(file: &Path, lines: &[String]) -> Result<(), String> {
    if let Some(parent) = file.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("não pude criar {}: {error}", parent.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let _ = fs::set_permissions(parent, fs::Permissions::from_mode(0o700));
        }
    }
    let mut handle = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .map_err(|error| format!("não pude abrir {}: {error}", file.display()))?;
    for line in lines {
        writeln!(handle, "{line}")
            .map_err(|error| format!("não pude gravar em {}: {error}", file.display()))?;
    }
    Ok(())
}
