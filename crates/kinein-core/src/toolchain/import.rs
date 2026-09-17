//! IMPORTAR um kit de um SDK que ja' esta' no disco.
//!
//! Item (d) do 42 §8 e `integracoes/39` §3 (2026-09-13): o que o SDK declara
//! vira uma PROPOSTA de kit — compiladores, gdb, sysroot, triple, arquivo de
//! toolchain — e nada e' gravado ate' o `toolchain.setKit` aplicar.
//!
//! Tres formas, por evidencia no caminho dado:
//!
//! ```text
//! yocto        um `environment-setup-<arch>-*` (o arquivo, ou a pasta do SDK que
//!              o contem). O script e' executado por `sh` — e' o que o SDK manda
//!              fazer (`. environment-setup-...`) — e as variaveis que ele exporta
//!              sao lidas: CC/CXX/GDB (o CC ja' traz `--sysroot=$SDKTARGETSYSROOT`),
//!              SDKTARGETSYSROOT, OECORE_NATIVE_SYSROOT (o toolchain file do SDK
//!              mora em usr/share/cmake/OEToolchainConfig.cmake), TARGET_PREFIX
//!              (docs.yoctoproject.org, sdk-manual "Using the SDK Toolchain
//!              Directly" e "Makefile-Based Projects", lidos em 2026-09-13)
//! buildroot    `output/` (ou `output/host`): host/bin/<triple>-gcc,
//!              host/<triple>/sysroot, host/share/buildroot/toolchainfile.cmake
//!              (manual do Buildroot, "Using the generated toolchain outside
//!              Buildroot" e "Using Buildroot toolchain with CMake", 2026-09-13)
//! toolchain-dir  uma pasta com bin/<triple>-gcc: o tarball da Arm (sysroot em
//!              <triple>/libc), da Bootlin (<triple>/sysroot), ou o que a IDE
//!              instalou — o sysroot vem de `<gcc> -print-sysroot`
//! zephyr-sdk   a raiz de um Zephyr SDK (sdk-ng): `sdk_version` + `cmake/
//!              Zephyr-sdkConfig.cmake` provam o que e'; as toolchains GNU moram
//!              em `gnu/<toolchain>/bin/<toolchain>-gcc` (v1.x — o setup.sh
//!              extrai em `gnu/`; `sdk_gnu_toolchains` lista os nomes) ou na
//!              raiz (`<toolchain>/`, o layout 0.16/0.17 e os links de
//!              "bisectability" que o setup.sh cria). Lido no
//!              `scripts/template_setup_posix` e no `cmake/Zephyr-sdkConfig.cmake`
//!              do sdk-ng em 2026-09-17. A proposta e' UMA toolchain (a
//!              `arm-zephyr-eabi` quando ha' varias — o Cortex-M e' o caso
//!              comum — dita como escolha, com as outras listadas na evidencia);
//!              nada de board: quem escolhe o board e' o `west build -b`, e o
//!              Zephyr acha o SDK por `ZEPHYR_SDK_INSTALL_DIR` ou pelo registro
//!              CMake (`cmake -P cmake/zephyr_sdk_export.cmake`)
//! ```
//!
//! NAO MEDIDO contra um SDK Yocto ou uma arvore Buildroot reais — nao ha'
//! nenhum nesta maquina (42 §8 item 5). O que esta' provado e' o contrato: um
//! `environment-setup` que exporta as variaveis documentadas e uma arvore
//! `output/host` com a forma documentada.

use std::path::{Path, PathBuf};
use std::process::Command;

use kinein_protocol::KitImport;

/// Le `path` e propoe um kit; `Err` diz o que nao foi reconhecido.
pub fn import(path: &Path) -> Result<KitImport, String> {
    if !path.exists() {
        return Err(format!("{} nao existe", path.display()));
    }
    if let Some(script) = environment_setup(path) {
        return yocto(&script);
    }
    if let Some(kit) = zephyr_sdk(path) {
        return Ok(kit);
    }
    if let Some(host) = buildroot_host(path) {
        return Ok(buildroot(&host));
    }
    if path.join("bin").is_dir() {
        if let Some(kit) = toolchain_dir(path) {
            return Ok(kit);
        }
    }
    Err(format!(
        "{} nao e' um SDK que eu reconheca: procurei um environment-setup-* (Yocto), um \
         host/bin/<triple>-gcc com host/<triple>/sysroot (Buildroot), sdk_version + \
         cmake/Zephyr-sdkConfig.cmake (Zephyr SDK) e um bin/<triple>-gcc (pasta de toolchain)",
        path.display()
    ))
}

