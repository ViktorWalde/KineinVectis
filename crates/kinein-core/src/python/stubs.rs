//! Os STUBS do `MicroPython` por placa (C4 do `roadmaps/41` bloco C): o
//! `import machine` completa e o basedpyright para de dizer que o modulo
//! nao existe.
//!
//! Fonte (micropython-stubs.readthedocs.io, "Install the micropython-stubs",
//! lida em 2026-09-17): `pip install -U micropython-<port>[-<board>]-stubs
//! --no-user --target ./typings` — os stubs ficam numa pasta `typings` na
//! raiz do projeto, "sem precisar nem do `CPython`"; a pagina de exemplo do
//! `pyproject.toml` configura o pyright com `stubPath = "typings"` e
//! `reportMissingModuleSource = "none"` (um stub sem modulo de verdade e' o
//! normal: o `machine` so' existe na placa). Aqui o instalador e' o `uv`
//! (`uv pip install --target`, medido nesta maquina em 2026-09-17: 2
//! pacotes em 7 ms) ou, sem uv, o `pip` do interpretador do projeto.
//!
//! O nome do pacote por placa vem da lista publicada ("List of available
//! packages", lida em 2026-09-17): `micropython-esp32-stubs` (o port),
//! `micropython-esp32-esp32_generic_c3-stubs` (a placa), `micropython-rp2-
//! rpi_pico_w-stubs`… — o chip do kit/identidade escolhe a placa.

use std::path::{Path, PathBuf};

/// A pasta dos stubs, relativa a raiz (a da fonte).
pub const PASTA: &str = "typings";

/// O pacote de stubs para um `port` e (opcional) uma `board`.
#[must_use]
pub fn package(port: &str, board: Option<&str>) -> String {
    board.map(str::trim).filter(|b| !b.is_empty()).map_or_else(
        || format!("micropython-{port}-stubs"),
        |board| format!("micropython-{port}-{board}-stubs"),
    )
}

/// O que o modelo do projeto sugere: `(port, board)`.
///
/// A partir do chip do kit/identidade (`esp32c3` → `esp32`/
/// `esp32_generic_c3`) ou da familia (`rp2040` → `rp2`, sem placa: Pico e
/// Pico W tem stubs diferentes e o chip nao distingue). Sem evidencia, `None`.
#[must_use]
pub fn suggest(chip: Option<&str>, family: Option<&str>) -> Option<(String, Option<String>)> {
    if let Some(chip) = chip.map(|c| c.trim().to_ascii_lowercase()) {
        if let Some(sufixo) = chip.strip_prefix("esp32") {
            let board = if sufixo.is_empty() {
                "esp32_generic".to_owned()
            } else {
                format!("esp32_generic_{sufixo}")
            };
            return Some(("esp32".to_owned(), Some(board)));
        }
        if chip.starts_with("rp2") {
            return Some(("rp2".to_owned(), None));
        }
        if chip.starts_with("stm32") {
            return Some(("stm32".to_owned(), None));
        }
        if chip.starts_with("nrf") {
            return Some(("nrf".to_owned(), None));
        }
    }
    match family {
        Some("espressif") => Some(("esp32".to_owned(), None)),
        Some("rp2040") => Some(("rp2".to_owned(), None)),
        Some("stm32") => Some(("stm32".to_owned(), None)),
        Some("nrf") => Some(("nrf".to_owned(), None)),
        _ => None,
    }
}

/// Com que se instala: o `uv` (o mesmo que cria o `.venv`) ou o `pip` do
/// interpretador do projeto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installer {
    /// `uv pip install -U --target <typings> <pacote>`.
    Uv(PathBuf),
    /// `<python> -m pip install -U <pacote> --no-user --target <typings>`.
    Pip(PathBuf),
}

