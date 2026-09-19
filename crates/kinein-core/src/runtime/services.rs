//! Habilitacao dos servicos externos e configuracao dos servidores LSP do Core.

use crate::{Core, dap, jobs, lsp, terminal};

impl Core {
    /// Enables LSP, process execution and the terminal session, pushing
    /// async notifications (`event.lsp.*`, `event.run.*`,
    /// `event.terminal.*`) through `events`.
    ///
    /// Without this call (tests and `run_json_lines`), LSP operations are
    /// no-ops, no language server is spawned and `run.*`/`terminal.*` are
    /// unavailable.
    pub fn enable_lsp(&mut self, events: lsp::EventSender) {
        self.lsp = Some(lsp::LspManager::new(events.clone()));
        self.debug = Some(dap::DebugManager::new(events.clone()));
        self.jobs = Some(jobs::JobManager::new(events.clone()));
        self.terminal = Some(terminal::TerminalManager::new(events.clone()));
        self.events = Some(events);
    }

    /// Liga as respostas ADIADAS (Etapa 2 F6): as consultas LSP passam a
    /// esperar numa thread e a resposta vai por `responses` — o laco a
    /// escreve na ordem em que chega. Sem esta chamada, esperam inline.
    pub fn enable_deferred_responses(
        &mut self,
        responses: std::sync::mpsc::Sender<kinein_protocol::JsonRpcResponse>,
    ) {
        self.deferred = Some(responses);
    }

    /// Responde um pedido cujo trabalho e' LONGO e nao precisa do `Core`
    /// (F6-b): com o canal ligado, `work` roda numa thread e a resposta vai
    /// por ele; sem o canal, inline. `work` recebe o id de volta.
    pub(crate) fn defer_work<F>(
        &self,
        request_id: Option<serde_json::Value>,
        work: F,
    ) -> kinein_protocol::JsonRpcResponse
    where
        F: FnOnce(Option<serde_json::Value>) -> kinein_protocol::JsonRpcResponse + Send + 'static,
    {
        match self.deferred.as_ref() {
            Some(sender) => {
                let sender = sender.clone();
                std::thread::spawn(move || drop(sender.send(work(request_id))));
                crate::rpc::deferred_marker()
            }
            None => work(request_id),
        }
    }

    /// Responde uma consulta LSP: pronta na hora, adiada numa thread (com o
    /// canal ligado) ou esperada inline (sem ele — testes). A thread faz o
    /// que o laco fazia: espera ate' o timeout, traduz erro/resultado.
    pub(crate) fn defer_lsp(
        &self,
        request_id: Option<serde_json::Value>,
        begun: lsp::LspBegun,
    ) -> kinein_protocol::JsonRpcResponse {
        use kinein_protocol::JsonRpcResponse;
        let pending = match begun {
            lsp::LspBegun::Ready(value) => return JsonRpcResponse::success(request_id, value),
            lsp::LspBegun::Pending(pending) => pending,
        };
        let responder = move || -> JsonRpcResponse {
            match pending.reply.wait() {
                Ok(value) => JsonRpcResponse::success(request_id.clone(), (pending.finish)(&value)),
                Err(error) => crate::rpc::lsp_error_response(request_id, &error),
            }
        };
        match self.deferred.as_ref() {
            Some(sender) => {
                let sender = sender.clone();
                std::thread::spawn(move || drop(sender.send(responder())));
                crate::rpc::deferred_marker()
            }
            None => responder(),
        }
    }

    /// Aponta uma linguagem para outro executavel de language server.
    ///
    /// Devolve `false` quando o LSP nao esta habilitado neste loop ou quando a
    /// linguagem nao existe na tabela. Ver
    /// [`lsp::LspManager::use_server_command`] para o porque: e a costura que
    /// permite ao gate observar o que o core FALA com um servidor.
    pub fn use_language_server_command(
        &mut self,
        language: &str,
        command: &str,
        args: &[&str],
    ) -> bool {
        self.lsp
            .as_mut()
            .is_some_and(|lsp| lsp.use_server_command(language, command, args))
    }

    /// Poe (ou troca) um COMPANHEIRO de language server ao lado do principal
    /// de `language` — o `ruff server` ao lado do basedpyright. Mesma costura
    /// do [`Self::use_language_server_command`]: e' por aqui que o gate poe um
    /// servidor falso no lugar do companheiro e observa a fusao dos
    /// diagnosticos e das code actions.
    pub fn use_language_server_companion(
        &mut self,
        language: &'static str,
        key: &'static str,
        command: &str,
        args: &[&str],
    ) -> bool {
        self.lsp
            .as_mut()
            .is_some_and(|lsp| lsp.use_companion(language, key, command, args))
    }
}
