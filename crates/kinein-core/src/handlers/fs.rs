//! Handlers for `fs.*` requests (`impl Core`): the `fs.*` router plus the leaf
//! read/write/create/rename/delete/search handlers, all confined to the
//! workspace via `crate::fsops`.

use std::path::Path;

use kinein_protocol::{
    FsCreateDirectoryParams, FsCreateDirectoryResult, FsCreateFileParams, FsCreateFileResult,
    FsDeleteResult, FsFindFilesParams, FsFindFilesResult, FsListResult, FsPathParams, FsReadResult,
    FsRenameParams, FsRenameResult, FsReplaceParams, FsReplaceResult, FsSaveParams, FsSearchParams,
    FsSearchResult, FsWriteResult, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse,
};
use serde_json::{Value, json};

use crate::rpc::{fs_error_response, no_workspace_response, parse_params};
use crate::{Core, fsops};

impl Core {
    /// Roteia os metodos `fs.*`; `None` quando o metodo nao e de arquivos.
    pub(crate) fn fs_request_response(
        &mut self,
        method: &str,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> Option<JsonRpcResponse> {
        match method {
            "fs.list" => Some(self.fs_list_response(request_id, params)),
            "fs.read" => Some(self.fs_read_response(request_id, params)),
            "fs.createFile" => Some(self.fs_create_file_response(request_id, params)),
            "fs.createDirectory" => Some(self.fs_create_directory_response(request_id, params)),
            "fs.write" => Some(self.fs_write_response(request_id, params)),
            "fs.rename" => Some(self.fs_rename_response(request_id, params)),
            "fs.delete" => Some(self.fs_delete_response(request_id, params)),
            "fs.findFiles" => Some(self.fs_find_files_response(request_id, params)),
            "fs.search" => Some(self.fs_search_response(request_id, params)),
            "fs.replace" => Some(self.fs_replace_response(request_id, params)),
            _ => None,
        }
    }

    fn fs_replace_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.replace");
        };
        let parsed = match parse_params::<FsReplaceParams>(
            request_id.as_ref(),
            params,
            "fs.replace requer query, replacement e caseSensitive opcional",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        if parsed.query.is_empty() {
            return JsonRpcResponse::failure(
                request_id,
                JsonRpcError::new(
                    JsonRpcErrorCode::InvalidParams,
                    "fs.replace requer query nao vazia",
                    None,
                ),
            );
        }
        // A RECUSA de quebra de linha caiu em 2026-09-02 (etapa 9 do
        // `roadmaps/30`), e caiu pelo motivo certo: ela existia porque a busca
        // casava LINHA A LINHA enquanto a substituicao casava no conteudo
        // inteiro, entao uma query com `\n` era invisivel para o preview e
        // ativa para a escrita. O usuario via "0 resultados", mandava
        // substituir, e arquivos eram reescritos.
        //
        // O que mudou nao foi a guarda — foi a busca. Agora as duas casam no
        // MESMO texto, com o mesmo `find_literal`, e o preview de um casamento
        // multi-linha mostra as quebras. Remover a guarda antes disso teria
        // sido trocar honestidade por funcionalidade.
        match fsops::replace(
            &root,
            &parsed.query,
            &parsed.replacement,
            parsed.case_sensitive,
        ) {
            Ok((files, replacements)) => {
                self.syntax.clear();
                JsonRpcResponse::success(
                    request_id,
                    json!(FsReplaceResult {
                        files,
                        replacements,
                    }),
                )
            }
            Err(error) => fs_error_response(request_id, &error),
        }
    }

