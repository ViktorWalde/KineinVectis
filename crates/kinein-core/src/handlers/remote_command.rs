//! `remote.command` (`impl Core`) — compor a linha, sem rodar nada.
//!
//! Arquivo proprio por RESPONSABILIDADE, nao por tamanho: o
//! `handlers/remote.rs` guarda o catalogo do projeto e as operacoes que movem
//! bytes (probe e deploy sao jobs). Aqui nada roda e nada sai do processo — a
//! unidade PURA que traduz "quero rodar/depurar/abrir shell/copiar a chave"
//! na linha exata, e devolve tambem DE ONDE cada pedaco veio (`source`), para
//! a UI poder mostrar antes de executar.
//!
//! Quem executa e' a UI, pelo dono certo: configuracao de execucao, o kit, ou
//! o terminal da IDE.

use kinein_protocol::{
    JsonRpcResponse, RemoteCommandKind, RemoteCommandParams, RemoteCommandResult, RemoteTarget,
};
use serde_json::{Value, json};

use crate::Core;
use crate::remote;
use crate::rpc::{no_workspace_response, parse_params};

use super::remote::alvo_ou_falha;

/// `remote.command { kind: copyId }`: a linha que copia a chave do usuario.
///
/// PURO, como os outros `kind`: compoe e devolve. Quem executa e' a UI, e so'
/// depois de mostrar a linha e receber um gesto explicito — copiar chave nao
/// pode acontecer porque alguem clicou em "sondar".
fn copy_id_response(request_id: Option<Value>, target: &RemoteTarget) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request_id,
        json!(RemoteCommandResult {
            command: remote::ssh_copy_id_line(target),
            remote_target: None,
            name: format!("Copiar chave para {}", target.name),
            source: vec![
                format!("alvo `{}` de .kinein/remotes.json", target.name),
                "a chave publica e' SUA; a IDE nao gera chave nem digita senha".to_owned(),
            ],
        }),
    )
}

impl Core {
    /// Roteia `remote.command`.
    pub(crate) fn remote_command_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        // Braco de `match` e nao `then(...)`: o `verificar-fiacao-ipc.sh` e a
        // lista canonica do `03-ipc-protocol` acham metodo roteado por este
        // formato. Escrever diferente some com o metodo das duas contagens
        // sem que nada reprove — medido em 2026-09-24, quando o total caiu de
        // 167 para 166.
        // Braco de `match` e nao `then(...)`: o `verificar-fiacao-ipc.sh` e a
        // lista canonica do `03-ipc-protocol` acham metodo roteado por este
        // formato. Escrever diferente some com o metodo das duas contagens —
        // medido em 2026-09-24, quando o total caiu de 167 para 166. Hoje a
        // checagem inversa do gate reprova isso.
        match method {
            "remote.command" => Some(self.remote_command_response(request_id, params)),
            _ => None,
        }
    }

    /// `remote.command { name, kind, program?, port? }` -> a linha `ssh …`
    /// pronta para `run.start` (`sh -c`) e o `host:porta` do kit. PURO.
    fn remote_command_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteCommandParams>(
            request_id.as_ref(),
            params,
            "remote.command requer name e kind (run | debugServer | debugpy | shell)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "remote.command");
        };
        let target = match alvo_ou_falha(&root, request_id.as_ref(), &parsed.name) {
            Ok(target) => target,
            Err(response) => return *response,
        };
        if parsed.kind == RemoteCommandKind::CopyId {
            return copy_id_response(request_id, &target);
        }
        let projeto = root.file_name().map_or_else(
            || "projeto".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let mut source = vec![format!("alvo `{}` de .kinein/remotes.json", target.name)];
        let program = match (
            &parsed.kind,
            parsed.program.as_deref().filter(|p| !p.trim().is_empty()),
        ) {
            (RemoteCommandKind::Shell, _) => String::new(),
            (_, Some(p)) if p.starts_with('/') || p.starts_with('~') => {
                source.push("programa: caminho no alvo, como dado".to_owned());
                p.to_owned()
            }
            (_, Some(p)) => {
                source.push(format!("programa: {p} dentro de deployDir"));
                format!("{}/{p}", remote::deploy_dir(&target, &projeto))
            }
            (_, None) => {
                source.push("programa: <binario> dentro de deployDir — edite".to_owned());
                format!("{}/<binario>", remote::deploy_dir(&target, &projeto))
            }
        };
        let port = parsed.port.unwrap_or(match parsed.kind {
            RemoteCommandKind::Debugpy => remote::DEBUGPY_PORT,
            _ => remote::GDBSERVER_PORT,
        });
        let remoto = remote::remote_command(parsed.kind, &program, port);
        let command = remote::ssh_shell_line(&target, remoto.as_deref());
        let (name, remote_target) = match parsed.kind {
            RemoteCommandKind::Run => (format!("Rodar em {}", target.name), None),
            RemoteCommandKind::DebugServer => {
                source.push(format!(
                    "gdbserver na porta {port}; o kit recebe remoteTarget"
                ));
                (
                    format!("gdbserver em {}", target.name),
                    Some(format!("{}:{port}", target.host)),
                )
            }
            RemoteCommandKind::Debugpy => {
                source.push(format!("debugpy na porta {port}; attach por host:porta"));
                (
                    format!("debugpy em {}", target.name),
                    Some(format!("{}:{port}", target.host)),
                )
            }
            // CopyId sai por `copy_id_response` la' em cima; o compilador
            // exige o braco, e ele nao pode inventar um nome silencioso.
            RemoteCommandKind::Shell | RemoteCommandKind::CopyId => {
                (format!("Shell em {}", target.name), None)
            }
        };
        JsonRpcResponse::success(
            request_id,
            json!(RemoteCommandResult {
                command,
                remote_target,
                name,
                source
            }),
        )
    }
}
