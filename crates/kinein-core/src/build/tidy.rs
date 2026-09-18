//! O `clang-tidy` do projeto inteiro no `quality.run` (D6 do `roadmaps/41`,
//! P5 do 40 §4.1, 2026-09-17): C/C++ deixa de ter so' o clippy do Rust.
//!
//! O que o clangd faz arquivo a arquivo (`--clang-tidy`, `toolchain/
//! arguments.rs`), aqui e' o projeto: `run-clang-tidy -p <pasta da CDB>
//! -quiet` (o script que o LLVM instala ao lado do `clang-tidy`; paraleliza
//! e le a `compile_commands.json`), ou, sem ele, `clang-tidy -p <pasta>
//! <arquivos da CDB>` — os arquivos vem da propria CDB (`file` de cada
//! entrada), nunca de um glob. O `.clang-tidy` do projeto e' lido pela
//! ferramenta sozinha; a IDE nao escolhe checks. A saida e' `arquivo:linha:
//! coluna: warning: mensagem [check]` — o parser gcc-like casa, e o nome do
//! check fica na mensagem.
//!
//! Sem CDB nao ha' tidy: o clang-tidy sem `-p` chutaria flags. A mensagem
//! diz "configure/compile" — o `CMake` escreve a CDB no configure, o Makefile
//! puro com o bear (P0).

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use super::{BuildError, BuildEvent, BuildOutcome, DiagnosticFormat, stream_command};

/// O que o handler resolve para o job de qualidade: os linters desta
/// maquina (o job nao alcanca o detector).
#[derive(Debug, Clone, Default)]
pub struct QualityTools {
    /// O `ruff` detectado (Python).
    pub ruff: Option<PathBuf>,
    /// `run-clang-tidy` detectado (o script do LLVM).
    pub run_clang_tidy: Option<PathBuf>,
    /// `clang-tidy` detectado.
    pub clang_tidy: Option<PathBuf>,
}

/// Os arquivos-fonte da CDB, sem repetir, so' os que existem: e' o que o
/// `clang-tidy -p` recebe quando nao ha' `run-clang-tidy`.
#[must_use]
pub fn cdb_sources(cdb_dir: &Path) -> Vec<PathBuf> {
    let Ok(texto) = std::fs::read_to_string(cdb_dir.join("compile_commands.json")) else {
        return Vec::new();
    };
    let Ok(entradas) = serde_json::from_str::<Vec<serde_json::Value>>(&texto) else {
        return Vec::new();
    };
    let mut arquivos: Vec<PathBuf> = Vec::new();
    for e in &entradas {
        let Some(file) = e.get("file").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let mut caminho = PathBuf::from(file);
        if caminho.is_relative()
            && let Some(dir) = e.get("directory").and_then(serde_json::Value::as_str)
        {
            caminho = Path::new(dir).join(caminho);
        }
        if caminho.is_file() && !arquivos.contains(&caminho) {
            arquivos.push(caminho);
        }
    }
    arquivos
}

pub(super) fn run_clang_tidy(
    root: &Path,
    tools: &QualityTools,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    let cdb = crate::cdb::status(root);
    let Some(dir) = cdb.directory.as_deref() else {
        return Err(BuildError::ToolMissing {
            tool: "compile_commands.json".to_owned(),
            hint: "sem compilation database nao ha' clang-tidy (ele chutaria as flags): \
                   configure o CMake (Configurar) ou compile o Makefile com o bear"
                .to_owned(),
        });
    };
    let cdb_dir = root.join(dir);
    if let Some(script) = &tools.run_clang_tidy {
        let mut command = Command::new(script);
        command.arg("-p").arg(&cdb_dir).arg("-quiet");
        command.current_dir(root);
        return stream_command(
            command,
            &format!("run-clang-tidy -p {dir} -quiet"),
            DiagnosticFormat::GccLike,
            cancel,
            sink,
        );
    }
    let Some(clang_tidy) = &tools.clang_tidy else {
        return Err(BuildError::ToolMissing {
            tool: "clang-tidy".to_owned(),
            hint: "instale o clang-tidy (pacote `clang-tidy` da distro; o painel Instalar \
                   ferramentas mostra o passo)"
                .to_owned(),
        });
    };
    let arquivos = cdb_sources(&cdb_dir);
    if arquivos.is_empty() {
        return Err(BuildError::ToolMissing {
            tool: "compile_commands.json".to_owned(),
            hint: format!("a CDB em {dir} nao lista nenhum arquivo existente: reconfigure"),
        });
    }
    let mut command = Command::new(clang_tidy);
    command.arg("-p").arg(&cdb_dir).args(&arquivos);
    command.current_dir(root);
    stream_command(
        command,
        &format!("clang-tidy -p {dir} ({} arquivos)", arquivos.len()),
        DiagnosticFormat::GccLike,
        cancel,
        sink,
    )
}

#[cfg(test)]
mod tests {
    use super::cdb_sources;

    #[test]
    fn the_sources_come_from_the_cdb_without_repeats_or_ghosts() {
        let raiz = std::env::temp_dir().join(format!("kinein-tidy-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(raiz.join("build")).unwrap();
        std::fs::write(raiz.join("a.cpp"), "").unwrap();
        std::fs::write(raiz.join("b.c"), "").unwrap();
        let cdb = format!(
            r#"[{{"directory":"{r}","file":"{r}/a.cpp","command":"c++ a.cpp"}},
                {{"directory":"{r}","file":"b.c","command":"cc b.c"}},
                {{"directory":"{r}","file":"{r}/a.cpp","command":"c++ -DX a.cpp"}},
                {{"directory":"{r}","file":"{r}/nao.cpp","command":"c++ nao.cpp"}}]"#,
            r = raiz.display()
        );
        std::fs::write(raiz.join("build/compile_commands.json"), cdb).unwrap();
        let fontes = cdb_sources(&raiz.join("build"));
        assert_eq!(fontes, vec![raiz.join("a.cpp"), raiz.join("b.c")]);
        assert!(cdb_sources(&raiz.join("nao")).is_empty());
        let _ = std::fs::remove_dir_all(&raiz);
    }
}