    fn fs_list_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.list");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.list requer o campo path",
        ) {
            Ok(parsed) => match fsops::list_dir(&root, Path::new(&parsed.path)) {
                Ok((path, entries)) => {
                    self.watch_workspace_directory(&path);
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsListResult {
                            path: path.display().to_string(),
                            entries,
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_read_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.read");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.read requer o campo path",
        ) {
            Ok(parsed) => match fsops::read_file(&root, Path::new(&parsed.path)) {
                Ok((path, content)) => {
                    if let Some(parent) = path.parent() {
                        self.watch_workspace_directory(parent);
                    }
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.did_open(&path, &content);
                    }
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsReadResult {
                            path: path.display().to_string(),
                            content,
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_search_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.search");
        };
        match parse_params::<FsSearchParams>(
            request_id.as_ref(),
            params,
            "fs.search requer o campo query",
        ) {
            Ok(parsed) => {
                if parsed.query.is_empty() {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            "fs.search requer query nao vazia",
                            None,
                        ),
                    );
                }
                match fsops::search(&root, &parsed.query, parsed.case_sensitive) {
                    Ok((matches, truncated)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsSearchResult { matches, truncated }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_find_files_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.findFiles");
        };
        match parse_params::<FsFindFilesParams>(
            request_id.as_ref(),
            params,
            "fs.findFiles requer o campo query",
        ) {
            Ok(parsed) => {
                if parsed.query.is_empty() {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            "fs.findFiles requer query nao vazia",
                            None,
                        ),
                    );
                }
                match fsops::find_files(&root, &parsed.query) {
                    Ok((matches, truncated)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsFindFilesResult { matches, truncated }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_create_file_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.createFile");
        };
        match parse_params::<FsCreateFileParams>(
            request_id.as_ref(),
            params,
            "fs.createFile requer o campo path e aceita content opcional",
        ) {
            Ok(parsed) => {
                match fsops::create_file(&root, Path::new(&parsed.path), &parsed.content) {
                    Ok((path, bytes_written)) => JsonRpcResponse::success(
                        request_id,
                        json!(FsCreateFileResult {
                            path: path.display().to_string(),
                            bytes_written,
                        }),
                    ),
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_create_directory_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.createDirectory");
        };
        match parse_params::<FsCreateDirectoryParams>(
            request_id.as_ref(),
            params,
            "fs.createDirectory requer o campo path",
        ) {
            Ok(parsed) => match fsops::create_directory(&root, Path::new(&parsed.path)) {
                Ok(path) => JsonRpcResponse::success(
                    request_id,
                    json!(FsCreateDirectoryResult {
                        path: path.display().to_string(),
                    }),
                ),
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }

    fn fs_write_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.write");
        };
        match parse_params::<FsSaveParams>(
            request_id.as_ref(),
            params,
            "fs.write requer path, content e expectedContent",
        ) {
            Ok(parsed) => {
                match fsops::write_file_if_unchanged(
                    &root,
                    Path::new(&parsed.path),
                    &parsed.content,
                    &parsed.expected_content,
                ) {
                    Ok((path, bytes_written)) => {
                        if let Some(lsp) = self.lsp.as_mut() {
                            lsp.did_save(&path, &parsed.content);
                        }
                        // M-S1: arquivo salvo em disco → rascunho é obsoleto.
                        super::draft::clear_draft_after_save(self, &path);
                        // Num espelho remoto (P6 fatia 2), salvar EMPURRA o
                        // arquivo para o alvo — como job, visivel.
                        self.mirror_push_after_write(&path);
                        JsonRpcResponse::success(
                            request_id,
                            json!(FsWriteResult {
                                path: path.display().to_string(),
                                bytes_written,
                            }),
                        )
                    }
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_rename_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.rename");
        };
        match parse_params::<FsRenameParams>(
            request_id.as_ref(),
            params,
            "fs.rename requer os campos from e to",
        ) {
            Ok(parsed) => {
                match fsops::rename(&root, Path::new(&parsed.from), Path::new(&parsed.to)) {
                    Ok((from, to)) => {
                        // O rascunho acompanha o arquivo: a chave da store e' o
                        // caminho absoluto (DocsPublic/seguranca/23).
                        super::draft::rename_draft_after_move(self, &from, &to);
                        JsonRpcResponse::success(
                            request_id,
                            json!(FsRenameResult {
                                from: from.display().to_string(),
                                to: to.display().to_string(),
                            }),
                        )
                    }
                    Err(error) => fs_error_response(request_id, &error),
                }
            }
            Err(response) => *response,
        }
    }

    fn fs_delete_response(
        &mut self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(root) = self.workspace_root() else {
            return no_workspace_response(request_id, "fs.delete");
        };
        match parse_params::<FsPathParams>(
            request_id.as_ref(),
            params,
            "fs.delete requer o campo path",
        ) {
            Ok(parsed) => match fsops::delete(&root, Path::new(&parsed.path)) {
                Ok(path) => {
                    // Documento apagado que continua aberto no servidor deixa
                    // diagnostico de um arquivo que nao existe mais na aba
                    // Problemas. Fechar e' a unica forma de o servidor
                    // esquece-lo (protocolo 0.63.0).
                    if let Some(lsp) = self.lsp.as_mut() {
                        lsp.did_close(&path);
                    }
                    JsonRpcResponse::success(
                        request_id,
                        json!(FsDeleteResult {
                            path: path.display().to_string(),
                        }),
                    )
                }
                Err(error) => fs_error_response(request_id, &error),
            },
            Err(response) => *response,
        }
    }
}
