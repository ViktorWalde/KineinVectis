//! Descoberta e diagnóstico da compilation database do C/C++.
//!
//! # Por que este módulo não é do domínio `cmake`
//!
//! A pergunta que ele responde é *"este workspace tem uma compilation database
//! utilizável, onde, e ela está velha?"* — e isso vale para `CMake`, Meson, `bear`
//! sobre um Makefile ou uma CDB escrita à mão. Pendurar no `cmake` obrigaria um
//! ramo por build system, que é exatamente o que o `tools.rs` evita: **detectar
//! é agnóstico.**
//!
//! # O que o clangd já faz sozinho — medido na fonte (2026-08-30)
//!
//! O clangd procura `compile_commands.json` **nos diretórios pai do arquivo
//! editado e em subdiretórios `build/`**: ao editar `$SRC/gui/window.cpp` ele
//! busca em `$SRC/gui/`, `$SRC/gui/build/`, `$SRC/`, `$SRC/build/`, …
//! (<https://clangd.llvm.org/installation>).
//!
//! **A consequência é que este módulo NÃO existe para apontar o clangd.** Um
//! projeto Meson com `build/compile_commands.json` já funciona hoje, sem uma
//! linha nossa. Ele existe para o que o clangd não faz: **dizer ao usuário** se
//! há CDB, de onde ela veio e se envelheceu. Sem isso, um projeto sem CDB dá
//! erro de include em tudo e a IDE fica muda.
//!
//! As duas exceções à busca do clangd, e é por elas que a lista abaixo começa
//! como começa: `<root>/.kinein/build/` não é `$SRC/build/` (daí o
//! `--compile-commands-dir` no `lsp/server.rs`), e `builddir/` — nome comum de
//! Meson — não é procurado.

use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Nome do arquivo, fixado pela especificação da JSON Compilation Database.
const ARQUIVO: &str = "compile_commands.json";

/// Diretórios procurados, em ordem de precedência.
///
/// A ordem não é arbitrária: a CDB que a **IDE** gerou vem primeiro porque é a
/// única que sabemos estar casada com o configure mais recente. As demais são
/// convenções de build system, e `""` é a própria raiz (muita gente cria um
/// symlink `compile_commands.json` ali justamente para o clangd achar).
const DIRETORIOS: &[&str] = &[".kinein/build", "", "build", "builddir", "out/build"];

/// Fontes que podem invalidar uma CDB: se alguma for mais nova que ela, as
/// flags no disco descrevem um projeto que não existe mais.
const FONTES_DE_INVALIDACAO: &[&str] = &[
    "CMakeLists.txt",
    "CMakePresets.json",
    "CMakeUserPresets.json",
    "meson.build",
    "Makefile",
];

/// Diagnóstico da compilation database de um workspace.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CdbStatus {
    /// Diretório que contém a CDB, relativo ao root; `None` quando não há.
    pub directory: Option<String>,
    /// A CDB é mais velha que algum arquivo de build que a define.
    pub stale: bool,
    /// Arquivo que tornou a CDB obsoleta, quando `stale`.
    pub stale_because: Option<String>,
}

impl CdbStatus {
    /// Não há compilation database alcançável neste workspace.
    #[must_use]
    pub const fn ausente() -> Self {
        Self {
            directory: None,
            stale: false,
            stale_because: None,
        }
    }
}

/// Procura a CDB e diz se ela envelheceu.
///
/// Nunca falha: um workspace sem CDB é um estado legítimo (projeto Rust, ou
/// C/C++ ainda não configurado), não um erro.
#[must_use]
pub fn status(root: &Path) -> CdbStatus {
    let Some((diretorio, arquivo)) = encontrar(root) else {
        return CdbStatus::ausente();
    };
    let (stale, stale_because) = envelheceu(root, &arquivo);
    CdbStatus {
        directory: Some(diretorio),
        stale,
        stale_because,
    }
}

