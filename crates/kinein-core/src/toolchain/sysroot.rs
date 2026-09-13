//! O que uma pasta de sysroot REALMENTE contem (`integracoes/39` §3, item (b)
//! do 42 §8, 2026-09-13).
//!
//! Um `--sysroot` aponta para uma pasta; se ela nao tem `usr/include` e
//! `usr/lib`, o compilador nao acha nada e o erro aparece longe da causa. A
//! IDE le a pasta e DIZ: headers, bibliotecas, `lib/` (o loader e a libc de
//! uma raiz de verdade), os `usr/lib/<triple>` do multiarch, quantos `.pc` o
//! pkg-config veria, e qual libc (glibc pela versao em `usr/include/
//! features.h`; musl pela `libc.so` sem glibc). Puro: so' le o disco.

use std::path::Path;

use kinein_protocol::{SysrootFolders, SysrootReport};

/// Le `path` e monta o relatorio.
#[must_use]
pub fn inspect(path: &Path) -> SysrootReport {
    let exists = path.is_dir();
    let usr_include = path.join("usr/include").is_dir();
    let usr_lib = path.join("usr/lib").is_dir();
    let lib = path.join("lib").is_dir();
    let triple_lib_dirs = if exists {
        triple_dirs(path)
    } else {
        Vec::new()
    };
    let pkgconfig_files = if exists { conta_pc(path) } else { 0 };
    let libc = if exists { libc(path) } else { None };
    let verdict = if !exists {
        "a pasta nao existe".to_owned()
    } else if usr_include && (usr_lib || lib) {
        match (&libc, pkgconfig_files) {
            (Some(c), 0) => format!(
                "utilizavel: headers e bibliotecas ({c}); sem .pc — o pkg-config nao vai achar bibliotecas de terceiros"
            ),
            (Some(c), n) => {
                format!("utilizavel: headers, bibliotecas ({c}) e {n} .pc para o pkg-config")
            }
            (None, _) => "utilizavel: headers e bibliotecas; libc nao identificada".to_owned(),
        }
    } else if usr_include {
        "so' headers (usr/include): compila, nao linka — falta usr/lib ou lib".to_owned()
    } else if usr_lib || lib {
        "so' bibliotecas: linka, nao compila — falta usr/include".to_owned()
    } else {
        "vazia para o compilador: sem usr/include, usr/lib e lib (o sysroot de distro do \
         Fedora e' assim; copie o da placa ou use o tarball da Bootlin/Arm)"
            .to_owned()
    };
    SysrootReport {
        path: path.display().to_string(),
        exists,
        folders: SysrootFolders {
            usr_include,
            usr_lib,
            lib,
        },
        triple_lib_dirs,
        pkgconfig_files,
        libc,
        verdict,
    }
}

/// `usr/lib/<triple>` e `lib/<triple>` do multiarch (Debian/Raspberry Pi OS),
/// relativos ao sysroot, ordenados.
fn triple_dirs(root: &Path) -> Vec<String> {
    let mut saida = Vec::new();
    for base in ["usr/lib", "lib"] {
        let Ok(entradas) = std::fs::read_dir(root.join(base)) else {
            continue;
        };
        for e in entradas.flatten() {
            let nome = e.file_name();
            let nome = nome.to_string_lossy();
            if e.path().is_dir() && nome.contains("-linux-") {
                saida.push(format!("{base}/{nome}"));
            }
        }
    }
    saida.sort();
    saida.dedup();
    saida
}

