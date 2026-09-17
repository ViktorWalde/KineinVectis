//! Os builds por motor de framework (bloco E do `roadmaps/41`, 2026-09-17):
//! `idf.py build`, `west build`, `pio run`. O pico-sdk nao mora aqui — e' o
//! `CMake` nativo com `-DPICO_SDK_PATH`, ver `engine::cmake_extra_args`.
//!
//! Linhas, lidas na fonte em 2026-09-17: ESP-IDF "Start a Project"
//! (`idf.py build`; a pasta e' `build/`); Zephyr "west build" (`west build
//! -b <board> -d <dir>`, e a placa fica em `CACHED_BOARD` no `CMakeCache` ou
//! em `west config build.board`); `PlatformIO` "pio run" (`pio run` compila
//! todos os `env`, ou os `default_envs` do platformio.ini). Diagnosticos:
//! os tres chamam gcc/clang por baixo — o parser `GccLike` casa.

use std::{
    path::Path,
    process::Command,
    sync::{Arc, atomic::AtomicBool},
};

use super::engine::{Engine, idf_command_args};
use super::{BuildError, BuildEvent, BuildOutcome, DiagnosticFormat, stream_command};

/// A pasta de build dos wrappers (`idf.py` e `west` usam `build/`).
pub(super) const BUILD_DIR: &str = "build";

pub(super) fn run_engine_build(
    root: &Path,
    engine: &Engine,
    cancel: &Arc<AtomicBool>,
    sink: &mut dyn FnMut(BuildEvent),
) -> Result<BuildOutcome, BuildError> {
    match engine {
        Engine::EspIdf { activation } => {
            let mut command = Command::new("bash");
            command.args(idf_command_args(activation, &["build"]));
            command.current_dir(root);
            sink(BuildEvent::Output {
                stream: "stderr",
                line: format!("ativando o ESP-IDF: . {}", activation.display()),
            });
            stream_command(
                command,
                "idf.py build",
                DiagnosticFormat::GccLike,
                cancel,
                sink,
            )
        }
        Engine::Zephyr { west } => {
            let mut command = Command::new(west);
            command.arg("build").arg("-d").arg(BUILD_DIR);
            let placa = zephyr_board(root, west);
            match &placa {
                Some(board) => {
                    command.arg("-b").arg(board);
                }
                None => sink(BuildEvent::Output {
                    stream: "stderr",
                    line: "sem placa fixada: `west config build.board <placa>` (ou -b na \
                           primeira vez) — o west vai reclamar se precisar dela"
                        .to_owned(),
                }),
            }
            command.current_dir(root);
            let rotulo = placa.map_or_else(
                || "west build -d build".to_owned(),
                |b| format!("west build -d build -b {b}"),
            );
            stream_command(command, &rotulo, DiagnosticFormat::GccLike, cancel, sink)
        }
        Engine::PlatformIo { pio } => {
            let mut command = Command::new(pio);
            command.arg("run");
            command.current_dir(root);
            stream_command(command, "pio run", DiagnosticFormat::GccLike, cancel, sink)
        }
        Engine::PicoSdk { .. } => Err(BuildError::Unsupported {
            kind: "pico-sdk fora do CMake".to_owned(),
        }),
    }
}

/// A placa do Zephyr: o `CACHED_BOARD` de um build anterior, senao o que o
/// `west config build.board` diz (a forma da doc do west). `None` = nenhum
/// dos dois — o west decide se pode viver sem.
#[must_use]
pub fn zephyr_board(root: &Path, west: &Path) -> Option<String> {
    let cache = root.join(BUILD_DIR).join("CMakeCache.txt");
    if let Ok(texto) = std::fs::read_to_string(cache)
        && let Some(valor) = texto.lines().find_map(|l| {
            l.strip_prefix("CACHED_BOARD:STRING=")
                .or_else(|| l.strip_prefix("BOARD:STRING="))
        })
        && !valor.trim().is_empty()
    {
        return Some(valor.trim().to_owned());
    }
    let saida = Command::new(west)
        .args(["config", "build.board"])
        .current_dir(root)
        .output()
        .ok()?;
    let valor = String::from_utf8_lossy(&saida.stdout).trim().to_owned();
    (saida.status.success() && !valor.is_empty()).then_some(valor)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::zephyr_board;

    #[test]
    fn the_zephyr_board_comes_from_the_cache_then_from_west_config() {
        let raiz = std::env::temp_dir().join(format!("kinein-zephyr-board-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(raiz.join("build")).unwrap();
        let west = raiz.join("west");
        std::fs::write(&west, "#!/bin/sh\nif [ \"$1 $2\" = \"config build.board\" ]; then echo nrf52840dk/nrf52840; fi\n").unwrap();
        std::fs::set_permissions(&west, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            zephyr_board(&raiz, &west).as_deref(),
            Some("nrf52840dk/nrf52840")
        );
        std::fs::write(
            raiz.join("build/CMakeCache.txt"),
            "X:STRING=1\nCACHED_BOARD:STRING=esp32_devkitc/esp32/procpu\n",
        )
        .unwrap();
        assert_eq!(
            zephyr_board(&raiz, &west).as_deref(),
            Some("esp32_devkitc/esp32/procpu")
        );
        std::fs::write(&west, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::remove_file(raiz.join("build/CMakeCache.txt")).unwrap();
        assert_eq!(zephyr_board(&raiz, &west), None);
        let _ = std::fs::remove_dir_all(&raiz);
    }
}