/// Primeiro diretório de [`DIRETORIOS`] que contém uma CDB legível.
///
/// Devolve o caminho RELATIVO ao root (o que a UI mostra) e o absoluto (o que
/// o `stat` usa). Um `compile_commands.json` que existe mas não é um arquivo
/// regular é ignorado como se não estivesse lá.
fn encontrar(root: &Path) -> Option<(String, PathBuf)> {
    DIRETORIOS.iter().find_map(|relativo| {
        let diretorio = if relativo.is_empty() {
            root.to_path_buf()
        } else {
            root.join(relativo)
        };
        let arquivo = diretorio.join(ARQUIVO);
        arquivo.is_file().then(|| {
            let rotulo = if relativo.is_empty() {
                ".".to_owned()
            } else {
                (*relativo).to_owned()
            };
            (rotulo, arquivo)
        })
    })
}

/// A CDB é mais velha que alguma fonte que a define?
///
/// Compara mtime. Se o mtime da CDB não puder ser lido, a resposta é "não
/// envelheceu": **um diagnóstico que não sabe não deve gritar.** Aviso falso
/// ensina a ignorar aviso (`ARCHITECTURE.md` §4 regra 11).
fn envelheceu(root: &Path, cdb: &Path) -> (bool, Option<String>) {
    let Some(cdb_em) = modificado_em(cdb) else {
        return (false, None);
    };
    for fonte in FONTES_DE_INVALIDACAO {
        let caminho = root.join(fonte);
        let Some(fonte_em) = modificado_em(&caminho) else {
            continue;
        };
        if fonte_em > cdb_em {
            return (true, Some((*fonte).to_owned()));
        }
    }
    (false, None)
}

fn modificado_em(caminho: &Path) -> Option<SystemTime> {
    std::fs::metadata(caminho).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, thread, time::Duration};

    use super::{CdbStatus, status};

    fn raiz(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-cdb-tests")
            .join(format!("{}-{nome}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn escrever_cdb(dir: &std::path::Path) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join("compile_commands.json"), "[]\n").unwrap();
    }

    #[test]
    fn workspace_sem_cdb_nao_e_erro() {
        let root = raiz("ausente");

        assert_eq!(status(&root), CdbStatus::ausente());
    }

    #[test]
    fn encontra_a_cdb_na_raiz_e_no_build() {
        let root = raiz("na-raiz");
        escrever_cdb(&root);
        assert_eq!(status(&root).directory.as_deref(), Some("."));

        let root = raiz("no-build");
        escrever_cdb(&root.join("build"));
        assert_eq!(status(&root).directory.as_deref(), Some("build"));

        // `builddir` e' nome comum de Meson e NAO e' procurado pelo clangd.
        let root = raiz("no-builddir");
        escrever_cdb(&root.join("builddir"));
        assert_eq!(status(&root).directory.as_deref(), Some("builddir"));
    }

    /// A CDB da IDE vence: e' a unica que sabemos casada com o ultimo configure.
    #[test]
    fn a_cdb_da_ide_tem_precedencia_sobre_as_convencoes() {
        let root = raiz("precedencia");
        escrever_cdb(&root.join("build"));
        escrever_cdb(&root);
        escrever_cdb(&root.join(".kinein").join("build"));

        assert_eq!(status(&root).directory.as_deref(), Some(".kinein/build"));
    }

    #[test]
    fn cdb_mais_velha_que_o_cmakelists_e_reportada_com_o_culpado() {
        let root = raiz("velha");
        escrever_cdb(&root.join("build"));
        // mtime tem granularidade; garante ordem observavel.
        thread::sleep(Duration::from_millis(20));
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let resultado = status(&root);

        assert!(resultado.stale);
        assert_eq!(resultado.stale_because.as_deref(), Some("CMakeLists.txt"));
    }

    #[test]
    fn cdb_mais_nova_que_as_fontes_nao_e_velha() {
        let root = raiz("nova");
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();
        thread::sleep(Duration::from_millis(20));
        escrever_cdb(&root.join("build"));

        let resultado = status(&root);

        assert!(!resultado.stale);
        assert_eq!(resultado.stale_because, None);
    }

    /// Sem CDB nao existe "velha": stale so faz sentido sobre algo que existe.
    #[test]
    fn ausente_nunca_e_reportada_como_velha() {
        let root = raiz("ausente-nao-e-velha");
        fs::write(root.join("CMakeLists.txt"), "project(x)\n").unwrap();

        let resultado = status(&root);

        assert_eq!(resultado.directory, None);
        assert!(!resultado.stale);
    }
}
