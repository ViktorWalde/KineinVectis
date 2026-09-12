//! Types for the `project.*` domain: the MODEL of an embedded project.
//!
//! Pillar 0 of `roadmaps/42` (2026-09-12). `workspace.open` says *which build
//! system* a folder has by looking at the root; this domain says *what the
//! project IS*: the embedded framework (with the file that proves it), the
//! SDKs and toolchains it requires and whether they are here, the artifacts
//! the last build produced, and the target deduced from all of that — flash
//! engine, monitor, debug adapter. Every field carries its evidence; nothing
//! is guessed silently (`roadmaps/35` §5.1 item 4).

use serde::{Deserialize, Serialize};

/// An embedded framework the detector recognises.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Framework {
    /// Espressif `IoT` Development Framework: `project.cmake` included, `sdkconfig`.
    EspIdf,
    /// Zephyr RTOS: `find_package(Zephyr)`, `prj.conf`, `west.yml`.
    Zephyr,
    /// Raspberry Pi Pico SDK: `pico_sdk_import.cmake`, `pico_sdk_init()`.
    PicoSdk,
    /// `PlatformIO`: `platformio.ini`.
    PlatformIo,
    /// `STM32CubeMX` project: `*.ioc`.
    Stm32Cube,
    /// Embedded Rust: `.cargo/config.toml` with a bare-metal target, `memory.x`,
    /// `Embed.toml`.
    CargoEmbedded,
    /// `MicroPython`/`CircuitPython` scripts: `boot.py`/`main.py` importing `machine`
    /// or `board`, or `MicroPython` stubs in `typings/`.
    MicroPython,
    /// Yocto/OpenEmbedded build directory or layer: `conf/local.conf`,
    /// `conf/bblayers.conf`, `conf/layer.conf`.
    Yocto,
    /// Buildroot tree or external: `.config` with `BR2_` keys, `external.desc`.
    Buildroot,
}

/// One recognised framework and the file that proves it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkInfo {
    /// Which framework.
    pub framework: Framework,
    /// Workspace-relative path of the marker that decided it.
    pub evidence: String,
    /// What the marker said, when it says something useful: the IDF target,
    /// the `PlatformIO` environments, the `CubeMX` device, the cargo target, the
    /// Yocto `MACHINE`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Something a framework needs on this machine, and whether it is here.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SdkRequirement {
    /// Stable id (`esp-idf`, `xtensa-esp-elf`, `pico-sdk`, `zephyr-sdk`, `west`,
    /// `platformio`, `arm-none-eabi`, `rustup-target`, `mpremote`, `bitbake`).
    pub id: String,
    /// Human label.
    pub label: String,
    /// Environment variable consulted, when there is one (`IDF_PATH`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Where it was found (a directory or an executable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Present on this machine.
    pub found: bool,
    /// The official step to get it — printed, never run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// What the last build left behind, as far as the model can see.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArtifacts {
    /// ELF candidates (newest first).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elf: Vec<String>,
    /// Raw images (`.bin`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bin: Vec<String>,
    /// Intel HEX images.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hex: Vec<String>,
    /// UF2 images (Pico BOOTSEL).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uf2: Vec<String>,
    /// Linker map files.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub map: Vec<String>,
    /// ESP-IDF's `flasher_args.json`: the flash recipe the build wrote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flasher_args: Option<String>,
    /// ESP-IDF partition table (`partitions.csv` or the built `.bin`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partition_table: Option<String>,
    /// Rust `memory.x`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_x: Option<String>,
    /// Linker scripts in the source tree (not in build dirs).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linker_scripts: Vec<String>,
}

/// The target the model deduced, each field with where it came from.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetModel {
    /// Chip name (`esp32`, `STM32F401CCUx`, `rp2040`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    /// Family the engines key on: `espressif`, `stm32`, `rp2040`, `nrf`,
    /// `cortex-m`, `riscv`, `linux`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    /// Target triple, when a build system declares one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triple: Option<String>,
    /// Suggested flash engine: `esptool`, `probe-rs`, `picotool`, `dfu-util`,
    /// `west`, `platformio`, `openocd`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_engine: Option<String>,
    /// Suggested monitor: `espflash`, `serial`, `rtt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<String>,
    /// Suggested DAP adapter: `probe-rs`, `gdb`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_adapter: Option<String>,
    /// Why: one line per deduction (`chip: sdkconfig CONFIG_IDF_TARGET`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Result payload for `project.model`, and payload of `event.project.changed`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectModel {
    /// Workspace root.
    pub root: String,
    /// Any embedded framework recognised.
    pub embedded: bool,
    /// Frameworks found, most specific first.
    pub frameworks: Vec<FrameworkInfo>,
    /// What they require, and what is here.
    pub sdks: Vec<SdkRequirement>,
    /// What the last build produced.
    pub artifacts: ProjectArtifacts,
    /// The deduced target.
    pub target: TargetModel,
    /// What the model could not decide, in words.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

/// Parameters for `project.model`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectModelParams {}