/// A toolchain do Zephyr SDK que a proposta usa quando ha' varias: o
/// Cortex-M e' o caso comum, e a escolha e' DITA na evidencia.
const ZEPHYR_TOOLCHAIN_PREFERIDA: &str = "arm-zephyr-eabi";

/// Um Zephyr SDK (sdk-ng): a raiz com `sdk_version` e o pacote `CMake`.
fn zephyr_sdk(path: &Path) -> Option<KitImport> {
    let versao = std::fs::read_to_string(path.join("sdk_version")).ok()?;
    let versao = versao.trim().to_owned();
    let config = path.join("cmake/Zephyr-sdkConfig.cmake");
    if !config.is_file() {
        return None;
    }
    // As toolchains GNU: `gnu/<nome>/bin/<nome>-gcc` (v1.x) e `<nome>/bin/
    // <nome>-gcc` (0.16/0.17 e os links de bisectability). Caminho canonico
    // uma vez so' — o link e a pasta sao a mesma toolchain.
    let mut toolchains: Vec<(String, PathBuf)> = Vec::new();
    for base in [path.join("gnu"), path.to_path_buf()] {
        let Ok(entradas) = std::fs::read_dir(&base) else {
            continue;
        };
        for entrada in entradas.flatten() {
            let nome = entrada.file_name().to_string_lossy().into_owned();
            let gcc = entrada.path().join("bin").join(format!("{nome}-gcc"));
            if !gcc.is_file() {
                continue;
            }
            let real = std::fs::canonicalize(&gcc).unwrap_or(gcc);
            if toolchains.iter().any(|(_, g)| *g == real) {
                continue;
            }
            toolchains.push((nome, real));
        }
    }
    toolchains.sort();
    let mut evidence = vec![
        format!(
            "{}: Zephyr SDK {versao}",
            path.join("sdk_version").display()
        ),
        format!("{}: pacote CMake Zephyr-sdk", config.display()),
    ];
    if toolchains.is_empty() {
        return Some(KitImport {
            kind: "zephyr-sdk".to_owned(),
            path: path.display().to_string(),
            evidence,
            c_compiler: None,
            cxx_compiler: None,
            gdb: None,
            sysroot: None,
            target_triple: None,
            toolchain_file: None,
            hint: Some(
                "Zephyr SDK sem toolchain GNU instalada: rode o setup.sh do SDK (`./setup.sh \
                 -t arm-zephyr-eabi`, ou `-t all`) e importe de novo"
                    .to_owned(),
            ),
        });
    }
    let escolhida = toolchains
        .iter()
        .position(|(n, _)| n == ZEPHYR_TOOLCHAIN_PREFERIDA)
        .unwrap_or(0);
    let (nome, gcc) = toolchains[escolhida].clone();
    let nomes: Vec<&str> = toolchains.iter().map(|(n, _)| n.as_str()).collect();
    evidence.push(format!(
        "toolchains GNU instaladas: {} — proposta: {nome}{}",
        nomes.join(", "),
        if toolchains.len() > 1 {
            "; para outra, importe a pasta dela (gnu/<toolchain>)"
        } else {
            ""
        }
    ));
    let bin = gcc.parent().map(Path::to_path_buf).unwrap_or_default();
    let com = |sufixo: &str| {
        let p = bin.join(format!("{nome}-{sufixo}"));
        p.is_file().then(|| p.display().to_string())
    };
    // O sysroot de uma toolchain bare metal e' o que o gcc declara (a libc
    // da toolchain: picolibc/newlib), nao um sistema de destino.
    let sysroot = print_sysroot(&gcc).filter(|s| s.is_dir());
    if let Some(s) = &sysroot {
        evidence.push(format!("sysroot (libc da toolchain) em {}", s.display()));
    }
    Some(KitImport {
        kind: "zephyr-sdk".to_owned(),
        path: path.display().to_string(),
        evidence,
        c_compiler: Some(gcc.display().to_string()),
        cxx_compiler: com("g++"),
        gdb: com("gdb"),
        sysroot: sysroot.map(|s| s.display().to_string()),
        target_triple: Some(nome.clone()),
        toolchain_file: None,
        hint: Some(format!(
            "projeto Zephyr compila pelo `west build -b <board>`, que acha o SDK por \
             ZEPHYR_SDK_INSTALL_DIR={} ou pelo registro CMake do setup.sh; o kit serve ao \
             clangd (compilador cross) e ao depurador (gdb do SDK)",
            path.display()
        )),
    })
}

