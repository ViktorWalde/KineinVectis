//! External AI CLI Bridge handlers (`aiBridge.*`).
//!
//! The bridge only launches user-installed, allowlisted CLIs in the existing
//! PTY manager. It does not call cloud APIs, inspect responses or apply edits.

use kinein_protocol::{
    AiCliProfile, AiCliProfileId, AiCliProfilesResult, AiTerminalOpenParams, AiTerminalOpenResult,
    JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::Core;
use crate::rpc::{
    no_workspace_response, parse_params, terminal_error_response, terminal_unavailable_response,
};

struct ProfileSpec {
    id: AiCliProfileId,
    name: &'static str,
    command: &'static str,
    args: &'static [&'static str],
}

const PROFILES: &[ProfileSpec] = &[
    ProfileSpec {
        id: AiCliProfileId::Claude,
        name: "Claude CLI",
        command: "claude",
        args: &[],
    },
    ProfileSpec {
        id: AiCliProfileId::Codex,
        name: "Codex CLI",
        command: "codex",
        // O TUI padrao usa a alternate screen, que por definicao nao gera
        // scrollback. No painel dedicado da IDE usamos o modo inline oficial
        // do Codex: a conversa continua sendo renderizada pela CLI real, mas
        // ganha o mesmo historico navegavel de um terminal profissional.
        args: &["--no-alt-screen"],
    },
];

impl Core {
    /// Routes `aiBridge.*`; returns `None` for another domain.
    pub(crate) fn ai_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "aiBridge.profiles" => Some(self.ai_profiles_response(request_id)),
            "aiBridge.terminal.open" => Some(self.ai_terminal_open_response(request_id, params)),
            _ => None,
        }
    }

    fn ai_profiles_response(&self, request_id: Option<Value>) -> JsonRpcResponse {
        let profiles = PROFILES
            .iter()
            .map(|profile| AiCliProfile {
                id: profile.id,
                name: profile.name.to_owned(),
                command: profile.command.to_owned(),
                available: self.detector.find_binary(profile.command).is_some(),
            })
            .collect();
        let default_profile = crate::settings::resolve(
            &crate::settings::load_global(),
            &self
                .workspace_root()
                .map(|root| crate::settings::load_workspace(&root))
                .unwrap_or_default(),
        )
        .ai_cli_profile;
        JsonRpcResponse::success(
            request_id,
            json!(AiCliProfilesResult {
                profiles,
                default_profile,
            }),
        )
    }

    fn ai_terminal_open_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "aiBridge.terminal.open");
        };
        let parsed = match parse_params::<AiTerminalOpenParams>(
            request_id.as_ref(),
            params,
            "aiBridge.terminal.open requer profileId (claude|codex)",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let profile = profile_spec(parsed.profile_id);
        let Some(binary) = self.detector.find_binary(profile.command) else {
            return missing_cli_response(request_id, profile);
        };
        let command = binary.to_string_lossy().into_owned();
        let Some(terminal) = self.terminal.as_mut() else {
            return terminal_unavailable_response(request_id, "aiBridge.terminal.open");
        };
        let args: Vec<String> = profile
            .args
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect();
        match terminal.open_command_preserving_scrollback(&root, &command, &args) {
            Ok(id) => JsonRpcResponse::success(
                request_id,
                json!(AiTerminalOpenResult {
                    id,
                    profile_id: profile.id,
                    name: profile.name.to_owned(),
                    command: display_command(&command, profile.args),
                }),
            ),
            Err(error) => terminal_error_response(request_id, &error),
        }
    }
}

fn display_command(program: &str, args: &[&str]) -> String {
    if args.is_empty() {
        return program.to_owned();
    }
    format!("{program} {}", args.join(" "))
}

fn profile_spec(id: AiCliProfileId) -> &'static ProfileSpec {
    match id {
        AiCliProfileId::Claude => &PROFILES[0],
        AiCliProfileId::Codex => &PROFILES[1],
    }
}

fn missing_cli_response(request_id: Option<Value>, profile: &ProfileSpec) -> JsonRpcResponse {
    JsonRpcResponse::failure(
        request_id,
        JsonRpcError::new(
            JsonRpcErrorCode::ToolNotFound,
            format!(
                "{} nao esta instalado ou nao foi encontrado no PATH; instale a CLI antes de usar o KV Context",
                profile.name
            ),
            Some(json!({ "command": profile.command, "profileId": profile.id })),
        ),
    )
}

#[cfg(test)]
mod tests {
    use kinein_protocol::AiCliProfileId;

    use super::{display_command, profile_spec};

    #[test]
    fn codex_uses_its_official_inline_terminal_mode() {
        let profile = profile_spec(AiCliProfileId::Codex);

        assert_eq!(profile.args, ["--no-alt-screen"]);
        assert_eq!(
            display_command("/usr/bin/codex", profile.args),
            "/usr/bin/codex --no-alt-screen"
        );
    }
}
