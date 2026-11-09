//! Fresh compilation context for the conservative correction consumer.

// Crate-internal consumer shared across domains; public export conflicts with unreachable_pub.
#![allow(
    clippy::redundant_pub_crate,
    reason = "Keep shared verifier details internal to the core"
)]

use crate::cdb::{VerificationContext, verification::cdb_bytes};
use std::path::Path;

pub(crate) fn verification_context(
    root: &Path,
    path: &Path,
) -> Result<VerificationContext, String> {
    let language = match path.extension().and_then(|s| s.to_str()) {
        Some("c") => "c",
        Some("cc" | "cpp" | "cxx") => "cpp",
        _ => return Err("Escopo ainda não suportado: requer uma unidade C/C++.".into()),
    };
    let status = crate::cdb::status(root);
    if status.stale {
        return Err("CDB obsoleta: reconfigure o projeto.".into());
    }
    let cdb = status.file(root).ok_or("Contexto ausente: sem CDB.")?;
    let bytes = cdb_bytes(&cdb)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| "CDB não é UTF-8.")?;
    let loaded = super::CompileContext::from_cdb_text(root, &cdb, text);
    if loaded.summary().cdb_stale {
        return Err("CDB obsoleta: reconfigure o projeto.".into());
    }
    if !loaded.unambiguous_cdb_unit(path) {
        return Err("Contexto ausente ou ambíguo na CDB.".into());
    }
    let context = loaded.for_file(root, path, language);
    if let Some(reason) = context.stale_because {
        return Err(format!("Contexto obsoleto: {reason}"));
    }
    let unit = context
        .unit
        .ok_or("Contexto ausente: sem unidade exata na CDB.")?;
    Ok(VerificationContext { unit, cdb, bytes })
}
