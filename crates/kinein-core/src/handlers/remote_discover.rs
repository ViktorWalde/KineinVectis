//! `remote.discover` / `remote.resolve` (`impl Core`) — a DESCOBERTA e a
//! EXPLICACAO do SSH que a maquina ja' tem (`0.132.0`, fatia R0.5 de
//! `especificacoes/remote-ssh-ui-hud.md`).
//!
//! Arquivo proprio porque a responsabilidade e' outra, nao porque o
//! `handlers/remote.rs` cresceu: ali estao o CATALOGO do projeto (perfis que o
//! autor salva) e as operacoes SOBRE um alvo. Aqui nao existe alvo ainda — a
//! pergunta e' sobre a MAQUINA, e a resposta nao depende do projeto aberto.
//! Foi o mesmo corte que separou o `remote_mirror.rs`.
//!
//! Nenhuma das duas conecta: `discover` le' arquivo local e `resolve` roda
//! `ssh -G`, que imprime a configuracao efetiva sem abrir sessao.

use std::process::Command;

use kinein_protocol::{
    JsonRpcResponse, RemoteDiscoverParams, RemoteParseParams, RemoteParseResult,
    RemoteResolveParams,
};
use serde_json::{Value, json};

use crate::Core;
use crate::remote;
use crate::rpc::parse_params;

use super::remote::falha;

/// A primeira linha que o `ssh -G` reclamou, para a UI repetir em vez de
/// inventar um resumo.
fn recusa_do_ssh(host: &str, stderr: &[u8]) -> String {
    let texto = String::from_utf8_lossy(stderr);
    let motivo = texto.lines().map(str::trim).find(|l| !l.is_empty());
    motivo.map_or_else(
        || format!("o `ssh` nao reconheceu `{host}`"),
        |linha| format!("o `ssh` nao aceitou `{host}`: {linha}"),
    )
}

impl Core {
    /// Roteia a descoberta de `remote.*`.
    pub(crate) fn remote_discover_request_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "remote.discover" => Some(self.remote_discover_response(request_id, params)),
            "remote.resolve" => Some(self.remote_resolve_response(request_id, params)),
            "remote.parseCommand" => Some(Self::remote_parse_response(request_id, params)),
            _ => None,
        }
    }

    /// `remote.discover` (`0.132.0`): os aliases que a maquina JA' tem.
    ///
    /// NAO exige workspace, de proposito: a pergunta e' sobre a maquina, nao
    /// sobre o projeto, e o primeiro uso precisa dela antes de existir alvo
    /// salvo. Le' arquivo local; nao conecta e nao varre rede.
    fn remote_discover_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RemoteDiscoverParams>(
            request_id.as_ref(),
            params,
            "remote.discover nao aceita parametros",
        ) {
            return *response;
        }
        // `sdk_home` e' o `$HOME` do core, sobrescrivivel em teste — nao um
        // detalhe de SDK. E' a mesma home de onde o `ssh` leria o config.
        let Some(home) = self.sdk_home() else {
            return falha(
                request_id,
                "sem HOME: nao sei onde procurar o `~/.ssh/config`",
            );
        };
        JsonRpcResponse::success(request_id, json!(remote::discover::aliases(&home)))
    }

    /// `remote.parseCommand` (`0.134.0`): a linha `ssh` colada vira um perfil
    /// PROPOSTO.
    ///
    /// NAO exige workspace e NAO salva nada: e' texto entrando, proposta
    /// saindo. Quem grava e' o `remote.save`, depois de a pessoa conferir. E
    /// nao executa a linha — ela e' lida, nunca rodada.
    fn remote_parse_response(request_id: Option<Value>, params: Option<&Value>) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteParseParams>(
            request_id.as_ref(),
            params,
            "remote.parseCommand requer command",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match remote::parse::parse_ssh_command(&parsed.command) {
            Ok((target, source)) => {
                JsonRpcResponse::success(request_id, json!(RemoteParseResult { target, source }))
            }
            Err(motivo) => falha(request_id, motivo),
        }
    }

    /// `remote.resolve` (`0.132.0`): o que o OpenSSH FARIA com este host.
    ///
    /// `ssh -G` imprime a configuracao efetiva sem conectar, entao roda
    /// sincronamente como os outros comandos locais curtos do core (git, probe).
    /// Ressalva dita: um `Match exec` no config do proprio usuario executa
    /// durante a avaliacao e pode demorar — exatamente como demoraria o `ssh`.
    fn remote_resolve_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RemoteResolveParams>(
            request_id.as_ref(),
            params,
            "remote.resolve requer host",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if let Err(motivo) = remote::discover::validate_host(&parsed.host) {
            return falha(request_id, motivo);
        }
        let Some(ssh) = self.detector.find_in_path("ssh") else {
            return falha(
                request_id,
                "nao achei `ssh` no PATH — instale o openssh-client",
            );
        };
        // O mesmo arquivo que a descoberta leu: o OpenSSH nao honra `$HOME`
        // para achar o config, entao sem `-F` os dois contratos podiam falar de
        // arquivos diferentes.
        let config = self
            .sdk_home()
            .map(|home| remote::discover::config_path(&home))
            .unwrap_or_default();
        match Command::new(&ssh)
            .args(remote::discover::resolve_args(&parsed.host, &config))
            .output()
        {
            Ok(fim) if fim.status.success() => JsonRpcResponse::success(
                request_id,
                json!(remote::discover::parse_resolved(
                    &parsed.host,
                    &String::from_utf8_lossy(&fim.stdout)
                )),
            ),
            Ok(fim) => falha(request_id, recusa_do_ssh(&parsed.host, &fim.stderr)),
            Err(erro) => falha(request_id, format!("nao pude rodar `ssh -G`: {erro}")),
        }
    }
}
