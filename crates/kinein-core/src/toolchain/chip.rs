//! The chip's Espressif Xtensa driver (52 §14.42).
//!
//! Without `-mdynconfig`, `xtensa-esp-elf-gcc` targets a big-endian Xtensa
//! core (measured on esp-15.2.0: `__BYTE_ORDER__ __ORDER_BIG_ENDIAN__`); the
//! ESP-IDF build compiles with `xtensa-<chip>-elf-gcc`, the little-endian
//! driver of the chip in the same folder. A kit with an Xtensa chip resolves
//! the generic driver to that sibling, so configure, clangd and the drift
//! check see the build's compiler. RISC-V Espressif chips use
//! `riscv32-esp-elf-gcc` with `-march` and have no per-chip driver.

use std::path::Path;

use kinein_protocol::ToolchainRole;

/// Espressif chips with an Xtensa core and a per-chip driver.
const XTENSA_CHIPS: [&str; 3] = ["esp32", "esp32s2", "esp32s3"];

/// `path`, or the chip's sibling driver when `path` is the generic Xtensa
/// driver of the C/C++ role and the kit names an Xtensa chip.
#[must_use]
pub(super) fn chip_driver(
    role: ToolchainRole,
    effective_id: Option<&str>,
    path: Option<String>,
    chip: Option<&str>,
) -> Option<String> {
    let tool = match (role, effective_id) {
        (ToolchainRole::CCompiler, Some("xtensa-esp-elf-gcc")) => "gcc",
        (ToolchainRole::CxxCompiler, Some("xtensa-esp-elf-gxx")) => "g++",
        _ => return path,
    };
    let Some(chip) = chip.filter(|chip| XTENSA_CHIPS.contains(chip)) else {
        return path;
    };
    let generic = Path::new(path.as_deref()?);
    if generic.file_stem().and_then(|stem| stem.to_str()) != Some(&format!("xtensa-esp-elf-{tool}"))
    {
        return path;
    }
    let mut sibling = generic.with_file_name(format!("xtensa-{chip}-elf-{tool}"));
    if let Some(extension) = generic.extension() {
        sibling.set_extension(extension);
    }
    if sibling.is_file() && crate::run::is_executable(&sibling) {
        Some(sibling.display().to_string())
    } else {
        path
    }
}

