//! O catalogo do provedor de instalacao: releases PINADOS, com o checksum que
//! a FONTE publicou (`integracoes/39` §5). Nunca "latest".
//!
//! Cada entrada foi medida em 2026-09-13: o arquivo `.sha256`/`.sha256asc`/
//! `.sha` da fonte foi baixado e lido, e o tamanho veio do `Content-Length`
//! de um HEAD no tarball. Um numero daqui que divergir da fonte e' a fonte
//! que mudou o release — e ai' a entrada e' que envelhece, nao o usuario que
//! recebe um binario trocado: o SHA-256 e' conferido antes de desempacotar.
//!
//! O que NAO esta' aqui, e por que (`integracoes/39` §5/§6): Espressif (o
//! `idf_tools.py` do ESP-IDF e' o instalador oficial, e a IDE ja' le
//! `~/.espressif/tools`); Zephyr SDK (precisa do `setup.sh` — e' importar
//! kit, P1); "latest" de qualquer fonte.

/// Uma toolchain instalavel: o que a tela mostra ANTES do clique.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entrada {
    /// Id do catalogo e nome da pasta sob a raiz de instalacao.
    pub id: &'static str,
    /// Rotulo humano.
    pub label: &'static str,
    /// Versao pinada.
    pub version: &'static str,
    /// Familia que ela serve (a mesma palavra do `project.model`).
    pub family: &'static str,
    /// URL exata do tarball (linux `x86_64`).
    pub url: &'static str,
    /// Tamanho em bytes (`Content-Length` medido).
    pub size_bytes: u64,
    /// SHA-256 publicado pela fonte, hexadecimal minusculo.
    pub sha256: &'static str,
    /// Licenca da toolchain, lida na fonte.
    pub license: &'static str,
    /// De onde o checksum foi lido.
    pub source: &'static str,
}