/// O `environment-setup-*` dado, ou o unico dentro da pasta.
fn environment_setup(path: &Path) -> Option<PathBuf> {
    let nome = path.file_name()?.to_string_lossy();
    if path.is_file() && nome.starts_with("environment-setup-") {
        return Some(path.to_path_buf());
    }
    if !path.is_dir() {
        return None;
    }
    let mut achados: Vec<PathBuf> = std::fs::read_dir(path)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("environment-setup-"))
        })
        .collect();
    achados.sort();
    (achados.len() == 1).then(|| achados.remove(0))
}

/// Yocto: executa o script por `sh` e le o que ele exporta.
fn yocto(script: &Path) -> Result<KitImport, String> {
    // `. script` no proprio shell, e depois um `printf` por variavel: e' o que o
    // manual do SDK manda o usuario fazer antes de compilar. O `command -v`
    // resolve o CC (primeira palavra) no PATH que o script montou.
    let programa = r#". "$1" >/dev/null 2>&1 || exit 3
printf 'CC=%s\n' "$CC"
printf 'CXX=%s\n' "$CXX"
printf 'GDB=%s\n' "$GDB"
printf 'SDKTARGETSYSROOT=%s\n' "$SDKTARGETSYSROOT"
printf 'OECORE_NATIVE_SYSROOT=%s\n' "$OECORE_NATIVE_SYSROOT"
printf 'TARGET_PREFIX=%s\n' "$TARGET_PREFIX"
printf 'CC_PATH=%s\n' "$(command -v "${CC%% *}" 2>/dev/null)"
printf 'CXX_PATH=%s\n' "$(command -v "${CXX%% *}" 2>/dev/null)"
printf 'GDB_PATH=%s\n' "$(command -v "${GDB%% *}" 2>/dev/null)"
"#;
    let saida = Command::new("sh")
        .arg("-c")
        .arg(programa)
        .arg("sh")
        .arg(script)
        .output()
        .map_err(|e| format!("nao consegui executar sh: {e}"))?;
    if !saida.status.success() {
        return Err(format!(
            "o script {} nao pode ser carregado pelo sh (saiu com {}): {}",
            script.display(),
            saida.status,
            String::from_utf8_lossy(&saida.stderr).trim()
        ));
    }
    let texto = String::from_utf8_lossy(&saida.stdout);
    let var = |chave: &str| -> Option<String> {
        texto
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{chave}=")))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
    };
    let mut evidence = vec![format!("{}: carregado pelo sh", script.display())];
    let sysroot = var("SDKTARGETSYSROOT");
    if let Some(s) = &sysroot {
        evidence.push(format!("SDKTARGETSYSROOT={s}"));
    }
    let toolchain_file = var("OECORE_NATIVE_SYSROOT").and_then(|nativo| {
        let arquivo = Path::new(&nativo).join("usr/share/cmake/OEToolchainConfig.cmake");
        arquivo.is_file().then(|| {
            evidence.push(format!("OEToolchainConfig.cmake em {}", arquivo.display()));
            arquivo.display().to_string()
        })
    });
    let target_triple = var("TARGET_PREFIX")
        .map(|p| p.trim_end_matches('-').to_owned())
        .or_else(|| var("CC").and_then(|cc| triple_de(cc.split_whitespace().next()?)));
    if let Some(t) = &target_triple {
        evidence.push(format!("triple {t} (TARGET_PREFIX/CC)"));
    }
    let c_compiler = var("CC_PATH");
    let cxx_compiler = var("CXX_PATH");
    let gdb = var("GDB_PATH");
    if c_compiler.is_none() {
        return Err(format!(
            "o script {} nao exporta um CC que o sh encontre no PATH dele — e' um \
             environment-setup de SDK Yocto?",
            script.display()
        ));
    }
    Ok(KitImport {
        kind: "yocto".to_owned(),
        path: script.display().to_string(),
        evidence,
        c_compiler,
        cxx_compiler,
        gdb,
        sysroot,
        target_triple,
        toolchain_file,
        hint: Some(
            "o CC do SDK ja' leva --sysroot; com o OEToolchainConfig.cmake no kit, o configure \
             e' o do SDK (o do preset vence se houver)"
                .to_owned(),
        ),
    })
}

