//! C2 do `roadmaps/41` bloco C: os ARQUIVOS NA PLACA de um `MicroPython`,
//! pelo `mpremote fs` (referencia de UX: o "Files on device" do Thonny).
//!
//! Esta unidade e' PURA: monta a linha do `mpremote` e LE o que ele
//! imprimiu; quem roda e' o job (`handlers/serial_files`). Tudo abaixo foi
//! medido no ESP32 real desta maquina (CP2102, `/dev/ttyUSB0`) com o
//! mpremote 1.29.0 em 2026-09-17, e conferido no codigo dele
//! (`mpremote/commands.py::do_filesystem`):
//!
//! ```text
//! $ mpremote connect /dev/ttyUSB0 fs ls            (":" = a raiz; `ls :lib` uma pasta)
//! ls :
//!          139 boot.py                              "{:12} {name}{/ se diretorio}"
//!        13588 main.py
//! $ mpremote connect /dev/ttyUSB0 fs cp :boot.py ./boot.py     (":" = na placa)
//! cp :boot.py ./boot.py
//! $ mpremote connect /dev/ttyUSB0 fs cp :nao.py ./nao.py
//! cp :nao.py ./nao.py
//! mpremote: cp: nao.py: No such file or directory.    (stderr, exit 1)
//! ```
//!
//! # O que a placa faz quando o mpremote conecta
//!
//! Todo `fs` entra no raw REPL: o programa em execucao e' INTERROMPIDO
//! (Ctrl-C) e, ao sair, a placa recebe um soft reset — o `main.py` volta a
//! rodar. Por isso ate' o `ls` e' gesto explicito e `JobRisk::Medium`; os
//! que ESCREVEM (`put`, `rm`, `mkdir`) sao `High` e a tela confirma antes.
//!
//! # A primeira conexao pode falhar
//!
//! Medido em 2026-09-17: com o firmware do autor inundando a UART (1,3 MB
//! de binario em segundos), a primeira tentativa morreu com `could not enter
//! raw repl`; a segunda entrou. O job repete UMA vez nesse caso — e' o
//! programa da placa, nao a IDE, que decide se a segunda entra.

use std::path::Path;

use kinein_protocol::{SerialFileEntry, SerialFilesAction};

/// O que o mpremote imprime quando nao consegue interromper o programa da
/// placa para entrar no raw REPL (`transport_serial.py::enter_raw_repl`).
const RAW_REPL_FAILED: &str = "could not enter raw repl";