/// O catalogo, na ordem em que a tela lista.
pub const CATALOGO: &[Entrada] = &[
    Entrada {
        id: "arm-gnu-arm-none-eabi",
        label: "Arm GNU Toolchain (arm-none-eabi) — Cortex-M/R bare metal",
        version: "15.2.rel1",
        family: "cortex-m",
        url: "https://developer.arm.com/-/media/Files/downloads/gnu/15.2.rel1/binrel/arm-gnu-toolchain-15.2.rel1-x86_64-arm-none-eabi.tar.xz",
        size_bytes: 155_499_480,
        sha256: "597893282ac8c6ab1a4073977f2362990184599643b4c5ee34870a8215783a16",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (newlib, libstdc++) — processo, nunca crate",
        source: "developer.arm.com, arquivo .sha256asc do release, lido em 2026-09-13",
    },
    Entrada {
        id: "xpack-arm-none-eabi-gcc",
        label: "xPack GNU Arm Embedded GCC (arm-none-eabi)",
        version: "15.2.1-1.1",
        family: "cortex-m",
        url: "https://github.com/xpack-dev-tools/arm-none-eabi-gcc-xpack/releases/download/v15.2.1-1.1/xpack-arm-none-eabi-gcc-15.2.1-1.1-linux-x64.tar.gz",
        size_bytes: 307_090_703,
        sha256: "da6a49ad4003944b823c6c93702a8787c922ab34bd7e918ec0eaf6933a9b1ff6",
        license: "MIT (o empacotamento xPack; GCC/binutils GPL-3.0) — processo",
        source: "GitHub release, arquivo .sha, lido em 2026-09-13",
    },
    Entrada {
        id: "xpack-riscv-none-elf-gcc",
        label: "xPack GNU RISC-V Embedded GCC (riscv-none-elf)",
        version: "15.2.0-1",
        family: "riscv",
        url: "https://github.com/xpack-dev-tools/riscv-none-elf-gcc-xpack/releases/download/v15.2.0-1/xpack-riscv-none-elf-gcc-15.2.0-1-linux-x64.tar.gz",
        size_bytes: 433_494_794,
        sha256: "aaaa8060c914851a3e5ee1ba82cc3d6f80972f90638a05c6e823a37557a33758",
        license: "MIT (o empacotamento xPack; GCC/binutils GPL-3.0) — processo",
        source: "GitHub release, arquivo .sha, lido em 2026-09-13",
    },
    Entrada {
        id: "arm-toolchain-for-embedded",
        label: "Arm Toolchain for Embedded (clang/LLVM, bare metal)",
        version: "23.1.0",
        family: "cortex-m",
        url: "https://github.com/arm/arm-toolchain/releases/download/release-23.1.0-ATfE/ATfE-23.1.0-Linux-x86_64.tar.xz",
        size_bytes: 213_607_916,
        sha256: "a7be511613af15151c93961ad39c8cf9ae6889a453c8b547f42f4051a416dc8e",
        license: "Apache-2.0 WITH LLVM-exception (LICENSE.TXT do repositorio)",
        source: "GitHub release, arquivo .sha256, lido em 2026-09-13",
    },
    Entrada {
        id: "arm-gnu-aarch64-none-linux-gnu",
        label: "Arm GNU Toolchain (aarch64-none-linux-gnu) — Linux 64 bits, com sysroot",
        version: "15.2.rel1",
        family: "aarch64-linux",
        url: "https://developer.arm.com/-/media/Files/downloads/gnu/15.2.rel1/binrel/arm-gnu-toolchain-15.2.rel1-x86_64-aarch64-none-linux-gnu.tar.xz",
        size_bytes: 159_241_804,
        sha256: "9a685b335bd709d683a8c782253c37e8c36c10e6924e59e39d4769b02132eb43",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (glibc, libstdc++) — processo",
        source: "developer.arm.com, arquivo .sha256asc do release, lido em 2026-09-13",
    },
    Entrada {
        id: "arm-gnu-arm-none-linux-gnueabihf",
        label: "Arm GNU Toolchain (arm-none-linux-gnueabihf) — Linux 32 bits hard-float, com sysroot",
        version: "15.2.rel1",
        family: "arm-linux",
        url: "https://developer.arm.com/-/media/Files/downloads/gnu/15.2.rel1/binrel/arm-gnu-toolchain-15.2.rel1-x86_64-arm-none-linux-gnueabihf.tar.xz",
        size_bytes: 134_624_396,
        sha256: "3c65d820a6b8f677f8f6fbfc749fe00a4f16dde12341436c9df5b7092a47c0fb",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (glibc, libstdc++) — processo",
        source: "developer.arm.com, arquivo .sha256asc do release, lido em 2026-09-13",
    },
    Entrada {
        id: "bootlin-aarch64-glibc-stable",
        label: "Bootlin aarch64 glibc stable — Linux 64 bits, com sysroot",
        version: "2026.08-1",
        family: "aarch64-linux",
        url: "https://toolchains.bootlin.com/downloads/releases/toolchains/aarch64/tarballs/aarch64--glibc--stable-2026.08-1.tar.xz",
        size_bytes: 99_347_372,
        sha256: "0213efac9b5577f20d58de9431960a191347ffc2257b27ffe7250522bf1f7867",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (glibc) — Buildroot; processo",
        source: "toolchains.bootlin.com, arquivo .sha256, lido em 2026-09-13",
    },
    Entrada {
        id: "bootlin-armv7-eabihf-glibc-stable",
        label: "Bootlin armv7-eabihf glibc stable — Linux 32 bits hard-float, com sysroot",
        version: "2026.08-1",
        family: "arm-linux",
        url: "https://toolchains.bootlin.com/downloads/releases/toolchains/armv7-eabihf/tarballs/armv7-eabihf--glibc--stable-2026.08-1.tar.xz",
        size_bytes: 91_223_644,
        sha256: "9b7e25a74e87dac1e05d399444295e254a3073a056101e3197a859490e5701cd",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (glibc) — Buildroot; processo",
        source: "toolchains.bootlin.com, arquivo .sha256, lido em 2026-09-13",
    },
    Entrada {
        id: "bootlin-riscv64-lp64d-glibc-stable",
        label: "Bootlin riscv64-lp64d glibc stable — Linux RISC-V 64, com sysroot",
        version: "2026.08-1",
        family: "riscv64-linux",
        url: "https://toolchains.bootlin.com/downloads/releases/toolchains/riscv64-lp64d/tarballs/riscv64-lp64d--glibc--stable-2026.08-1.tar.xz",
        size_bytes: 114_411_884,
        sha256: "5807565bd7af28d6be94d3e1a7da9f40d9a1875c741191a62c4c8024fd9467df",
        license: "GPL-3.0 (binutils/gcc/gdb) + LGPL (glibc) — Buildroot; processo",
        source: "toolchains.bootlin.com, arquivo .sha256, lido em 2026-09-13",
    },
];