/// `output/host` de uma arvore Buildroot, dada ela ou o `output/`. A marca
/// e' `share/buildroot/` (onde o Buildroot poe o toolchainfile.cmake) — e os
/// tarballs da Bootlin a trazem tambem, porque SAO SDKs do Buildroot:
/// entram por aqui, com o arquivo de `CMake`. Uma pasta so' com bin/ e
/// `<triple>/sysroot` e' `toolchain-dir`.
fn buildroot_host(path: &Path) -> Option<PathBuf> {
    [
        path.to_path_buf(),
        path.join("host"),
        path.join("output/host"),
    ]
    .into_iter()
    .find(|candidato| {
        candidato.join("bin").is_dir()
            && candidato.join("share/buildroot").is_dir()
            && sysroot_buildroot(candidato).is_some()
    })
}

/// `host/<triple>/sysroot`, o unico que existe.
fn sysroot_buildroot(host: &Path) -> Option<PathBuf> {
    let mut achados: Vec<PathBuf> = std::fs::read_dir(host)
        .ok()?
        .flatten()
        .map(|e| e.path().join("sysroot"))
        .filter(|p| p.is_dir())
        .collect();
    achados.sort();
    (achados.len() == 1).then(|| achados.remove(0))
}

fn buildroot(host: &Path) -> KitImport {
    let bin = host.join("bin");
    let mut evidence = vec![format!("{}: host de Buildroot", host.display())];
    let gcc = gcc_em(&bin);
    let triple = gcc
        .as_ref()
        .and_then(|g| triple_de(&g.file_name()?.to_string_lossy()));
    let sysroot = sysroot_buildroot(host);
    if let Some(s) = &sysroot {
        evidence.push(format!("sysroot em {}", s.display()));
    }
    let toolchain_file = host.join("share/buildroot/toolchainfile.cmake");
    let toolchain_file = toolchain_file.is_file().then(|| {
        evidence.push(format!(
            "toolchainfile.cmake em {}",
            toolchain_file.display()
        ));
        toolchain_file.display().to_string()
    });
    let com_triple = |sufixo: &str| {
        triple
            .as_ref()
            .map(|t| bin.join(format!("{t}-{sufixo}")))
            .filter(|p| p.is_file())
            .map(|p| p.display().to_string())
    };
    KitImport {
        kind: "buildroot".to_owned(),
        path: host.display().to_string(),
        evidence,
        c_compiler: gcc.as_ref().map(|g| g.display().to_string()),
        cxx_compiler: com_triple("g++"),
        gdb: com_triple("gdb"),
        sysroot: sysroot.map(|s| s.display().to_string()),
        target_triple: triple,
        toolchain_file,
        hint: Some(
            "o toolchainfile.cmake do Buildroot ja' fixa compiladores e sysroot; com ele no \
             kit, o configure e' o do Buildroot"
                .to_owned(),
        ),
    }
}