/// The generic driver beside a chip driver, when it exists. `--query-driver`
/// lists it too, so a `.clangd` that names the generic driver (written before
/// the kit resolved the chip's) keeps its builtin headers (52 §14.42).
#[must_use]
pub(super) fn generic_sibling(path: &str) -> Option<String> {
    let driver = Path::new(path);
    let stem = driver.file_stem()?.to_str()?;
    let tool = ["gcc", "g++"].into_iter().find(|tool| {
        XTENSA_CHIPS
            .iter()
            .any(|chip| stem == format!("xtensa-{chip}-elf-{tool}"))
    })?;
    let mut generic = driver.with_file_name(format!("xtensa-esp-elf-{tool}"));
    if let Some(extension) = driver.extension() {
        generic.set_extension(extension);
    }
    (generic.is_file() && crate::run::is_executable(&generic))
        .then(|| generic.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake installation; each entry is the path to run (`.exe` on Windows).
    fn installation(drivers: &[&str]) -> Vec<String> {
        let bin = std::env::temp_dir()
            .join("kinein-toolchain")
            .join(format!("{}-chip-drivers", std::process::id()));
        let _ = std::fs::remove_dir_all(&bin);
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::create_dir_all(bin.join("xtensa-x")).unwrap();
        drivers
            .iter()
            .map(|driver| {
                crate::write_executable(bin.join(driver), "#!/bin/sh\n")
                    .display()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn an_xtensa_chip_resolves_the_generic_driver_to_its_own() {
        // A RISC-V name present as an Xtensa sibling, and a file a crafted chip
        // could reach: the chip list keeps the kit to known drivers only.
        let paths = installation(&[
            "xtensa-esp-elf-gcc",
            "xtensa-esp-elf-g++",
            "xtensa-esp32-elf-gcc",
            "xtensa-esp32s3-elf-g++",
            "xtensa-esp32c3-elf-gcc",
            "xtensa-x/../escape-elf-gcc",
        ]);
        let (generic_c, generic_cxx) = (Some(paths[0].clone()), Some(paths[1].clone()));
        let c = |chip| {
            chip_driver(
                ToolchainRole::CCompiler,
                Some("xtensa-esp-elf-gcc"),
                generic_c.clone(),
                chip,
            )
        };
        assert_eq!(c(Some("esp32")), Some(paths[2].clone()));
        assert_eq!(c(None), generic_c, "no chip in the kit");
        assert_eq!(c(Some("esp32s2")), generic_c, "sibling missing");
        assert_eq!(c(Some("esp32c3")), generic_c, "RISC-V chip");
        assert_eq!(c(Some("x/../escape")), generic_c, "a chip outside the list");
        assert_eq!(
            chip_driver(
                ToolchainRole::CxxCompiler,
                Some("xtensa-esp-elf-gxx"),
                generic_cxx,
                Some("esp32s3")
            ),
            Some(paths[3].clone())
        );
        let other = Some("/usr/bin/arm-none-eabi-gcc".to_owned());
        assert_eq!(
            chip_driver(
                ToolchainRole::CCompiler,
                Some("arm-none-eabi-gcc"),
                other.clone(),
                Some("esp32")
            ),
            other
        );
        assert_eq!(
            generic_sibling(&paths[2]),
            generic_c,
            "the generic beside esp32"
        );
        assert_eq!(generic_sibling(&paths[0]), None, "already generic");
        assert_eq!(
            generic_sibling(&paths[4]),
            None,
            "not an Xtensa chip driver"
        );
        assert_eq!(
            chip_driver(
                ToolchainRole::DebugAdapter,
                Some("xtensa-esp-elf-gcc"),
                generic_c.clone(),
                Some("esp32")
            ),
            generic_c,
            "only the compiler roles"
        );
    }
}

#[cfg(test)]
mod kit_tests {
    use kinein_protocol::{ToolInfo, ToolStatus};

    use super::super::Toolchain;

    fn detected(id: &str, path: &str) -> ToolInfo {
        ToolInfo {
            id: id.to_owned(),
            display_name: id.to_owned(),
            status: ToolStatus::Detected,
            path: Some(path.to_owned()),
            version: Some("15.2.0".to_owned()),
            suggested_install: None,
            message: None,
        }
    }

    /// With the chip in the kit, clangd may query the chip's driver and the
    /// generic one a hand-written `.clangd` may still name.
    #[test]
    fn the_kit_hands_clangd_both_xtensa_drivers() {
        let root = std::env::temp_dir()
            .join("kinein-toolchain")
            .join(format!("{}-chip-kit", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".kinein")).unwrap();
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let path = |name: &str| {
            crate::write_executable(bin.join(name), "#!/bin/sh\n")
                .display()
                .to_string()
        };
        let (generic_c, generic_cxx) = (path("xtensa-esp-elf-gcc"), path("xtensa-esp-elf-g++"));
        let (chip_c, chip_cxx) = (path("xtensa-esp32-elf-gcc"), path("xtensa-esp32-elf-g++"));
        std::fs::write(
            root.join(".kinein/toolchain.json"),
            r#"{"schemaVersion":2,"kits":{"":{"selections":{"cCompiler":"xtensa-esp-elf-gcc","cxxCompiler":"xtensa-esp-elf-gxx"},"chip":"esp32"}}}"#,
        )
        .unwrap();
        let tools = [
            detected("xtensa-esp-elf-gcc", &generic_c),
            detected("xtensa-esp-elf-gxx", &generic_cxx),
        ];
        let kit = Toolchain::resolve_kit(&root, &tools, "");
        assert_eq!(
            kit.program_for(kinein_protocol::ToolchainRole::CCompiler),
            Some(std::path::PathBuf::from(&chip_c))
        );
        assert!(
            kit.clangd_args().contains(&format!(
                "--query-driver={chip_c},{generic_c},{chip_cxx},{generic_cxx}"
            )),
            "{:?}",
            kit.clangd_args()
        );
    }
}