/// A entrada de `id`, se existe.
#[must_use]
pub fn entrada(id: &str) -> Option<&'static Entrada> {
    CATALOGO.iter().find(|e| e.id == id)
}

/// A familia do catalogo que serve uma familia do `project.model`.
///
/// O modelo diz `cortex-m`, `riscv`, `espressif`, `stm32`, `rp2040`, `nrf`,
/// `linux` (`TargetModel.family`); o catalogo fala a lingua da toolchain.
#[must_use]
pub fn familia_do_catalogo(familia_do_modelo: &str) -> Option<&'static str> {
    match familia_do_modelo {
        "cortex-m" | "stm32" | "rp2040" | "nrf" => Some("cortex-m"),
        "riscv" => Some("riscv"),
        // `linux` sem arquitetura no modelo: nao se recomenda nada.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CATALOGO, entrada, familia_do_catalogo};

    /// O catalogo e' o contrato com a fonte: ids unicos, SHA-256 hexadecimal
    /// de 64 digitos, tamanho e URL https com o nome do arquivo — e a versao
    /// aparece na URL (pinada, nunca "latest").
    #[test]
    fn every_entry_is_pinned_with_a_full_sha256_and_an_https_url() {
        let mut ids = Vec::new();
        for e in CATALOGO {
            assert!(!ids.contains(&e.id), "id repetido: {}", e.id);
            ids.push(e.id);
            assert_eq!(e.sha256.len(), 64, "{}", e.id);
            assert!(
                e.sha256
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{}",
                e.id
            );
            assert!(e.url.starts_with("https://"), "{}", e.id);
            assert!(
                e.url.contains(e.version),
                "a versao pinada esta' na URL: {}",
                e.id
            );
            assert!(!e.url.contains("latest"), "{}", e.id);
            assert!(
                e.size_bytes > 50_000_000,
                "toolchain de {} bytes? {}",
                e.size_bytes,
                e.id
            );
            assert!(
                e.url.ends_with(".tar.xz") || e.url.ends_with(".tar.gz"),
                "{}",
                e.id
            );
            assert!(!e.license.is_empty() && e.source.contains("2026-09-13"));
        }
        assert_eq!(
            entrada("arm-gnu-arm-none-eabi").map(|e| e.version),
            Some("15.2.rel1")
        );
        assert_eq!(entrada("nao-existe"), None);
    }

    #[test]
    fn model_families_map_to_catalogue_families() {
        assert_eq!(familia_do_catalogo("stm32"), Some("cortex-m"));
        assert_eq!(familia_do_catalogo("rp2040"), Some("cortex-m"));
        assert_eq!(familia_do_catalogo("riscv"), Some("riscv"));
        assert_eq!(
            familia_do_catalogo("espressif"),
            None,
            "Espressif instala pelo idf_tools.py"
        );
        assert_eq!(familia_do_catalogo("linux"), None);
    }
}
