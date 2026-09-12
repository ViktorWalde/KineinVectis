//! Onde as toolchains moram ALEM do `PATH`.
//!
//! Os diretorios que os distribuidores de toolchain usam por padrao, para a
//! IDE achar o que o usuario instalou sem pedir que ele edite variavel nenhuma
//! (`integracoes/39`, 2026-09-12).
//!
//! ```text
//! ~/.local/share/kinein-vectis/toolchains/<id>/<versao>/bin   a pasta da IDE
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

/// Os diretorios `bin` de toolchain que existem sob `home` (e sob os dois
/// caminhos que as variaveis de ambiente podem sobrescrever), na ordem de
/// preferencia: a pasta da IDE primeiro, `/opt` por ultimo.
#[must_use]
pub fn extra_search_dirs(
    home: &Path,
    xpacks_store: Option<&Path>,
    idf_tools: Option<&Path>,
) -> Vec<PathBuf> {
    let mut saida = Vec::new();
    // 1. o que a IDE instalou: toolchains/<id>/<versao>/bin
    for versao in subpastas_ate(&home.join(".local/share/kinein-vectis/toolchains"), 2) {
        empurrar(&mut saida, versao.join("bin"));
    }
    // 2. xpm: @xpack-dev-tools/<nome>/<versao>/.content/bin
    let store = xpacks_store.map_or_else(|| home.join(".local/xPacks"), Path::to_path_buf);
    for versao in subpastas_ate(&store.join("@xpack-dev-tools"), 2) {
        empurrar(&mut saida, versao.join(".content/bin"));
        empurrar(&mut saida, versao.join("bin"));
    }
    // 3. ESP-IDF: tools/<nome>/<versao>/<nome>/bin
    let esp = idf_tools.map_or_else(|| home.join(".espressif/tools"), Path::to_path_buf);
    for pasta in subpastas_ate(&esp, 3) {
        empurrar(&mut saida, pasta.join("bin"));
    }
    // 4. o que rustup/cargo/pipx instalam para o usuario
    empurrar(&mut saida, home.join(".cargo/bin"));
    empurrar(&mut saida, home.join(".local/bin"));
    // 5. tarballs desempacotados em /opt
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
