//! O que existe NESTA maquina: mede, nao adivinha.
//!
//! Espelha o [`crate::configaction::availability`]. A pergunta que este modulo
//! responde e uma so: *"o `find_package` acharia esta biblioteca aqui?"*.
//!
//! **POR QUE NAO EXECUTAMOS O CMAKE PARA DESCOBRIR.** Rodar um `configure` de
//! sonda por biblioteca custaria segundos cada e sujaria o build dir do
//! usuario. O que se faz e olhar onde o `CMake` olharia: os diretorios de config
//! package do sistema. E' uma aproximacao, e ela e' HONESTA nos dois sentidos —
//! quando diz "achei", o `find_package` acha; quando diz "nao achei", ainda
//! pode haver um caminho custom no `CMAKE_PREFIX_PATH` do usuario, e por isso
//! o resultado se chama `Detected`/`NotDetected`, nunca "ausente".

use std::path::{Path, PathBuf};

/// Onde um config package costuma morar num sistema Unix.
///
/// Nao e' a lista completa do `CMake` (que inclui variaveis de ambiente e
/// registro); e' o suficiente para responder "esta instalado pelo gerenciador
/// de pacotes da distro?", que e' a pergunta do usuario.
const PREFIXOS: &[&str] = &[
    "/usr/lib/cmake",
    "/usr/lib64/cmake",
    "/usr/local/lib/cmake",
    "/usr/local/lib64/cmake",
    "/usr/share/cmake",
    "/usr/lib/x86_64-linux-gnu/cmake",
];

/// Resultado da medicao para uma biblioteca.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Availability {
    /// Ha config package no sistema: `find_package` deve achar.
    Detected,
    /// Nao foi achado nos prefixos conhecidos. NAO significa "nao existe".
    NotDetected,
}

/// Mede um pacote pelo nome que o `find_package` usaria.
#[must_use]
pub(super) fn detect(package_name: &str) -> Availability {
    detect_in(package_name, PREFIXOS.iter().map(Path::new))
}

/// Idem, com os prefixos injetados — e o que torna isto testavel sem depender
/// do que esta instalado na maquina que roda o teste.
pub(super) fn detect_in<'a>(
    package_name: &str,
    prefixos: impl Iterator<Item = &'a Path>,
) -> Availability {
    let alvo = package_name.to_ascii_lowercase();
    for prefixo in prefixos {
        let Ok(entradas) = std::fs::read_dir(prefixo) else {
            continue;
        };
        for entrada in entradas.flatten() {
            if !entrada.path().is_dir() {
                continue;
            }
            let nome = entrada.file_name();
            let nome = nome.to_string_lossy().to_ascii_lowercase();
            // O diretorio costuma ser `<Pacote>` ou `<Pacote>-<versao>`.
            if nome == alvo || nome.starts_with(&format!("{alvo}-")) {
                return Availability::Detected;
            }
        }
    }
    Availability::NotDetected
}

/// Diretorios de config package usados na deteccao, para a UI poder explicar
/// ONDE se olhou quando o resultado for `NotDetected`.
#[must_use]
pub(super) fn search_paths() -> Vec<PathBuf> {
    PREFIXOS.iter().map(PathBuf::from).collect()
}
