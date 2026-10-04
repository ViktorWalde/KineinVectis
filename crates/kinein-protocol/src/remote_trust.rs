//! Confiar no servidor e o ultimo contato com cada alvo (`0.153.0`,
//! 2026-10-04): `remote.hostKey`, `remote.trustHost` e o `contacts` do
//! `remote.list`. Arquivo proprio pela catraca do `remote.rs` (500 linhas) e
//! porque a pergunta e' outra: nao o perfil do alvo, mas o que a IDE observou
//! dele e se a pessoa confia na identidade que ele apresenta.

use serde::{Deserialize, Serialize};

use crate::RemoteFailure;

/// One host key the target offers, as `ssh-keygen -lf` prints it (`0.153.0`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHostKey {
    /// Key type, e.g. `ED25519`, `ECDSA`, `RSA`.
    pub kind: String,
    /// `SHA256:...` fingerprint — what the person compares.
    pub fingerprint: String,
}

/// Result of `remote.hostKey { name }` (`0.153.0`): the keys the target
/// offers right now, read by `ssh-keyscan` without logging in.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHostKeyResult {
    /// The target's name, echoed.
    pub name: String,
    /// How `known_hosts` names it: `host` or `[host]:port`.
    pub host: String,
    /// The keys offered, strongest first.
    pub keys: Vec<RemoteHostKey>,
}

/// Parameters for `remote.trustHost` (`0.153.0`).
///
/// The fingerprints the person SAW. The core scans again and only records
/// keys whose fingerprint is in this list, so what was shown is exactly what
/// is trusted.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteTrustHostParams {
    /// The target's name.
    pub name: String,
    /// `SHA256:...` fingerprints shown to the person.
    pub fingerprints: Vec<String>,
}

/// Result of `remote.trustHost` (`0.153.0`).
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteTrustHostResult {
    /// The target's name, echoed.
    pub name: String,
    /// The `known_hosts` file the lines went to (the one `ssh -G` reports).
    pub file: String,
    /// How many key lines were recorded.
    pub recorded: usize,
}

/// The last probe of one target (`0.153.0`), kept in
/// `.kinein/remote-contacts.json`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteContact {
    /// The target's name.
    pub name: String,
    /// The probe reached the target and logged in.
    pub ok: bool,
    /// When, in Unix seconds.
    pub at: u64,
    /// `uname -m`, when it answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// `uname -sr`, when it answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    /// Why it did not, typed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<RemoteFailure>,
}
