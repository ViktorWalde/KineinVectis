//! O KIT chega aos language servers (`impl Core`): o compilador cross ao
//! clangd (`--query-driver`) e o alvo ao rust-analyzer (`cargo.target`).
//!
//! Arquivo proprio desde 2026-09-17 (P0 do 40 §4.1): o `handlers/workspace.rs`
//! chegou a 518 linhas com o rust-analyzer, e "o kit muda o que os servidores
//! sabem" e' uma responsabilidade — chamada ao abrir o workspace e a cada
//! `toolchain.set`/`toolchain.setKit`.

use serde_json::{Value, json};

use crate::Core;

impl Core {
    /// Ensina o clangd a entender o compilador cross do kit, via
    /// `--query-driver`. Resolve o toolchain e atualiza a tabela do LSP; o
    /// servidor JA' em execucao nao e' trocado (a troca so' vale na proxima
    /// subida), que e' o mesmo contrato de [`LspManager::use_server_command`].
    pub(crate) fn configure_clangd_from_toolchain(&mut self, root: &std::path::Path) {
        let toolchain = crate::toolchain::Toolchain::resolve(root, &self.detected_tools());
        let args = toolchain.clangd_args();
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.use_language_server_command("cpp", "clangd", &refs);
        self.configure_rust_lsp_from_toolchain(toolchain.target_triple());
    }

    /// O alvo do kit chega ao rust-analyzer (P0 do 40 §4.1, 2026-09-17): com
    /// `targetTriple` no kit, `rust-analyzer.cargo.target = <triple>` — o
    /// `cargo build` do kit ja' levava `--target` (`build/mod.rs::aplica_alvo`);
    /// o servidor checando para o HOST daria diagnostico do host, que parece
    /// certo e nao e'. A chave e' a do manual (rust-analyzer.github.io/book/
    /// configuration, conferida em 2026-09-17: "Compilation target override
    /// (target tuple)"), entregue como o basedpyright recebe a dele — secao
    /// `rust-analyzer` no `workspace/configuration` e no
    /// `didChangeConfiguration`. Vale na PROXIMA subida; um servidor vivo e'
    /// reiniciado, porque o alvo muda o que ele compila.
    pub(crate) fn configure_rust_lsp_from_toolchain(&mut self, target_triple: Option<&str>) {
        let settings = target_triple.filter(|t| !t.trim().is_empty()).map_or(
            Value::Null,
            |triple| json!({ "rust-analyzer": { "cargo": { "target": triple } } }),
        );
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.use_server_settings("rust", settings);
            if lsp.is_running("rust") {
                lsp.restart_language("rust");
            }
        }
    }
}
