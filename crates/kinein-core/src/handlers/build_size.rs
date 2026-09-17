//! `build.size` (`impl Core`): quanto o ELF ocupa de nao-volatil e de RAM.
//!
//! Arquivo proprio (bloco E, 2026-09-17): o `handlers/build.rs` passou de
//! 500 com os motores de framework, e medir o ELF e' outra responsabilidade
//! que subir o job de build — sincrono, sem job, sem processo longo.

use std::path::PathBuf;

use kinein_protocol::{BuildSizeParams, JsonRpcError, JsonRpcErrorCode, JsonRpcResponse};
use serde_json::{Value, json};

use crate::rpc::{no_workspace_response, parse_params};
use crate::{Core, dap, size};

impl Core {
    /// `build.size` — quanto o ELF ocupa de nao-volatil e de RAM.
    ///
    /// Sincrono: o `size` le um arquivo e volta em milissegundos, ao contrario
    /// do build. Resolve o ELF como o `debug.start` (ou aceita `program`), o
    /// prefixo das binutils vem do cross do kit, e o linker script e' o unico
    /// `.ld` do workspace quando ha' exatamente um — zero ou varios deixa as
    /// regioes vazias e a UI mostra so' os totais, sem adivinhar qual `.ld` vale.
    pub(crate) fn build_size_response(
        &self,
        request_id: Option<Value>,
        params: Option<&Value>,
    ) -> JsonRpcResponse {
        let Some(workspace) = self.workspace.clone() else {
            return no_workspace_response(request_id, "build.size");
        };
        let parsed = match parse_params::<BuildSizeParams>(
            request_id.as_ref(),
            params,
            "build.size aceita apenas o campo opcional program",
        ) {
            Ok(parsed) => parsed,
            Err(response) => return *response,
        };
        let root = PathBuf::from(&workspace.root);
        let program = match parsed.program.filter(|p| !p.trim().is_empty()) {
            Some(explicit) => {
                let path = PathBuf::from(explicit);
                if !path.is_file() {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(
                            JsonRpcErrorCode::InvalidParams,
                            format!("programa nao encontrado: {}", path.display()),
                            None,
                        ),
                    );
                }
                path
            }
            None => match dap::resolve_program(workspace.kind, &root).and_then(|alvo| {
                alvo.program_path()
                    .map(std::path::Path::to_path_buf)
                    .ok_or_else(|| dap::DebugError::NoTarget {
                        message: "o alvo e' um modulo Python, nao um ELF".to_owned(),
                    })
            }) {
                Ok(program) => program,
                Err(error) => {
                    return JsonRpcResponse::failure(
                        request_id,
                        JsonRpcError::new(JsonRpcErrorCode::InvalidParams, error.to_string(), None),
                    );
                }
            },
        };
        let toolchain = crate::toolchain::Toolchain::resolve(&root, &self.detected_tools());
        let prefixo = toolchain.binutils_prefix();
        let linker = unico_linker_script(&root);
        let mut relatorio = size::measure(&program, prefixo.as_deref(), linker.as_deref());
        // ESP-IDF: a flash e' a particao `app` que a receita de gravacao aponta,
        // e o usado e' a IMAGEM que vai para ela (pilar 0 do roadmaps/42). Sem
        // receita ou sem tabela, nada e' acrescentado — e a UI mostra os totais.
        let modelo = self.compute_project_model(&root);
        if let (Some(receita), Some(tabela)) =
            (modelo.artifacts.flash_recipe, modelo.artifacts.partitions)
        {
            let app = receita
                .files
                .iter()
                .find(|f| f.name.as_deref() == Some("app"));
            if let Some(app) = app {
                if let Ok(meta) = std::fs::metadata(&app.file) {
                    if let Some(regiao) =
                        size::region_from_partition(&tabela.entries, app.offset, meta.len())
                    {
                        relatorio.regions.push(regiao);
                    }
                }
            }
        }
        JsonRpcResponse::success(request_id, json!(relatorio))
    }
}

/// O unico `.ld` do workspace, quando ha' exatamente um.
///
/// Procura na raiz e um nivel abaixo (onde moram `linker/`, `ld/`, `boards/`),
/// sem descer na arvore inteira — um `.ld` de dependencia em `build/` nao e' o
/// do projeto. Zero ou mais de um devolve `None`: escolher entre varios seria
/// adivinhar (roadmaps/35 §5.6), e a UI entao mostra os totais sem a barra.
fn unico_linker_script(root: &std::path::Path) -> Option<PathBuf> {
    let mut achados = Vec::new();
    let mut olhar = |dir: &std::path::Path| {
        if let Ok(entradas) = std::fs::read_dir(dir) {
            for entrada in entradas.flatten() {
                let caminho = entrada.path();
                if caminho.extension().and_then(|e| e.to_str()) == Some("ld") {
                    achados.push(caminho);
                }
            }
        }
    };
    olhar(root);
    if let Ok(entradas) = std::fs::read_dir(root) {
        for entrada in entradas.flatten() {
            if entrada.file_type().is_ok_and(|t| t.is_dir()) {
                olhar(&entrada.path());
            }
        }
    }
    achados.sort();
    achados.dedup();
    match achados.as_slice() {
        [unico] => Some(unico.clone()),
        _ => None,
    }
}