/// Quantos `.pc` existem onde o pkg-config procura por padrao.
fn conta_pc(root: &Path) -> u32 {
    let mut pastas = vec![
        root.join("usr/lib/pkgconfig"),
        root.join("usr/lib64/pkgconfig"),
        root.join("usr/share/pkgconfig"),
    ];
    for triple in triple_dirs(root) {
        pastas.push(root.join(triple).join("pkgconfig"));
    }
    pastas
        .iter()
        .filter_map(|p| std::fs::read_dir(p).ok())
        .flat_map(std::iter::Iterator::flatten)
        .filter(|e| e.path().extension().is_some_and(|x| x == "pc"))
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

/// `glibc <maior>.<menor>` por `usr/include/features.h`; `musl` quando ha' a
/// `libc.so` do musl sem os macros da glibc.
fn libc(root: &Path) -> Option<String> {
    let features = std::fs::read_to_string(root.join("usr/include/features.h")).ok();
    if let Some(texto) = &features {
        let mut maior = None;
        let mut menor = None;
        for linha in texto.lines() {
            let l = linha.trim();
            if let Some(v) = l.strip_prefix("#define __GLIBC__") {
                maior = v.trim().parse::<u32>().ok();
            } else if let Some(v) = l.strip_prefix("#define __GLIBC_MINOR__") {
                menor = v.trim().parse::<u32>().ok();
            }
        }
        if let (Some(a), Some(b)) = (maior, menor) {
            return Some(format!("glibc {a}.{b}"));
        }
    }
    let musl = ["usr/lib/libc.so", "lib/libc.so", "lib/ld-musl-aarch64.so.1"]
        .iter()
        .any(|p| root.join(p).exists());
    if musl {
        return Some("musl".to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::inspect;

    fn raiz(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-sysroot-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// O sysroot de distro do Fedora (medido em 2026-09-12): a pasta existe e
    /// esta' vazia — o veredito diz o que fazer, nao "ok".
    #[test]
    fn an_empty_sysroot_says_so_and_what_to_do() {
        let dir = raiz("vazio");
        let r = inspect(&dir);
        assert!(r.exists && !r.folders.usr_include && !r.folders.usr_lib && !r.folders.lib);
        assert!(
            r.verdict.starts_with("vazia para o compilador"),
            "{}",
            r.verdict
        );
        assert!(r.verdict.contains("Bootlin"), "{}", r.verdict);
        let r = inspect(&dir.join("nao-existe"));
        assert!(!r.exists);
        assert_eq!(r.verdict, "a pasta nao existe");
    }

    /// Uma raiz copiada da placa (Raspberry Pi OS): multiarch, glibc pela
    /// features.h, os .pc contados onde o pkg-config procura.
    #[test]
    fn a_real_root_is_read_with_multiarch_glibc_and_pkgconfig() {
        let dir = raiz("pi");
        let mk = |rel: &str| std::fs::create_dir_all(dir.join(rel)).unwrap();
        mk("usr/include");
        mk("usr/lib/aarch64-linux-gnu/pkgconfig");
        mk("lib/aarch64-linux-gnu");
        mk("usr/share/pkgconfig");
        mk("usr/lib/python3.11");
        std::fs::write(
            dir.join("usr/include/features.h"),
            "#define __GLIBC__\t2\n#define __GLIBC_MINOR__\t36\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("usr/lib/aarch64-linux-gnu/pkgconfig/libgpiod.pc"),
            "",
        )
        .unwrap();
        std::fs::write(dir.join("usr/share/pkgconfig/xkeyboard-config.pc"), "").unwrap();
        std::fs::write(dir.join("usr/share/pkgconfig/README"), "").unwrap();
        let r = inspect(&dir);
        assert!(r.folders.usr_include && r.folders.usr_lib && r.folders.lib);
        assert_eq!(
            r.triple_lib_dirs,
            vec!["lib/aarch64-linux-gnu", "usr/lib/aarch64-linux-gnu"],
            "python3.11 nao e' um triple"
        );
        assert_eq!(r.pkgconfig_files, 2, "so' .pc conta");
        assert_eq!(r.libc.as_deref(), Some("glibc 2.36"));
        assert!(
            r.verdict.starts_with("utilizavel") && r.verdict.contains("2 .pc"),
            "{}",
            r.verdict
        );
    }

    /// O tarball generico (Arm/Bootlin): headers e libs, sem .pc — o veredito
    /// avisa; musl pela libc.so; so' headers ou so' libs tem cada um a sua frase.
    #[test]
    fn generic_musl_and_half_sysroots_get_their_own_verdicts() {
        let dir = raiz("generico");
        std::fs::create_dir_all(dir.join("usr/include")).unwrap();
        std::fs::create_dir_all(dir.join("usr/lib")).unwrap();
        std::fs::write(dir.join("usr/lib/libc.so"), "").unwrap();
        let r = inspect(&dir);
        assert_eq!(r.libc.as_deref(), Some("musl"));
        assert!(r.verdict.contains("sem .pc"), "{}", r.verdict);

        let so_headers = raiz("so-headers");
        std::fs::create_dir_all(so_headers.join("usr/include")).unwrap();
        assert!(inspect(&so_headers).verdict.starts_with("so' headers"));
        let so_libs = raiz("so-libs");
        std::fs::create_dir_all(so_libs.join("lib")).unwrap();
        assert!(inspect(&so_libs).verdict.starts_with("so' bibliotecas"));
        // usr/include + lib (sem usr/lib): uma raiz minima ainda linka.
        std::fs::create_dir_all(so_libs.join("usr/include")).unwrap();
        assert!(
            inspect(&so_libs).verdict.starts_with("utilizavel"),
            "{}",
            inspect(&so_libs).verdict
        );
    }
}