/// Uma pasta de toolchain (tarball da Arm/Bootlin ou a pasta da IDE).
fn toolchain_dir(path: &Path) -> Option<KitImport> {
    let bin = path.join("bin");
    let gcc = gcc_em(&bin)?;
    let triple = triple_de(&gcc.file_name()?.to_string_lossy())?;
    let mut evidence = vec![format!("{}: gcc da toolchain", gcc.display())];
    // O sysroot que o proprio gcc declara e' a verdade; as duas convencoes
    // (Arm: <triple>/libc; Bootlin: <triple>/sysroot) sao o fallback quando o
    // gcc nao pode ser executado (outra arquitetura de host, por exemplo).
    let sysroot = print_sysroot(&gcc)
        .filter(|s| s.is_dir())
        .or_else(|| {
            [
                path.join(&triple).join("libc"),
                path.join(&triple).join("sysroot"),
            ]
            .into_iter()
            .find(|p| p.is_dir())
        })
        .filter(|s| {
            s.join("usr/include").is_dir() || s.join("usr/lib").is_dir() || s.join("lib").is_dir()
        });
    if let Some(s) = &sysroot {
        evidence.push(format!("sysroot em {}", s.display()));
    }
    let com = |sufixo: &str| {
        let p = bin.join(format!("{triple}-{sufixo}"));
        p.is_file().then(|| p.display().to_string())
    };
    Some(KitImport {
        kind: "toolchain-dir".to_owned(),
        path: path.display().to_string(),
        evidence,
        c_compiler: Some(gcc.display().to_string()),
        cxx_compiler: com("g++"),
        gdb: com("gdb"),
        sysroot: sysroot.map(|s| s.display().to_string()),
        target_triple: Some(triple),
        toolchain_file: None,
        hint: Some(
            "toolchain sem arquivo de CMake: o kit leva compiladores, sysroot e triple, e o \
             CMAKE_SYSTEM_NAME sai do triple"
                .to_owned(),
        ),
    })
}

/// O `<triple>-gcc` em `bin` (nao `-gcc-14`, nao `-gcc-ar`), o unico.
fn gcc_em(bin: &Path) -> Option<PathBuf> {
    let mut achados: Vec<PathBuf> = std::fs::read_dir(bin)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name().is_some_and(|n| {
                    let n = n.to_string_lossy();
                    n.ends_with("-gcc")
                })
        })
        .collect();
    achados.sort();
    (achados.len() == 1).then(|| achados.remove(0))
}

/// `aarch64-buildroot-linux-gnu-gcc` -> `aarch64-buildroot-linux-gnu`.
fn triple_de(nome_do_gcc: &str) -> Option<String> {
    nome_do_gcc
        .rsplit_once("-gcc")
        .map(|(t, _)| t.to_owned())
        .filter(|t| !t.is_empty())
}

