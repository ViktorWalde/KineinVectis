//! Handlers for `workspace.*` requests (`impl Core`). `workspace_root` stays in
//! `lib.rs` because every domain relies on it.

use std::path::{Path, PathBuf};

use kinein_protocol::{
    DraftInfo, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse, RecentWorkspacePinParams,
    RecentWorkspaceRemoveParams, RecentWorkspacesParams, RecentWorkspacesResult,
    WorkspaceBrowseParams, WorkspaceCreateFolderParams, WorkspaceCreateFolderResult,
    WorkspaceCreateProjectParams, WorkspaceInfo, WorkspaceOpenParams, WorkspaceSaveSessionParams,
    WorkspaceSaveSessionResult,
};
use serde_json::{Value, json};

use crate::rpc::{
    fs_error_response, no_workspace_response, parse_params, workspace_error_response,
};
use crate::{Core, db, workspace};

fn recent_workspace_error_response(
    request_id: Option<Value>,
    error: &workspace::RecentWorkspaceError,
) -> JsonRpcResponse {
    let code = if error.is_invalid_params() {
        JsonRpcErrorCode::InvalidParams
    } else {
        JsonRpcErrorCode::InternalError
    };
    JsonRpcResponse::failure(request_id, JsonRpcError::new(code, error.to_string(), None))
}

impl Core {
    /// Liga a persistência local (rascunhos por-workspace + histórico global),
    /// apontando o estado global para `global_storage`.
    ///
    /// Mora aqui, e não no `lib.rs`, porque quem consome `global_storage` é o
    /// domínio workspace inteiro e mais ninguém (ARCHITECTURE.md §4 regra 9: o
    /// critério é responsabilidade).
    ///
    /// Ligar é gesto EXPLÍCITO, feito só pelo processo real em `run_stdio`: sem
    /// esta chamada o core não escreve NADA fora do workspace aberto. O porquê
    /// e a medição estão em
    /// `tests::workspace::test_core_never_writes_to_the_real_global_storage`.
    pub fn enable_persistence(&mut self, global_storage: PathBuf) {
        self.global_storage = Some(global_storage);
    }

