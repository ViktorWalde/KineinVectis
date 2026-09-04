//! File-name search delegated to `fd`, confined to the workspace root.

use std::{io, path::Path, process::Command};

use kinein_protocol::FsFileMatch;

use super::confine::{confine, confine_file};
use super::{FsError, MAX_FILE_MATCHES, SEARCH_SKIP_DIRS};

/// Finds files by name using `fd`, confined to the workspace root.
///
/// `fd` is intentionally used instead of a custom indexer: it is mature,
/// fast, and respects ignore files. Results are relative to `root`.
pub fn find_files(root: &Path, query: &str) -> Result<(Vec<FsFileMatch>, bool), FsError> {
    match find_files_with_binary(root, query, Path::new("fd")) {
        Err(FsError::MissingTool { .. }) => {
            find_files_with_binary(root, query, Path::new("fdfind"))
        }
        result => result,
    }
}

fn find_files_with_binary(
    root: &Path,
    query: &str,
    binary: &Path,
) -> Result<(Vec<FsFileMatch>, bool), FsError> {
    let root = confine(root, root)?;
    if !root.is_dir() {
        return Err(FsError::NotADirectory {
            path: root.display().to_string(),
        });
    }

    let mut command = Command::new(binary);
    command
        .arg("--type")
        .arg("f")
        .arg("--fixed-strings")
        .arg("--hidden")
        .arg("--color")
        .arg("never");
    for skipped in SEARCH_SKIP_DIRS {
        command.arg("--exclude").arg(skipped);
    }
    command.arg(query).arg(".").current_dir(&root);

    let output = command.output().map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            FsError::MissingTool { tool: "fd" }
        } else {
            FsError::Io {
                path: root.display().to_string(),
                source,
            }
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = stderr
            .lines()
            .chain(stdout.lines())
            .map(str::trim)
            .find(|line| !line.is_empty())
            .unwrap_or("sem detalhe")
            .to_owned();
        return Err(FsError::ToolFailed {
            tool: "fd",
            message: detail,
        });
    }

    let mut matches = Vec::new();
    let mut truncated = false;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let relative = strip_cwd_prefix(line.trim());
        if relative.is_empty() {
            continue;
        }
        let candidate = root.join(relative);
        if confine_file(&root, &candidate).is_err() {
            continue;
        }
        matches.push(FsFileMatch {
            path: relative.to_owned(),
            name: Path::new(relative).file_name().map_or_else(
                || relative.to_owned(),
                |name| name.to_string_lossy().into_owned(),
            ),
        });
        if matches.len() == MAX_FILE_MATCHES {
            truncated = true;
            break;
        }
    }

    matches.sort_by_key(|entry| entry.path.to_lowercase());
    Ok((matches, truncated))
}

/// Tira o `./` que o `fd` poe na frente quando recebe um caminho explicito.
///
/// POR QUE ESTA FUNCAO EXISTE, e o motivo e' caro (2026-09-04). Ate' hoje o
/// core passava `--strip-cwd-prefix` para o `fd` pedir isso a ele. No `fd`
/// 10.4.2 essa opcao passou a aceitar valor (`--strip-cwd-prefix[=<when>]`) e
/// virou INCOMPATIVEL com passar um caminho — que e' exatamente o que o core
/// faz. O resultado, medido:
///
/// ```text
/// fs.findFiles -> INTERNAL_ERROR
///   "fd falhou: error: the argument '--strip-cwd-prefix[=<when>]' cannot be
///    used with '[path]...'"
/// ```
///
/// **Toda busca por nome de arquivo da IDE falhava**, e o gate estava verde: o
/// teste desta funcao roda um `fd` FALSO, um script que imprime linhas fixas.
/// Fixture inventada nao ve' mudanca de CLI — foi a mesma licao do `probe.rs`
/// em 2026-09-03, e ela custou de novo.
///
/// A saida nao e' voltar a pedir a opcao certa para cada versao: e' **nao
/// depender dela**. Normalizar aqui funciona com `fd` que prefixa e com `fd`
/// que nao prefixa, hoje e depois.
#[must_use]
fn strip_cwd_prefix(line: &str) -> &str {
    line.strip_prefix("./").unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{find_files_with_binary, strip_cwd_prefix};

    fn temp_root(test_name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-fsops-find-tests")
            .join(format!("{}-{test_name}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    #[cfg(unix)]
    /// O `fd` 10.4.2 prefixa `./` quando recebe um caminho; versoes com
    /// `--strip-cwd-prefix` nao prefixavam. Normalizar aqui e' o que faz o
    /// core sobreviver as duas — ver o cabecalho de `strip_cwd_prefix`.
    #[test]
    fn o_prefixo_do_fd_some_com_ou_sem_ele() {
        assert_eq!(strip_cwd_prefix("./src/main.cpp"), "src/main.cpp");
        assert_eq!(strip_cwd_prefix("src/main.cpp"), "src/main.cpp");
        // Um caminho que COMECA com ponto nao pode perder o ponto.
        assert_eq!(strip_cwd_prefix(".config/x"), ".config/x");
        assert_eq!(strip_cwd_prefix(""), "");
    }

    #[test]
    fn find_files_uses_fd_output_and_confines_results() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_root("find-files");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("README.md"), "# demo\n").unwrap();

        let fd = root.join("fake-fd");
        fs::write(
            &fd,
            "#!/bin/sh\nprintf 'src/main.rs\\nREADME.md\\n../escape.rs\\n'\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&fd).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fd, permissions).unwrap();

        // Testes rodam em paralelo e outros deles fazem fork/exec; um filho
        // pode herdar por instantes o fd de escrita do script acima e o
        // primeiro exec falha com ETXTBSY (race classico de Unix). Retentar
        // poucas vezes elimina o flake sem mascarar erro real.
        let mut outcome = find_files_with_binary(&root, "main", &fd);
        for _attempt in 0..20 {
            let is_busy = matches!(
                &outcome,
                Err(super::FsError::Io { source, .. })
                    if source.raw_os_error() == Some(26)
            );
            if !is_busy {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            outcome = find_files_with_binary(&root, "main", &fd);
        }
        let (matches, truncated) = outcome.unwrap();
        let paths = matches
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>();

        assert!(!truncated);
        assert_eq!(paths, ["README.md", "src/main.rs"]);
        assert_eq!(matches[1].name, "main.rs");
    }
}
