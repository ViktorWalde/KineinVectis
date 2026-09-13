//! Depurar Python (fatia 4 da cadeia do `roadmaps/41` bloco B, 2026-09-13):
//! o debugpy e' um MODULO do interpretador do projeto, nao um binario.
//!
//! Por isso o adaptador nao vem do kit nem do `PATH`: o programa e' o
//! interpretador que `python::env` resolve (o mesmo do status, do índice, do
//! basedpyright, do run e do pytest), e o adaptador e' `-m debugpy.adapter`
//! (`dap/adapter.rs`). Antes de subir, este modulo PERGUNTA ao interpretador
//! se o modulo existe — sem isso o adaptador morre no primeiro request e a
//! falha apareceria como timeout do `initialize`, longe da causa.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// O passo para instalar o debugpy NO ambiente do projeto, como a fonte
/// oficial escreve (docs do uv e do pip, lidas em 2026-09-13).
pub const DICA_DE_INSTALACAO: &str = "instale-o NO ambiente do projeto: `uv add --dev debugpy` (projeto do uv) ou \
     `.venv/bin/python -m pip install debugpy`";

/// Tempo maximo para o interpretador responder ao `import` (um python parado
/// no `site` nao pode travar o `debug.start`).
const PRAZO: Duration = Duration::from_secs(10);

/// `Ok(())` se `interpreter` importa `debugpy`.
///
/// Senao, a mensagem que nomeia o modulo, o interpretador e o passo. O
/// `import` roda com `-I` (isolado: sem `PYTHONPATH`, sem site do usuario) —
/// e' o que o adaptador vai ver.
pub fn debugpy_available(interpreter: &Path) -> Result<(), String> {
    let mut child = Command::new(interpreter)
        .args(["-I", "-c", "import debugpy"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("nao consegui executar {}: {e}", interpreter.display()))?;
    let inicio = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if inicio.elapsed() < PRAZO => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                drop(child.kill());
                return Err(format!(
                    "{} nao respondeu ao `import debugpy` em {}s",
                    interpreter.display(),
                    PRAZO.as_secs()
                ));
            }
            Err(e) => return Err(format!("falha ao esperar {}: {e}", interpreter.display())),
        }
    };
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "debugpy ausente em {}: {DICA_DE_INSTALACAO}",
        interpreter.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::debugpy_available;

    #[cfg(unix)]
    fn executavel(dir: &Path, nome: &str, corpo: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let caminho = dir.join(nome);
        std::fs::write(&caminho, corpo).unwrap();
        std::fs::set_permissions(&caminho, std::fs::Permissions::from_mode(0o755)).unwrap();
        caminho
    }

    /// Um "python" que aceita o import, outro que o recusa como o `CPython`
    /// (exit 1 com `No module named debugpy`), e um que nao existe.
    #[test]
    #[cfg(unix)]
    fn the_import_probe_reports_presence_absence_and_a_missing_interpreter() {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-debugpy-probe", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let registro = dir.join("args.txt");
        let com = executavel(
            &dir,
            "python-com",
            &format!("#!/bin/sh\necho \"$@\" > {}\nexit 0\n", registro.display()),
        );
        assert_eq!(debugpy_available(&com), Ok(()));
        assert_eq!(
            std::fs::read_to_string(&registro).unwrap().trim(),
            "-I -c import debugpy",
            "isolado (-I): o que o adaptador vai ver"
        );

        let sem = executavel(
            &dir,
            "python-sem",
            "#!/bin/sh\necho 'No module named debugpy' >&2\nexit 1\n",
        );
        let erro = debugpy_available(&sem).unwrap_err();
        assert!(
            erro.contains("debugpy ausente") && erro.contains("uv add --dev debugpy"),
            "{erro}"
        );
        assert!(
            erro.contains("python-sem"),
            "nomeia o interpretador: {erro}"
        );

        let erro = debugpy_available(&dir.join("nao-existe")).unwrap_err();
        assert!(erro.contains("nao consegui executar"), "{erro}");
    }
}
