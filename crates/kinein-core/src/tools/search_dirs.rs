//! Onde as toolchains moram ALEM do `PATH`.
//!
//! Os diretorios que os distribuidores de toolchain usam por padrao, para a
//! IDE achar o que o usuario instalou sem pedir que ele edite variavel nenhuma
//! (`integracoes/39`, 2026-09-12).
//!
//! ```text
//! ~/.local/share/kinein-vectis/toolchains/<id>/<versao>/bin   a pasta da IDE
//!                                            (installed_bin_dirs: lida a cada busca)
//! ~/.local/xPacks/@xpack-dev-tools/<nome>/<versao>/.content/bin   xpm (fonte:
//!                                            xpack.github.io/xpm/docs/user/folders,
//!                                            XPACKS_STORE_FOLDER sobrescreve)
//! ~/.espressif/tools/<nome>/<versao>/<nome>/bin    idf_tools.py do ESP-IDF
//!                                            (IDF_TOOLS_PATH sobrescreve)
//! ~/.cargo/bin, ~/.local/bin                 rustup/cargo install, pipx
//! /opt/<algo>/bin                            tarballs da Arm, Bootlin, SDKs
//! ```
//!
//! Puro: recebe a raiz do usuario e as duas variaveis ja' lidas por quem
//! chama, devolve os diretorios que EXISTEM. Nao le ambiente, nao executa nada.

use std::path::{Path, PathBuf};

/// A pasta onde a IDE instala toolchains (`integracoes/39` §5): nunca fora
/// dela, nunca no sistema.
#[must_use]
pub fn install_root(home: &Path) -> PathBuf {
    home.join(".local/share/kinein-vectis/toolchains")
}

/// Os `bin` das toolchains que a IDE instalou em `root` (`<id>/<versao>/bin`).
///
/// Os que EXISTEM agora: enumerado a cada busca, e nao na construcao do
/// detector — uma toolchain instalada nesta sessao aparece na proxima
/// deteccao sem reiniciar nada.
#[must_use]
pub fn installed_bin_dirs(root: &Path) -> Vec<PathBuf> {
    let mut saida = Vec::new();
    for versao in subpastas_ate(root, 2) {
        empurrar(&mut saida, versao.join("bin"));
    }
    saida
}

/// Os diretorios `bin` de toolchain que existem sob `home` ALEM da pasta da IDE.
///
/// E sob os dois caminhos que as variaveis de ambiente podem sobrescrever; na
/// ordem de preferencia: xpm primeiro, `/opt` por ultimo.
#[must_use]
pub fn extra_search_dirs(
    home: &Path,
    xpacks_store: Option<&Path>,
    idf_tools: Option<&Path>,
) -> Vec<PathBuf> {
    let mut saida = Vec::new();
    // 1. xpm: @xpack-dev-tools/<nome>/<versao>/.content/bin
    let store = xpacks_store.map_or_else(|| home.join(".local/xPacks"), Path::to_path_buf);
    for versao in subpastas_ate(&store.join("@xpack-dev-tools"), 2) {
        empurrar(&mut saida, versao.join(".content/bin"));
        empurrar(&mut saida, versao.join("bin"));
    }
    // 2. ESP-IDF: tools/<nome>/<versao>/<nome>/bin
    let esp = idf_tools.map_or_else(|| home.join(".espressif/tools"), Path::to_path_buf);
    for pasta in subpastas_ate(&esp, 3) {
        empurrar(&mut saida, pasta.join("bin"));
    }
    // 3. o que rustup/cargo/pipx instalam para o usuario
    empurrar(&mut saida, home.join(".cargo/bin"));
    empurrar(&mut saida, home.join(".local/bin"));
    // 4. tarballs desempacotados em /opt
    for pasta in subpastas_ate(Path::new("/opt"), 1) {
        empurrar(&mut saida, pasta.join("bin"));
    }
    saida
}

fn empurrar(saida: &mut Vec<PathBuf>, dir: PathBuf) {
    if dir.is_dir() && !saida.contains(&dir) {
        saida.push(dir);
    }
}

/// As subpastas de `raiz` a exatamente `nivel` niveis de profundidade, em
/// ordem de nome (deterministico), ignorando o que nao e' pasta.
fn subpastas_ate(raiz: &Path, nivel: usize) -> Vec<PathBuf> {
    let mut atual = vec![raiz.to_path_buf()];
    for _ in 0..nivel {
        let mut proximo = Vec::new();
        for dir in &atual {
            let Ok(entradas) = std::fs::read_dir(dir) else {
                continue;
            };
            let mut filhas: Vec<PathBuf> = entradas
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect();
            filhas.sort();
            proximo.extend(filhas);
        }
        atual = proximo;
    }
    atual
}