impl Installer {
    /// O programa e os argumentos.
    #[must_use]
    pub fn command_line(&self, package: &str, target: &Path) -> (PathBuf, Vec<String>) {
        let alvo = target.display().to_string();
        match self {
            Self::Uv(uv) => (
                uv.clone(),
                vec![
                    "pip".to_owned(),
                    "install".to_owned(),
                    "-U".to_owned(),
                    // A cache do uv e a pasta do projeto podem estar em
                    // discos diferentes: copiar evita o aviso de hardlink.
                    "--link-mode=copy".to_owned(),
                    "--target".to_owned(),
                    alvo,
                    package.to_owned(),
                ],
            ),
            Self::Pip(python) => (
                python.clone(),
                vec![
                    "-m".to_owned(),
                    "pip".to_owned(),
                    "install".to_owned(),
                    "-U".to_owned(),
                    package.to_owned(),
                    "--no-user".to_owned(),
                    "--target".to_owned(),
                    alvo,
                ],
            ),
        }
    }
}

/// `<root>/typings` quando existe e tem ao menos um `.pyi` — e' o que vai ao
/// basedpyright como `stubPath`.
#[must_use]
pub fn installed(root: &Path) -> Option<PathBuf> {
    let pasta = root.join(PASTA);
    let tem_stub = std::fs::read_dir(&pasta)
        .ok()?
        .flatten()
        .any(|e| e.path().extension().is_some_and(|x| x == "pyi"));
    tem_stub.then_some(pasta)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{Installer, installed, package, suggest};

    #[test]
    fn the_package_name_follows_the_source_and_the_chip_picks_the_board() {
        assert_eq!(package("esp32", None), "micropython-esp32-stubs");
        assert_eq!(
            package("esp32", Some("esp32_generic_c3")),
            "micropython-esp32-esp32_generic_c3-stubs"
        );
        assert_eq!(package("rp2", Some("")), "micropython-rp2-stubs");
        assert_eq!(
            suggest(Some("esp32c3"), None),
            Some(("esp32".to_owned(), Some("esp32_generic_c3".to_owned())))
        );
        assert_eq!(
            suggest(Some("ESP32"), Some("espressif")),
            Some(("esp32".to_owned(), Some("esp32_generic".to_owned())))
        );
        assert_eq!(
            suggest(Some("rp2040"), None),
            Some(("rp2".to_owned(), None))
        );
        assert_eq!(
            suggest(None, Some("rp2040")),
            Some(("rp2".to_owned(), None))
        );
        assert_eq!(
            suggest(Some("STM32F401CC"), None),
            Some(("stm32".to_owned(), None))
        );
        assert_eq!(suggest(None, Some("cortex-m")), None);
        assert_eq!(suggest(None, None), None);
    }

    #[test]
    fn the_installer_lines_are_the_documented_ones() {
        let (p, a) = Installer::Uv(PathBuf::from("/x/uv"))
            .command_line("micropython-esp32-stubs", Path::new("/w/typings"));
        assert_eq!(p, PathBuf::from("/x/uv"));
        assert_eq!(
            a,
            [
                "pip",
                "install",
                "-U",
                "--link-mode=copy",
                "--target",
                "/w/typings",
                "micropython-esp32-stubs"
            ]
        );
        let (p, a) = Installer::Pip(PathBuf::from("/w/.venv/bin/python"))
            .command_line("micropython-rp2-stubs", Path::new("/w/typings"));
        assert_eq!(p, PathBuf::from("/w/.venv/bin/python"));
        assert_eq!(
            a,
            [
                "-m",
                "pip",
                "install",
                "-U",
                "micropython-rp2-stubs",
                "--no-user",
                "--target",
                "/w/typings"
            ]
        );
    }

    #[test]
    fn installed_means_a_typings_folder_with_a_pyi() {
        let raiz = std::env::temp_dir().join(format!("kinein-stubs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&raiz);
        std::fs::create_dir_all(raiz.join("typings")).unwrap();
        assert_eq!(installed(&raiz), None, "pasta vazia nao conta");
        std::fs::write(raiz.join("typings/machine.pyi"), "").unwrap();
        assert_eq!(installed(&raiz), Some(raiz.join("typings")));
        let _ = std::fs::remove_dir_all(&raiz);
    }
}