/// `<gcc> -print-sysroot`, quando o gcc roda nesta maquina.
fn print_sysroot(gcc: &Path) -> Option<PathBuf> {
    let saida = Command::new(gcc).arg("-print-sysroot").output().ok()?;
    if !saida.status.success() {
        return None;
    }
    let texto = String::from_utf8_lossy(&saida.stdout).trim().to_owned();
    (!texto.is_empty()).then(|| PathBuf::from(texto))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};

    use super::import;

    fn raiz(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-core-tests")
            .join(format!("{}-kit-import-{nome}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn executavel(p: &Path, corpo: &str) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, corpo).unwrap();
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// Um SDK Yocto na forma documentada: o environment-setup exporta CC com
    /// --sysroot, `SDKTARGETSYSROOT`, `OECORE_NATIVE_SYSROOT` (com o
    /// `OEToolchainConfig.cmake` dentro) e `TARGET_PREFIX`; os binarios estao
    /// no PATH que o script monta. Tanto o arquivo quanto a pasta servem.
    #[test]
    fn a_yocto_sdk_is_read_by_sourcing_its_environment_script() {
        let sdk = raiz("yocto");
        let nativo = sdk.join("sysroots/x86_64-pokysdk-linux");
        let alvo = sdk.join("sysroots/cortexa72-poky-linux");
        std::fs::create_dir_all(alvo.join("usr/include")).unwrap();
        std::fs::create_dir_all(nativo.join("usr/share/cmake")).unwrap();
        std::fs::write(
            nativo.join("usr/share/cmake/OEToolchainConfig.cmake"),
            "# oe\n",
        )
        .unwrap();
        let bin = nativo.join("usr/bin/aarch64-poky-linux");
        for nome in [
            "aarch64-poky-linux-gcc",
            "aarch64-poky-linux-g++",
            "aarch64-poky-linux-gdb",
        ] {
            executavel(&bin.join(nome), "#!/bin/sh\n");
        }
        let script = sdk.join("environment-setup-cortexa72-poky-linux");
        std::fs::write(
            &script,
            format!(
                "export SDKTARGETSYSROOT={alvo}\nexport OECORE_NATIVE_SYSROOT={nativo}\nexport PATH={bin}:$PATH\nexport TARGET_PREFIX=aarch64-poky-linux-\nexport CC=\"aarch64-poky-linux-gcc -mcpu=cortex-a72 --sysroot=$SDKTARGETSYSROOT\"\nexport CXX=\"aarch64-poky-linux-g++ --sysroot=$SDKTARGETSYSROOT\"\nexport GDB=aarch64-poky-linux-gdb\n",
                alvo = alvo.display(),
                nativo = nativo.display(),
                bin = bin.display()
            ),
        )
        .unwrap();
        for entrada in [script.as_path(), sdk.as_path()] {
            let kit = import(entrada).unwrap();
            assert_eq!(kit.kind, "yocto");
            assert_eq!(
                kit.c_compiler.as_deref(),
                Some(bin.join("aarch64-poky-linux-gcc").to_str().unwrap())
            );
            assert_eq!(
                kit.cxx_compiler.as_deref(),
                Some(bin.join("aarch64-poky-linux-g++").to_str().unwrap())
            );
            assert_eq!(
                kit.gdb.as_deref(),
                Some(bin.join("aarch64-poky-linux-gdb").to_str().unwrap())
            );
            assert_eq!(kit.sysroot.as_deref(), Some(alvo.to_str().unwrap()));
            assert_eq!(kit.target_triple.as_deref(), Some("aarch64-poky-linux"));
            assert_eq!(
                kit.toolchain_file.as_deref(),
                Some(
                    nativo
                        .join("usr/share/cmake/OEToolchainConfig.cmake")
                        .to_str()
                        .unwrap()
                )
            );
            assert!(
                kit.evidence.iter().any(|e| e.contains("carregado pelo sh")),
                "{:?}",
                kit.evidence
            );
        }
        // Um script que nao exporta CC nao e' um SDK: erro que diz isso.
        std::fs::write(sdk.join("environment-setup-vazio"), "export FOO=1\n").unwrap();
        let erro = import(&sdk.join("environment-setup-vazio")).unwrap_err();
        assert!(erro.contains("nao exporta um CC"), "{erro}");
        // Com DOIS scripts na pasta a IDE nao escolhe: aponte o arquivo.
        let erro = import(&sdk).unwrap_err();
        assert!(erro.contains("nao e' um SDK que eu reconheca"), "{erro}");
        // Um script que falha ao carregar: o erro traz o status.
        std::fs::write(sdk.join("environment-setup-quebrado"), "exit 7\n").unwrap();
        let erro = import(&sdk.join("environment-setup-quebrado")).unwrap_err();
        assert!(erro.contains("nao pode ser carregado"), "{erro}");
    }

    /// Uma arvore Buildroot na forma documentada, dada por `output/`, por
    /// `output/host` ou pela raiz da arvore: gcc/g++/gdb do triple, o sysroot
    /// em host/<triple>/sysroot e o toolchainfile.cmake.
    #[test]
    fn a_buildroot_tree_is_read_from_output_or_host() {
        let arvore = raiz("buildroot");
        let host = arvore.join("output/host");
        for nome in [
            "aarch64-buildroot-linux-gnu-gcc",
            "aarch64-buildroot-linux-gnu-g++",
            "aarch64-buildroot-linux-gnu-gdb",
            "aarch64-buildroot-linux-gnu-gcc-ar",
        ] {
            executavel(&host.join("bin").join(nome), "#!/bin/sh\n");
        }
        std::fs::create_dir_all(host.join("aarch64-buildroot-linux-gnu/sysroot/usr/include"))
            .unwrap();
        std::fs::create_dir_all(host.join("share/buildroot")).unwrap();
        std::fs::write(host.join("share/buildroot/toolchainfile.cmake"), "# br\n").unwrap();
        for entrada in [host.clone(), arvore.join("output"), arvore.clone()] {
            let kit = import(&entrada).unwrap_or_else(|e| panic!("{}: {e}", entrada.display()));
            assert_eq!(kit.kind, "buildroot");
            assert_eq!(
                kit.target_triple.as_deref(),
                Some("aarch64-buildroot-linux-gnu")
            );
            assert_eq!(
                kit.c_compiler.as_deref(),
                Some(
                    host.join("bin/aarch64-buildroot-linux-gnu-gcc")
                        .to_str()
                        .unwrap()
                )
            );
            assert_eq!(
                kit.cxx_compiler.as_deref(),
                Some(
                    host.join("bin/aarch64-buildroot-linux-gnu-g++")
                        .to_str()
                        .unwrap()
                )
            );
            assert_eq!(
                kit.gdb.as_deref(),
                Some(
                    host.join("bin/aarch64-buildroot-linux-gnu-gdb")
                        .to_str()
                        .unwrap()
                )
            );
            assert_eq!(
                kit.sysroot.as_deref(),
                Some(
                    host.join("aarch64-buildroot-linux-gnu/sysroot")
                        .to_str()
                        .unwrap()
                )
            );
            assert_eq!(
                kit.toolchain_file.as_deref(),
                Some(
                    host.join("share/buildroot/toolchainfile.cmake")
                        .to_str()
                        .unwrap()
                )
            );
        }
    }

    /// Uma pasta de toolchain sem `share/buildroot` (um tarball generico, ou
    /// o que a IDE instalou): o gcc do triple, o sysroot pelo `-print-sysroot`
    /// do proprio gcc (um falso que o imprime), e sem arquivo de `CMake`. Pasta
    /// sem nada reconhecivel e' erro que diz o que procurou.
    #[test]
    fn a_toolchain_folder_is_read_with_the_sysroot_the_gcc_declares() {
        let pasta = raiz("generica");
        let sysroot = pasta.join("aarch64-buildroot-linux-gnu/sysroot");
        std::fs::create_dir_all(sysroot.join("usr/lib")).unwrap();
        executavel(
            &pasta.join("bin/aarch64-buildroot-linux-gnu-gcc"),
            &format!(
                "#!/bin/sh\n[ \"$1\" = -print-sysroot ] && echo {}\n",
                sysroot.display()
            ),
        );
        executavel(
            &pasta.join("bin/aarch64-buildroot-linux-gnu-g++"),
            "#!/bin/sh\n",
        );
        let kit = import(&pasta).unwrap();
        assert_eq!(kit.kind, "toolchain-dir");
        assert_eq!(
            kit.target_triple.as_deref(),
            Some("aarch64-buildroot-linux-gnu")
        );
        assert_eq!(kit.sysroot.as_deref(), Some(sysroot.to_str().unwrap()));
        assert!(kit.cxx_compiler.is_some() && kit.gdb.is_none());
        assert_eq!(kit.toolchain_file, None);

        // O gcc que nao roda aqui (outra arquitetura de host), ou que declara
        // um sysroot que NAO EXISTE (o pacote de distro): o fallback pela
        // convencao da Arm, <triple>/libc.
        let arm = raiz("arm");
        std::fs::create_dir_all(arm.join("aarch64-none-linux-gnu/libc/usr/include")).unwrap();
        executavel(
            &arm.join("bin/aarch64-none-linux-gnu-gcc"),
            "#!/bin/sh\nexit 1\n",
        );
        let kit = import(&arm).unwrap();
        assert_eq!(
            kit.sysroot.as_deref(),
            Some(arm.join("aarch64-none-linux-gnu/libc").to_str().unwrap())
        );
        executavel(
            &arm.join("bin/aarch64-none-linux-gnu-gcc"),
            "#!/bin/sh\necho /nao/existe/sys-root\n",
        );
        let kit = import(&arm).unwrap();
        assert_eq!(
            kit.sysroot.as_deref(),
            Some(arm.join("aarch64-none-linux-gnu/libc").to_str().unwrap())
        );

        let nada = raiz("nada");
        std::fs::create_dir_all(nada.join("docs")).unwrap();
        let erro = import(&nada).unwrap_err();
        assert!(
            erro.contains("Yocto") && erro.contains("Buildroot") && erro.contains("Zephyr"),
            "{erro}"
        );
        assert!(import(&nada.join("x")).unwrap_err().contains("nao existe"));
    }

    /// Um Zephyr SDK na forma do sdk-ng (lida no setup.sh e no
    /// Zephyr-sdkConfig.cmake em 2026-09-17): `sdk_version`, o pacote `CMake`,
    /// as toolchains em `gnu/<nome>/` (v1.x) e o link de bisectability na
    /// raiz (contado uma vez). Com varias, a proposta e' a `arm-zephyr-eabi`
    /// e a evidencia lista as outras; o sysroot e' o que o gcc declara; sem
    /// toolchain nenhuma, a proposta e' vazia com a dica do setup.sh.
    #[test]
    fn a_zephyr_sdk_proposes_the_arm_toolchain_and_lists_the_others() {
        let sdk = raiz("zephyr");
        std::fs::write(sdk.join("sdk_version"), "1.0.1\n").unwrap();
        std::fs::create_dir_all(sdk.join("cmake")).unwrap();
        std::fs::write(sdk.join("cmake/Zephyr-sdkConfig.cmake"), "# pacote\n").unwrap();
        // Sem toolchain: proposta vazia, dica do setup.sh.
        let vazio = import(&sdk).unwrap();
        assert_eq!(vazio.kind, "zephyr-sdk");
        assert!(vazio.c_compiler.is_none());
        assert!(vazio.hint.as_deref().unwrap().contains("setup.sh"));

        let libc = sdk.join("gnu/arm-zephyr-eabi/arm-zephyr-eabi");
        std::fs::create_dir_all(libc.join("include")).unwrap();
        executavel(
            &sdk.join("gnu/arm-zephyr-eabi/bin/arm-zephyr-eabi-gcc"),
            &format!(
                "#!/bin/sh\n[ \"$1\" = -print-sysroot ] && echo {}\n",
                libc.display()
            ),
        );
        executavel(
            &sdk.join("gnu/arm-zephyr-eabi/bin/arm-zephyr-eabi-g++"),
            "#!/bin/sh\n",
        );
        executavel(
            &sdk.join("gnu/arm-zephyr-eabi/bin/arm-zephyr-eabi-gdb"),
            "#!/bin/sh\n",
        );
        executavel(
            &sdk.join("gnu/riscv64-zephyr-elf/bin/riscv64-zephyr-elf-gcc"),
            "#!/bin/sh\nexit 1\n",
        );
        // O link de bisectability na raiz e' a MESMA toolchain.
        std::os::unix::fs::symlink(sdk.join("gnu/arm-zephyr-eabi"), sdk.join("arm-zephyr-eabi"))
            .unwrap();

        let kit = import(&sdk).unwrap();
        assert_eq!(kit.kind, "zephyr-sdk");
        assert_eq!(kit.target_triple.as_deref(), Some("arm-zephyr-eabi"));
        assert!(
            kit.c_compiler
                .as_deref()
                .unwrap()
                .ends_with("gnu/arm-zephyr-eabi/bin/arm-zephyr-eabi-gcc")
        );
        assert!(kit.cxx_compiler.is_some() && kit.gdb.is_some());
        assert_eq!(kit.sysroot.as_deref(), Some(libc.to_str().unwrap()));
        assert_eq!(kit.toolchain_file, None);
        let evidencia = kit.evidence.join("\n");
        assert!(evidencia.contains("Zephyr SDK 1.0.1"), "{evidencia}");
        assert!(
            evidencia.contains("toolchains GNU instaladas: arm-zephyr-eabi, riscv64-zephyr-elf — proposta: arm-zephyr-eabi"),
            "{evidencia}"
        );
        assert!(
            kit.hint
                .as_deref()
                .unwrap()
                .contains("ZEPHYR_SDK_INSTALL_DIR")
        );

        // So' a RISC-V: a proposta e' ela (a primeira, dita).
        let so_riscv = raiz("zephyr-riscv");
        std::fs::write(so_riscv.join("sdk_version"), "1.0.1\n").unwrap();
        std::fs::create_dir_all(so_riscv.join("cmake")).unwrap();
        std::fs::write(so_riscv.join("cmake/Zephyr-sdkConfig.cmake"), "").unwrap();
        executavel(
            &so_riscv.join("riscv64-zephyr-elf/bin/riscv64-zephyr-elf-gcc"),
            "#!/bin/sh\nexit 1\n",
        );
        let kit = import(&so_riscv).unwrap();
        assert_eq!(kit.target_triple.as_deref(), Some("riscv64-zephyr-elf"));
        assert!(kit.sysroot.is_none());
        // Sem o pacote CMake nao e' um Zephyr SDK: cai no erro geral.
        std::fs::remove_dir_all(so_riscv.join("cmake")).unwrap();
        assert!(import(&so_riscv).is_err());
    }
}
