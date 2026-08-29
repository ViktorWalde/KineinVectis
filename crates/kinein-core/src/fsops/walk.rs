//! Caminhada deterministica sobre os arquivos de texto do workspace.
//!
//! `fs.search` e `fs.replace` tem de enxergar **exatamente o mesmo conjunto de
//! arquivos**: o usuario le o resultado da busca e manda substituir. Enquanto
//! cada um tinha a sua copia do walk, essa igualdade era uma coincidencia
//! mantida a mao — e as copias ja divergiam num ponto (diretorio ilegivel
//! abortava o `replace` inteiro e era apenas pulado no `search`). Um walk,
//! um dono.
//!
//! A ordem e' depth-first, por nome em caixa baixa: raiz antes das subpastas,
//! subpastas em ordem alfabetica. Determinismo importa porque o resultado vai
//! para uma lista que o usuario le e para um `replace` que precisa ser
//! reproduzivel.

use std::{fs, path::Path, path::PathBuf};

use super::{FsError, SEARCH_SKIP_DIRS};

/// Percorre `root` e entrega cada arquivo elegivel a `visitar`.
///
/// Pula silenciosamente: diretorios de [`SEARCH_SKIP_DIRS`] (VCS, cache, saida
/// de build), symlinks (evita ciclo e escape do root) e diretorios ilegiveis
/// que nao sejam a raiz. So a raiz ilegivel e' erro: o pedido inteiro nao tem
/// como ser atendido.
///
/// `visitar` devolve `true` para PARAR o walk (limite atingido). O retorno de
/// [`walk_text_files`] e' `true` quando parou por esse motivo.
pub(super) fn walk_text_files<F>(root: &Path, mut visitar: F) -> Result<bool, FsError>
where
    F: FnMut(&Path) -> bool,
{
    let mut pendentes = vec![root.to_path_buf()];

    while let Some(diretorio) = pendentes.pop() {
        let Ok(entradas) = fs::read_dir(&diretorio) else {
            if diretorio == root {
                return Err(FsError::NotADirectory {
                    path: diretorio.display().to_string(),
                });
            }
            continue;
        };

        let mut arquivos: Vec<(String, PathBuf)> = Vec::new();
        let mut subdiretorios: Vec<(String, PathBuf)> = Vec::new();
        for entrada in entradas.flatten() {
            let Ok(tipo) = entrada.file_type() else {
                continue;
            };
            if tipo.is_symlink() {
                continue;
            }
            let nome = entrada.file_name().to_string_lossy().to_lowercase();
            if tipo.is_dir() {
                if !SEARCH_SKIP_DIRS.contains(&nome.as_str()) {
                    subdiretorios.push((nome, entrada.path()));
                }
            } else if tipo.is_file() {
                arquivos.push((nome, entrada.path()));
            }
        }

        arquivos.sort_by(|esquerda, direita| esquerda.0.cmp(&direita.0));
        // Ordem inversa: a pilha desempilha do fim, entao o menor nome sai primeiro.
        subdiretorios.sort_by(|esquerda, direita| direita.0.cmp(&esquerda.0));
        pendentes.extend(subdiretorios.into_iter().map(|(_nome, caminho)| caminho));

        for (_nome, arquivo) in arquivos {
            if visitar(&arquivo) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::walk_text_files;

    fn temp_root(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-fsops-walk-tests")
            .join(format!("{}-{nome}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    fn visitados(root: &std::path::Path) -> Vec<String> {
        let mut vistos = Vec::new();
        let parou = walk_text_files(root, |arquivo| {
            vistos.push(
                arquivo
                    .strip_prefix(root)
                    .unwrap_or(arquivo)
                    .display()
                    .to_string(),
            );
            false
        })
        .unwrap();
        assert!(!parou);
        vistos
    }

    #[test]
    fn walk_is_depth_first_in_lowercase_name_order() {
        let root = temp_root("ordem");
        fs::create_dir(root.join("zeta")).unwrap();
        fs::create_dir(root.join("Alfa")).unwrap();
        fs::write(root.join("zeta/z.txt"), "x").unwrap();
        fs::write(root.join("Alfa/a.txt"), "x").unwrap();
        fs::write(root.join("raiz.txt"), "x").unwrap();

        assert_eq!(visitados(&root), ["raiz.txt", "Alfa/a.txt", "zeta/z.txt"]);
    }

    #[test]
    fn walk_skips_build_dirs_and_symlinks() {
        let root = temp_root("pula");
        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/gerado.rs"), "x").unwrap();
        fs::write(root.join("fonte.rs"), "x").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(root.join("fonte.rs"), root.join("link.rs")).unwrap();

        assert_eq!(visitados(&root), ["fonte.rs"]);
    }

    #[test]
    fn walk_reports_only_an_unreadable_root_as_error() {
        let root = temp_root("raiz-ausente");
        let ausente = root.join("nao-existe");

        let erro = walk_text_files(&ausente, |_| false);

        assert!(erro.is_err());
    }

    #[test]
    fn walk_stops_when_the_visitor_asks() {
        let root = temp_root("parada");
        fs::write(root.join("a.txt"), "x").unwrap();
        fs::write(root.join("b.txt"), "x").unwrap();

        let mut vistos = 0_usize;
        let parou = walk_text_files(&root, |_| {
            vistos += 1;
            true
        })
        .unwrap();

        assert!(parou);
        assert_eq!(vistos, 1);
    }
}