/// Valida o pedido ANTES de tocar a porta: o caminho na placa nao pode ter
/// `:` na frente (e' a IDE que o poe), nem quebra de linha; `get`/`put`
/// exigem `local`; `rm`/`mkdir`/`get`/`put` exigem `path`.
pub fn validate(
    action: SerialFilesAction,
    path: Option<&str>,
    local: Option<&Path>,
) -> Result<(), String> {
    let path = path.unwrap_or("").trim();
    if path.starts_with(':') {
        return Err("o caminho na placa vai sem `:` na frente — a IDE o acrescenta".to_owned());
    }
    if path.contains('\n') || path.contains('\r') {
        return Err("o caminho na placa nao pode ter quebra de linha".to_owned());
    }
    if action != SerialFilesAction::List && path.is_empty() {
        return Err(format!(
            "`{}` exige o campo path (o arquivo ou a pasta na placa)",
            action.fs_command()
        ));
    }
    match action {
        SerialFilesAction::Get | SerialFilesAction::Put => {
            let Some(local) = local else {
                return Err(format!(
                    "`{}` exige o campo local (o arquivo nesta maquina)",
                    if action == SerialFilesAction::Get {
                        "get"
                    } else {
                        "put"
                    }
                ));
            };
            if action == SerialFilesAction::Put && !local.is_file() {
                return Err(format!(
                    "nao ha' arquivo em {} para enviar a placa",
                    local.display()
                ));
            }
            if action == SerialFilesAction::Get
                && let Some(pasta) = local.parent()
                && !pasta.as_os_str().is_empty()
                && !pasta.is_dir()
            {
                return Err(format!(
                    "a pasta {} nao existe para receber o arquivo",
                    pasta.display()
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// A linha: `mpremote connect <device> fs <cmd> …`. O `:` marca o lado da
/// placa (no `cp`, os dois lados sao explicitos: `:main.py ./main.py`).
#[must_use]
pub fn command_line(
    mpremote: &Path,
    device: &str,
    action: SerialFilesAction,
    path: &str,
    local: Option<&Path>,
) -> (String, Vec<String>) {
    let mut args = vec![
        "connect".to_owned(),
        device.to_owned(),
        "fs".to_owned(),
        action.fs_command().to_owned(),
    ];
    let na_placa = format!(":{path}");
    let local = local.map(|p| p.display().to_string());
    match action {
        SerialFilesAction::Get => {
            args.push(na_placa);
            args.push(local.unwrap_or_default());
        }
        SerialFilesAction::Put => {
            args.push(local.unwrap_or_default());
            args.push(na_placa);
        }
        SerialFilesAction::List | SerialFilesAction::Rm | SerialFilesAction::Mkdir => {
            args.push(na_placa);
        }
    }
    (mpremote.display().to_string(), args)
}

/// A conexao morreu antes do raw REPL: vale UMA repeticao.
#[must_use]
pub fn raw_repl_failed(raw: &str) -> bool {
    raw.contains(RAW_REPL_FAILED)
}

/// A linha `mpremote: <cmd>: <caminho>: <motivo>.` que o mpremote escreve no
/// stderr ao falhar — a mensagem de erro da tela. Sem ela, a cauda.
#[must_use]
pub fn error_line(raw: &str) -> Option<String> {
    raw.lines()
        .rev()
        .map(str::trim)
        .find(|l| l.starts_with("mpremote: "))
        .map(|l| l.trim_start_matches("mpremote: ").to_owned())
}

/// Le o `fs ls`: `{size:12} {name}[/]`. A linha `ls :<path>` do modo
/// verboso e qualquer outra sao ignoradas; um nome com espacos e' mantido
/// inteiro (o tamanho e' a primeira palavra, o resto e' o nome).
#[must_use]
pub fn parse_ls(raw: &str) -> Vec<SerialFileEntry> {
    raw.lines()
        .filter_map(|linha| {
            let sem_indent = linha.trim_start();
            let (tamanho, nome) = sem_indent.split_once(' ')?;
            let size = tamanho.parse::<u64>().ok()?;
            let nome = nome.trim();
            if nome.is_empty() {
                return None;
            }
            let directory = nome.ends_with('/');
            Some(SerialFileEntry {
                name: nome.trim_end_matches('/').to_owned(),
                size,
                directory,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use kinein_protocol::SerialFilesAction;

    use super::{command_line, error_line, parse_ls, raw_repl_failed, validate};

    /// A saida REAL do ESP32 do autor (2026-09-17), com a linha verbosa.
    const LS_REAL: &str = "ls :\n         139 boot.py\n         785 config.py\n       13588 main.py\n        1137 wifi_lib.py\n";

    #[test]
    fn the_command_line_marks_the_board_side_with_a_colon() {
        let mp = Path::new("/x/mpremote");
        let (p, a) = command_line(mp, "/dev/ttyUSB0", SerialFilesAction::List, "", None);
        assert_eq!(p, "/x/mpremote");
        assert_eq!(a, ["connect", "/dev/ttyUSB0", "fs", "ls", ":"]);
        let (_, a) = command_line(mp, "/dev/ttyUSB0", SerialFilesAction::List, "lib", None);
        assert_eq!(a[3..], ["ls", ":lib"]);
        let (_, a) = command_line(
            mp,
            "/dev/ttyUSB0",
            SerialFilesAction::Get,
            "main.py",
            Some(Path::new("/w/main.py")),
        );
        assert_eq!(a[3..], ["cp", ":main.py", "/w/main.py"]);
        let (_, a) = command_line(
            mp,
            "/dev/ttyUSB0",
            SerialFilesAction::Put,
            "main.py",
            Some(Path::new("/w/main.py")),
        );
        assert_eq!(a[3..], ["cp", "/w/main.py", ":main.py"]);
        let (_, a) = command_line(mp, "/dev/ttyUSB0", SerialFilesAction::Rm, "old.py", None);
        assert_eq!(a[3..], ["rm", ":old.py"]);
        let (_, a) = command_line(mp, "/dev/ttyUSB0", SerialFilesAction::Mkdir, "lib", None);
        assert_eq!(a[3..], ["mkdir", ":lib"]);
    }

    #[test]
    fn the_listing_is_read_from_the_real_output_and_marks_directories() {
        let entradas = parse_ls(LS_REAL);
        assert_eq!(entradas.len(), 4);
        assert_eq!(entradas[0].name, "boot.py");
        assert_eq!(entradas[0].size, 139);
        assert!(!entradas[0].directory);
        assert_eq!(entradas[2].size, 13588);
        let com_pasta = parse_ls("ls :\n           0 lib/\n          12 a b.txt\nlixo\n");
        assert_eq!(com_pasta.len(), 2);
        assert!(com_pasta[0].directory);
        assert_eq!(com_pasta[0].name, "lib");
        assert_eq!(com_pasta[1].name, "a b.txt");
        assert!(parse_ls("").is_empty());
    }

    #[test]
    fn errors_come_from_the_mpremote_line_and_raw_repl_asks_for_a_retry() {
        assert_eq!(
            error_line("cp :nao.py ./nao.py\nmpremote: cp: nao.py: No such file or directory.\n"),
            Some("cp: nao.py: No such file or directory.".to_owned())
        );
        assert_eq!(error_line("ls :\n 1 a\n"), None);
        assert!(raw_repl_failed(
            "Traceback\nmpremote.transport.TransportError: could not enter raw repl\n"
        ));
        assert!(!raw_repl_failed(
            "mpremote: ls: x: No such file or directory."
        ));
    }

    #[test]
    fn validation_refuses_what_would_fail_on_the_board() {
        let tmp = std::env::temp_dir().join(format!("kinein-serial-files-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        let existente = tmp.join("main.py");
        std::fs::write(&existente, "print(1)\n").unwrap();
        assert!(validate(SerialFilesAction::List, None, None).is_ok());
        assert!(validate(SerialFilesAction::List, Some(":lib"), None).is_err());
        assert!(validate(SerialFilesAction::List, Some("a\nb"), None).is_err());
        assert!(validate(SerialFilesAction::Rm, Some(""), None).is_err());
        assert!(validate(SerialFilesAction::Mkdir, Some("lib"), None).is_ok());
        assert!(validate(SerialFilesAction::Get, Some("main.py"), None).is_err());
        assert!(
            validate(
                SerialFilesAction::Get,
                Some("main.py"),
                Some(&tmp.join("x.py"))
            )
            .is_ok()
        );
        assert!(
            validate(
                SerialFilesAction::Get,
                Some("main.py"),
                Some(&tmp.join("nao/x.py"))
            )
            .is_err()
        );
        assert!(validate(SerialFilesAction::Put, Some("main.py"), Some(&existente)).is_ok());
        assert!(
            validate(
                SerialFilesAction::Put,
                Some("main.py"),
                Some(&PathBuf::from("/nao/existe.py"))
            )
            .is_err()
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
