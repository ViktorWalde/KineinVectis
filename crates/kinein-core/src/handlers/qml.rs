//! O `qmlls` do projeto (59 §2.4, 2026-10-08).
//!
//! O language server do QML vem com o Qt do usuario, fora do `PATH` nas
//! distros (`tools/search_dirs.rs`). Para conhecer os modulos QML do PROPRIO
//! projeto ele precisa do diretorio de build (`-b`): o canonico da IDE, quando
//! o configure ja' rodou. Sem build, ele sobe e conhece so' os modulos do Qt.

use std::path::Path;

use crate::Core;

impl Core {
    /// Aponta o servidor `qml` para o `qmlls` achado e para o build do projeto.
    /// Vale na proxima subida do servidor; nao inicia processo.
    pub(crate) fn configure_qml_lsp(&mut self, root: &Path) {
        let command = self
            .detector
            .find_in_path("qmlls")
            .map_or_else(|| "qmlls".to_owned(), |p| p.display().to_string());
        let build = crate::cmake::build_dir(root);
        let build_arg = build.display().to_string();
        let args: &[&str] = if build.join("CMakeCache.txt").is_file() {
            &["-b", &build_arg]
        } else {
            &[]
        };
        if let Some(lsp) = self.lsp.as_mut() {
            lsp.use_server_command("qml", &command, args);
        }
    }

    /// O configure terminou: o build pode ter nascido agora. Reconfigura e, se
    /// o `qmlls` esta' vivo, reinicia (a UI reabre os documentos no
    /// `event.lsp.restarted`, como no Python).
    pub(crate) fn on_cmake_finished_for_qml(&mut self) {
        let Some(root) = self.workspace_root() else {
            return;
        };
        self.configure_qml_lsp(&root);
        if let Some(lsp) = self.lsp.as_mut()
            && lsp.is_running("qml")
        {
            lsp.restart_language("qml");
        }
    }
}