    /// Routes the compact `workspace.recent.*` family outside the central
    /// composition root.
    pub(crate) fn recent_workspace_response(
        &self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "workspace.recent.list" => {
                Some(self.list_recent_workspaces_response(request_id, params))
            }
            "workspace.recent.pin" => Some(self.pin_recent_workspace_response(request_id, params)),
            "workspace.recent.remove" => {
                Some(self.remove_recent_workspace_response(request_id, params))
            }
            "workspace.recent.clear" => {
                Some(self.clear_recent_workspaces_response(request_id, params))
            }
            _ => None,
        }
    }

    fn recent_storage_unavailable_response(
        request_id: Option<Value>,
        method: &str,
    ) -> JsonRpcResponse {
        JsonRpcResponse::failure(
            request_id,
            JsonRpcError::new(
                JsonRpcErrorCode::InternalError,
                "storage global nao esta habilitado neste loop do core",
                Some(json!({ "method": method })),
            ),
        )
    }

    /// Lists the global recent-workspace snapshot. Lightweight in-process
    /// loops keep persistence disabled and therefore expose an empty list.
    pub(crate) fn list_recent_workspaces_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RecentWorkspacesParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.list nao aceita campos",
        ) {
            return *response;
        }
        let workspaces = self
            .global_storage
            .as_deref()
            .map_or_else(Vec::new, workspace::load_recent_workspaces_in);
        JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
    }

    /// Changes one recent workspace's pinned state.
    pub(crate) fn pin_recent_workspace_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RecentWorkspacePinParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.pin requer root e pinned",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(global_storage) = self.global_storage.as_deref() else {
            return Self::recent_storage_unavailable_response(request_id, "workspace.recent.pin");
        };
        match workspace::set_recent_workspace_pinned_in(global_storage, &parsed.root, parsed.pinned)
        {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Removes one recent workspace, including a root missing from disk.
    pub(crate) fn remove_recent_workspace_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let parsed = match parse_params::<RecentWorkspaceRemoveParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.remove requer root",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let Some(global_storage) = self.global_storage.as_deref() else {
            return Self::recent_storage_unavailable_response(
                request_id,
                "workspace.recent.remove",
            );
        };
        match workspace::remove_recent_workspace_in(global_storage, &parsed.root) {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Clears the complete global recent-workspace history.
    pub(crate) fn clear_recent_workspaces_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        if let Err(response) = parse_params::<RecentWorkspacesParams>(
            request_id.as_ref(),
            params,
            "workspace.recent.clear nao aceita campos",
        ) {
            return *response;
        }
        let Some(global_storage) = self.global_storage.as_deref() else {
            return Self::recent_storage_unavailable_response(request_id, "workspace.recent.clear");
        };
        match workspace::clear_recent_workspaces_in(global_storage) {
            Ok(workspaces) => {
                JsonRpcResponse::success(request_id, json!(RecentWorkspacesResult { workspaces }))
            }
            Err(error) => recent_workspace_error_response(request_id, &error),
        }
    }

    /// Rascunhos que DIFEREM do disco (recuperáveis após um crash) e apaga os
    /// obsoletos (já salvos). Chamado no `workspace.open` (DocsPublic/seguranca/23, M-S1).
    fn recover_drafts(&self) -> Vec<DraftInfo> {
        let Some(store) = self.drafts.as_ref() else {
            return Vec::new();
        };
        let Ok(drafts) = store.list() else {
            return Vec::new();
        };
        let mut recovered = Vec::new();
        for draft in drafts {
            let path = Path::new(&draft.path);
            if !path.is_file() {
                // O arquivo sumiu (apagado ou renomeado) desde o autosave. Um
                // rascunho so nasce contra arquivo EXISTENTE (`confine_file`),
                // entao aqui ele so pode estar obsoleto: "recuperar" o buffer
                // de um arquivo que o usuario apagou reabre a aba de um arquivo
                // que nao existe mais.
                drop(store.clear(&draft.path));
                continue;
            }
            let on_disk = std::fs::read_to_string(path).ok();
            if on_disk.as_deref() == Some(draft.content.as_str()) {
                // Buffer já salvo em disco → rascunho obsoleto, limpa.
                drop(store.clear(&draft.path));
            } else {
                recovered.push(DraftInfo {
                    path: draft.path,
                    content: draft.content,
                    saved_at: draft.saved_at,
                });
            }
        }
        recovered
    }

    /// Persiste as abas abertas/aba ativa em `.kinein/session.json`.
    pub(crate) fn save_session_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "workspace.saveSession");
        };
        let parsed = match parse_params::<WorkspaceSaveSessionParams>(
            request_id.as_ref(),
            params,
            "workspace.saveSession requer o campo openFiles",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        match workspace::save_session(&root, &parsed.open_files, parsed.active_file.as_deref()) {
            Ok(files) => {
                JsonRpcResponse::success(request_id, json!(WorkspaceSaveSessionResult { files }))
            }
            Err(error) => fs_error_response(request_id, &error),
        }
    }

    /// Estado por-workspace que TODA transicao tem de trocar junto.
    ///
    /// Este metodo existe para que a lista abaixo tenha UM dono. Enquanto ela
    /// estava copiada em `workspace.open`, `workspace.createProject` e
    /// `workspace.close`, cada copia esquecia uma peca diferente: a store de
    /// rascunhos (`self.drafts`) so era trocada no `open`, entao
    /// `workspace.createProject` deixava o projeto novo escrevendo autosave no
    /// banco do projeto ANTERIOR — ou sem autosave nenhum, respondendo
    /// "persistencia local de rascunhos indisponivel", quando nao havia
    /// projeto anterior. Falha silenciosa: a rede de seguranca de dados
    /// (DocsPublic/seguranca/23) desligava sem ninguem reclamar.
    ///
    /// Regra: quem acrescentar estado por-workspace ao `Core` acrescenta a
    /// troca dele AQUI, e em lugar nenhum mais. Verificado por
    /// `scripts/verificar-transicao-workspace.sh`.
    fn activate_workspace(&mut self, opened: &WorkspaceInfo) {
        if let Some(debug) = self.debug.as_mut() {
            debug.clear();
        }
        self.workspace = Some(opened.clone());
        // O modelo do projeto embarcado (pilar 0 do roadmaps/42) nasce com o
        // workspace — por qualquer porta: open, createProject — e vai por
        // evento: quem quiser ja' o tem antes de perguntar.
        self.emit_project_changed();
        let root = PathBuf::from(&opened.root);
        self.syntax.clear();
        self.workspace_edits.clear();
        self.reset_workspace_watcher(&root);
        // E o indice do projeto INTEIRO comeca a ser construido agora, em job:
        // a IDE le todas as pastas, arquivos e declaracoes sem esperar LSP.
        // As pastas que ele caminha entram no watcher quando o
        // event.index.finished passar pelo loop — depois disto tudo.
        self.start_index_build(Path::new(&opened.root));
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.set_root(Some(root.clone()));
        }
        // O clangd deste workspace aprende o compilador CROSS do kit: sem o
        // `--query-driver`, um `.cpp` de bare metal fica com os cabecalhos de
        // libstdc++ do GCC ARM em vermelho (medido em 2026-09-11). Vale na
        // PROXIMA subida do servidor cpp — o primeiro `.c/.cpp` aberto.
        self.configure_clangd_from_toolchain(&root);
        // E o basedpyright aprende o INTERPRETADOR do projeto (fatia 2 da
        // cadeia Python): sem ele, completar e tipos vem da stdlib errada.
        self.configure_python_lsp(&root);
        // M-S1: store local de rascunhos, uma por workspace (DocsPublic/seguranca/23).
        self.drafts = self
            .global_storage
            .as_ref()
            .and_then(|_| db::DraftStore::open(&root));
        if let Some(global_storage) = self.global_storage.as_deref() {
            drop(workspace::record_recent_workspace_in(
                global_storage,
                opened,
            ));
        }
    }

    /// Inverso de [`Self::activate_workspace`]: solta tudo que era do workspace
    /// que sai. Devolve o que estava aberto.
    fn deactivate_workspace(&mut self) -> Option<WorkspaceInfo> {
        if let Some(debug) = self.debug.as_mut() {
            debug.clear();
        }
        let closed = self.workspace.take();
        self.fswatch = None;
        self.syntax.clear();
        self.workspace_edits.clear();
        self.drafts = None;
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.set_root(None);
        }
        closed
    }

    pub(crate) fn close_workspace_response(
        &mut self,
        request_id: Option<Value>,
    ) -> JsonRpcResponse {
        let closed = self.deactivate_workspace();
        if let Some(runner) = self.run.as_mut() {
            drop(runner.stop());
        }
        if let Some(session) = self.terminal.as_mut() {
            session.close_all();
        }
        JsonRpcResponse::success(
            request_id,
            json!({
                "status": "ok",
                "closed": closed.map(|workspace| workspace.root),
            }),
        )
    }

    pub(crate) fn browse_workspace_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceBrowseParams>(params) {
            Ok(params) => match workspace::browse_directories(Path::new(&params.path)) {
                Ok(result) => JsonRpcResponse::success(request_id, json!(result)),
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.browse requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn create_workspace_folder_response(
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateFolderParams>(params) {
            Ok(params) => {
                match workspace::create_directory(Path::new(&params.parent), &params.name) {
                    Ok(path) => JsonRpcResponse::success(
                        request_id,
                        json!(WorkspaceCreateFolderResult {
                            path: path.display().to_string(),
                        }),
                    ),
                    Err(error) => workspace_error_response(request_id, &error),
                }
            }
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createFolder requer parent e name",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn create_workspace_project_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceCreateProjectParams>(params) {
            Ok(params) => match workspace::create_project(
                Path::new(&params.parent),
                &params.name,
                params.template,
            ) {
                Ok(opened) => {
                    self.activate_workspace(&opened);
                    JsonRpcResponse::success(request_id, json!(opened))
                }
                Err(error) => workspace_error_response(request_id, &error),
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.createProject requer parent, name e template",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }

    pub(crate) fn open_workspace_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let params = params.cloned().unwrap_or_else(|| json!({}));
        match serde_json::from_value::<WorkspaceOpenParams>(params) {
            Ok(params) => match workspace::open_workspace(Path::new(&params.path)) {
                Ok(opened) => {
                    self.activate_workspace(&opened);
                    // Recupera buffers não salvos que sobreviveram a um crash
                    // da UI (DocsPublic/seguranca/23). Só o `open` recupera: um
                    // projeto recém-criado não tem rascunho anterior.
                    let recovered = self.recover_drafts();
                    let session = workspace::load_session(Path::new(&opened.root));
                    let mut result = json!(opened);
                    if let Some(map) = result.as_object_mut() {
                        if let Some(session) = session {
                            map.insert("session".to_owned(), json!(session));
                        }
                        if !recovered.is_empty() {
                            map.insert("drafts".to_owned(), json!(recovered));
                        }
                    }
                    JsonRpcResponse::success(request_id, result)
                }
                Err(error) => {
                    let code = if error.is_invalid_path() {
                        JsonRpcErrorCode::InvalidParams
                    } else {
                        JsonRpcErrorCode::InternalError
                    };
                    JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            code,
                            error.to_string(),
                            Some(json!({ "path": params.path })),
                        ),
                    )
                }
            },
            Err(error) => JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "workspace.open requer params com o campo path",
                    Some(json!({ "error": error.to_string() })),
                ),
            ),
        }
    }
}
