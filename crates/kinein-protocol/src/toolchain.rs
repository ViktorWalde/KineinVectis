//! Toolchain payloads (`toolchain.*`).
//!
//! Uma TOOLCHAIN e a resposta a uma pergunta que ate 2026-09-02 nao tinha dono
//! neste projeto: **qual executavel roda cada papel?** Compilador, gerador e os
//! proprios `cmake`/`cargo` eram "o que estiver no `PATH`", entao trocar de
//! compilador exigia editar arquivo a mao ou exportar `CC`/`CXX` antes de abrir
//! a IDE — o "kit" que `CLion` e Qt Creator expoem nao existia
//! (`roadmaps/29` §5d, B2 do TR2).
//!
//! O vocabulario de PAPEIS e fechado de proposito. Papel novo e' entrada nova
//! aqui e no catalogo do core, nao string livre vinda da UI: um papel que o
//! core nao sabe usar nao configura nada, e o usuario ficaria com um seletor
//! que nao faz efeito.

use serde::{Deserialize, Serialize};

/// Papel que um executavel cumpre na construcao do projeto.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ToolchainRole {
    /// Compilador C (`CMAKE_C_COMPILER`).
    CCompiler,
    /// Compilador C++ (`CMAKE_CXX_COMPILER`).
    CxxCompiler,
    /// Gerador do `CMake` (`-G`).
    Generator,
    /// O proprio `cmake`.
    Cmake,
    /// O proprio `cargo`.
    Cargo,
    /// Adaptador de debug (DAP). `lldb-dap` no desktop; `probe-rs` em
    /// embarcado, que fala DAP nativamente por stdin/stdout.
    DebugAdapter,
    /// Monitor serial: o PROCESSO que a IDE abre numa aba de terminal sobre a
    /// porta da placa (`tio`, `picocom`, `minicom`, `espflash monitor`).
    /// Decisao do autor em 2026-09-11: processo pronto, nunca codigo serial
    /// nosso (`integracoes/38` §6).
    SerialMonitor,
}

impl ToolchainRole {
    /// Chave estavel usada no arquivo e no protocolo.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CCompiler => "cCompiler",
            Self::CxxCompiler => "cxxCompiler",
            Self::Generator => "generator",
            Self::Cmake => "cmake",
            Self::Cargo => "cargo",
            Self::DebugAdapter => "debugAdapter",
            Self::SerialMonitor => "serialMonitor",
        }
    }

    /// Todos os papeis, na ordem em que a UI os mostra.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::CCompiler,
            Self::CxxCompiler,
            Self::Generator,
            Self::Cmake,
            Self::Cargo,
            Self::DebugAdapter,
            Self::SerialMonitor,
        ]
    }
}

/// Um executavel que pode cumprir um papel neste computador.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainCandidate {
    /// Papel que ele cumpre.
    pub role: ToolchainRole,
    /// Identificador estavel do candidato (`clang`, `gxx`, `ninja`, ...).
    ///
    /// Para compiladores e ferramentas e o mesmo `id` do `tools.detect`; para
    /// o gerador e o nome que o `CMake` aceita em `-G`, normalizado.
    pub id: String,
    /// Rotulo curto para a UI.
    pub label: String,
    /// Caminho absoluto do binario, quando ha um.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Versao reportada pelo binario, quando o probe respondeu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// O que esta escolhido para um papel.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainSelection {
    /// Papel escolhido.
    pub role: ToolchainRole,
    /// Id do candidato escolhido; ausente e' AUTOMATICO (o que estiver no
    /// `PATH`), que e o comportamento historico e continua sendo o padrao.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Caminho que o core vai usar de fato, ja resolvido.
    ///
    /// Ausente quando a escolha e automatica e o binario nao foi encontrado —
    /// a UI mostra isso como "nao detectado" em vez de fingir que ha kit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
    /// Id do que sera' USADO — a escolha do autor, ou a que o core fez por ele.
    ///
    /// POR QUE E' SEPARADO DE [`Self::id`] (2026-09-04). `id` continua sendo
    /// "o que o AUTOR fixou", e ausencia ali continua significando "nao fixei
    /// nada". Misturar os dois apagaria a diferenca entre uma escolha e um
    /// palpite, e a UI precisa dela para nao mostrar como decisao do autor
    /// algo que ele nunca decidiu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_id: Option<String>,
    /// `true` quando o `effective_id` foi escolhido pelo CORE, nao pelo autor.
    ///
    /// Decisao do autor em 2026-09-04, mesclando as duas saidas que estavam em
    /// aberto: a IDE escolhe sozinha (para nao parar esperando) **e** mostra
    /// que escolheu (para o autor poder discordar). Nunca fica em silencio.
    #[serde(default)]
    pub automatic: bool,
}

/// Result payload de todo metodo `toolchain.*`.
///
/// Os tres metodos respondem o MESMO shape, como as run configs: a UI nunca
/// calcula estado derivado nem precisa casar respostas diferentes.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainResult {
    /// Kit lido/escrito; vazio = o kit padrao do workspace.
    #[serde(default)]
    pub preset: String,
    /// Raiz do sistema alvo, quando escolhida.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Triple do alvo, quando escolhido.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
    /// Chip do alvo, quando escolhido.
    ///
    /// Vai no `launch` do DAP, **nao** na linha de comando do adaptador:
    /// verificado na documentacao do probe-rs, onde `chip` e' campo da
    /// configuracao de launch e nao flag do `dap-server`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    /// Alvo REMOTO do depurador, quando o kit fala com um servidor GDB:
    /// `host:porta` que vai no `target remote` (0.89.0).
    ///
    /// E' o que liga o `gdb -i dap` a um QEMU, a um `OpenOCD` ou a um `pyOCD` —
    /// o GDB fala DAP desde a v14 (NEWS do gdb) e o `attach` dele leva
    /// `target`, "passed to the `target remote` command" (manual, capitulo
    /// Debugger Adapter Protocol). Sem este campo o adaptador `gdb` faz
    /// `launch`, que e' o desktop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_target: Option<String>,
    /// Comando que a IDE sobe ANTES de conectar ao alvo remoto, e derruba ao
    /// fim da sessao. `{program}` e' substituido pelo ELF (0.89.0).
    ///
    /// Ex.: `qemu-system-arm -machine lm3s6965evb -nographic -S -gdb tcp::3333
    /// -kernel {program}`, ou `openocd -f openocd.cfg`. Declarado pelo usuario,
    /// nunca deduzido: a IDE nao sabe qual maquina do QEMU e' a placa dele.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_server: Option<String>,
    /// Arquivo de toolchain que o PRESET declara (`toolchainFile`), quando ha.
    ///
    /// E informacao, nao escolha: quem manda nele e o `CMakePresets.json`, e a
    /// IDE mostra para o usuario nao procurar no lugar errado quando o
    /// compilador efetivo nao for o que ele escolheu aqui.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset_toolchain_file: Option<String>,
    /// Arquivo de toolchain do KIT (`0.104.0`): o `toolchainfile.cmake` do
    /// Buildroot, o `OEToolchainConfig.cmake` do SDK Yocto — importado ou
    /// digitado. Vira `-DCMAKE_TOOLCHAIN_FILE` no configure quando o preset
    /// nao declara um (o do preset vence).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_file: Option<String>,
    /// Escolha atual de cada papel, na ordem de [`ToolchainRole::all`].
    pub selections: Vec<ToolchainSelection>,
    /// O que existe nesta maquina para cada papel.
    pub candidates: Vec<ToolchainCandidate>,
    /// Alvos Rust INSTALADOS (`rustup target list --installed`), quando o
    /// `rustup` existe; `None` sem rustup (0.97.0, `integracoes/39`). Com o
    /// `targetTriple` do kit fora desta lista, a IDE diz o `rustup target add`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rust_targets: Option<Vec<String>>,
    /// O compilador cross efetivo nao traz o sistema alvo (0.97.0): medido com
    /// `<cc> -print-sysroot`, e' o caso dos `gcc-aarch64-linux-gnu` das
    /// distros — sem `usr/include` no sysroot, nenhum programa de usuario
    /// compila. Diz de onde vem um sysroot: a placa, a Bootlin, o SDK Yocto.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot_hint: Option<String>,
}

/// Parameters for `toolchain.get`.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainGetParams {
    /// Kit to read. `None` = the workspace default kit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// Parameters for `toolchain.set`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainSetParams {
    /// Papel a mudar.
    pub role: ToolchainRole,
    /// Id do candidato; ausente volta para automatico.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Kit a mudar. `None` = o kit padrao do workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

/// Parameters for `toolchain.setKit` — sysroot e alvo de cross-compilacao.
///
/// Campo ausente NAO e o mesmo que campo vazio: ausente preserva o valor
/// atual, string vazia limpa. Sem isso, mexer no sysroot apagaria o target.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainSetKitParams {
    /// Kit a mudar. `None` = o kit padrao do workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    /// Raiz do sistema alvo (`CMAKE_SYSROOT`). `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// Triple do alvo (`--target` do cargo, `CMAKE_SYSTEM_*`). `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
    /// Chip do alvo, para o adaptador de debug de embarcado. `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    /// `host:porta` do servidor GDB (`target remote`). `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_target: Option<String>,
    /// Comando do servidor que a IDE sobe antes de conectar. `""` limpa.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_server: Option<String>,
    /// Arquivo de toolchain do kit (`CMAKE_TOOLCHAIN_FILE`). `""` limpa
    /// (`0.104.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_file: Option<String>,
}

/// Parameters for `toolchain.inspectSysroot`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SysrootInspectParams {
    /// The folder to read (absolute).
    pub path: String,
}

/// What a sysroot folder actually contains (`0.104.0`, `integracoes/39` §3):
/// the answer to "will `--sysroot` here find headers and libraries?".
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysrootReport {
    /// The folder as given.
    pub path: String,
    /// The folder exists and is a directory.
    pub exists: bool,
    /// Which of the three folders a compiler needs are there.
    pub folders: SysrootFolders,
    /// Multiarch/triple library dirs found (`usr/lib/aarch64-linux-gnu`, …).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triple_lib_dirs: Vec<String>,
    /// `.pc` files under `usr/lib*/pkgconfig` and `usr/share/pkgconfig`.
    pub pkgconfig_files: u32,
    /// The C library, when readable (`glibc 2.39`, `musl`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libc: Option<String>,
    /// One line: usable, generic, or empty — and why.
    pub verdict: String,
}

/// The three folders `--sysroot` needs, each present or not.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysrootFolders {
    /// `usr/include` — headers.
    pub usr_include: bool,
    /// `usr/lib` — libraries.
    pub usr_lib: bool,
    /// `lib` — the loader and the libc of a real root.
    pub lib: bool,
}

/// Parameters for `toolchain.importKit`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KitImportParams {
    /// A Yocto SDK `environment-setup-*` script (or its folder), a Buildroot
    /// `output/` (or `output/host`), or a toolchain folder with `bin/`.
    pub path: String,
}

/// A kit PROPOSED from an SDK on disk (`0.104.0`): nothing is written until
/// `toolchain.setKit` applies it.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KitImport {
    /// `yocto`, `buildroot`, `toolchain-dir`.
    pub kind: String,
    /// The path that was read.
    pub path: String,
    /// One line per file that proved something.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
    /// Absolute path of the C compiler, when found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c_compiler: Option<String>,
    /// Absolute path of the C++ compiler, when found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cxx_compiler: Option<String>,
    /// Absolute path of the target `gdb`, when found.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gdb: Option<String>,
    /// The target sysroot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sysroot: Option<String>,
    /// The target triple (`aarch64-poky-linux`, `aarch64-buildroot-linux-gnu`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_triple: Option<String>,
    /// The `CMake` toolchain file the SDK ships, when it ships one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain_file: Option<String>,
    /// What the IDE could not decide, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// One toolchain the IDE can install into its own folder (`integracoes/39`
/// §5): a pinned release with the checksum its source published.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallableToolchain {
    /// Catalogue id, also the folder under the install root
    /// (`arm-gnu-arm-none-eabi`).
    pub id: String,
    /// Human label (`Arm GNU Toolchain (arm-none-eabi)`).
    pub label: String,
    /// Pinned version (`15.2.rel1`) — never "latest".
    pub version: String,
    /// The family it serves: `cortex-m`, `riscv`, `aarch64-linux`,
    /// `arm-linux`, `riscv64-linux`.
    pub family: String,
    /// Exact download URL, shown before the click.
    pub url: String,
    /// Size of the archive in bytes, as the server reported when catalogued.
    pub size_bytes: u64,
    /// SHA-256 the source published (hex, lowercase), verified before
    /// extraction.
    pub sha256: String,
    /// License of the toolchain, read in its source.
    pub license: String,
    /// Where the checksum was read (`arm.com .sha256asc`, `GitHub release
    /// .sha`, `toolchains.bootlin.com .sha256`).
    pub source: String,
    /// Where it lands: `<install root>/<id>/<version>`.
    pub install_dir: String,
    /// `true` when `<install_dir>/bin` already exists (toolchain) or the
    /// firmware file is in place.
    pub installed: bool,
    /// `true` when the open project's family matches this toolchain — or,
    /// for a firmware, when the project is `MicroPython` on that family.
    pub recommended: bool,
    /// `toolchain` (tarball with `bin/`) or `firmware` (one `.bin`/`.uf2`
    /// flashed by `runConfig.flashProposal { firmware }`; `0.116.0`).
    #[serde(default = "default_kind")]
    pub kind: String,
    /// How a firmware is flashed; absent for toolchains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware: Option<InstallableFirmware>,
}

fn default_kind() -> String {
    "toolchain".to_owned()
}

/// How an installable firmware is written to the board (`0.116.0`, C5 of
/// `roadmaps/41` bloco C), as the source page says.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallableFirmware {
    /// Board name at the source (`ESP32_GENERIC`, `RPI_PICO_W`).
    pub board: String,
    /// Flash engine (`esptool`, `picotool`).
    pub engine: String,
    /// `write-flash` offset for `esptool` (`0x1000` on the classic ESP32,
    /// `0x0` on C3/S3); absent for UF2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    /// Chip key for `--chip` (`esp32`), when the page fixes one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chip: Option<String>,
    /// Where the file is once installed (`<install_dir>/<file>`).
    pub file: String,
}

/// Result of `toolchain.installable`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainInstallableResult {
    /// The IDE's toolchain folder.
    pub install_root: String,
    /// Every catalogued toolchain, with its state on this machine.
    pub toolchains: Vec<InstallableToolchain>,
    /// The family the open project asks for, when a workspace is open and the
    /// model deduced one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_family: Option<String>,
}

/// Parameters for `toolchain.install`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolchainInstallParams {
    /// Catalogue id of the toolchain to install.
    pub id: String,
}

/// Payload of `event.toolchain.installed`: the install job ended.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolchainInstalledEvent {
    /// Job id.
    pub job_id: String,
    /// Catalogue id.
    pub id: String,
    /// Pinned version.
    pub version: String,
    /// Where it was installed (`<root>/<id>/<version>`).
    pub path: String,
    /// `true` when the archive was verified, extracted and `bin/` exists.
    pub success: bool,
    /// Why it failed, in words (checksum mismatch names both digests).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        ToolchainCandidate, ToolchainResult, ToolchainRole, ToolchainSelection, ToolchainSetParams,
    };

    #[test]
    fn roles_serialize_as_the_stable_key() {
        let value = serde_json::to_value(ToolchainRole::CxxCompiler).unwrap();
        assert_eq!(value, "cxxCompiler");
        assert_eq!(ToolchainRole::CxxCompiler.as_str(), "cxxCompiler");
        assert_eq!(ToolchainRole::all().len(), 7);

        // A chave e' CONTRATO: ela vai para o `.kinein/toolchain.json` e para o
        // wire. Renomear quebra o arquivo de quem ja escolheu.
        assert_eq!(
            serde_json::to_value(ToolchainRole::DebugAdapter).unwrap(),
            "debugAdapter"
        );
    }

    #[test]
    fn selection_omits_the_automatic_choice_and_the_missing_path() {
        let value = serde_json::to_value(ToolchainResult {
            preset: String::new(),
            sysroot: None,
            target_triple: None,
            chip: None,
            remote_target: None,
            debug_server: None,
            preset_toolchain_file: None,
            toolchain_file: None,
            selections: vec![ToolchainSelection {
                role: ToolchainRole::Cmake,
                id: None,
                resolved_path: None,
                effective_id: None,
                automatic: true,
            }],
            candidates: vec![ToolchainCandidate {
                role: ToolchainRole::Cmake,
                id: "cmake".to_owned(),
                label: "CMake".to_owned(),
                path: Some("/usr/bin/cmake".to_owned()),
                version: None,
            }],
            rust_targets: None,
            sysroot_hint: None,
        })
        .unwrap();

        assert!(value["selections"][0].get("id").is_none());
        assert!(value.get("rustTargets").is_none(), "sem rustup, sem campo");
        assert!(value.get("sysrootHint").is_none());
        assert!(value["selections"][0].get("resolvedPath").is_none());
        // `automatic` NAO e' omitido quando false por engano: ele diz "a
        // escolha nao e' sua", e a UI precisa dessa palavra mesmo quando ha
        // um id efetivo. Omiti-lo faria "sem informacao" parecer "escolhi eu".
        assert_eq!(value["selections"][0]["automatic"], true);
        assert_eq!(value["candidates"][0]["path"], "/usr/bin/cmake");
        assert!(value["candidates"][0].get("version").is_none());
    }

    #[test]
    fn set_params_accept_the_automatic_choice_and_reject_unknown_fields() {
        let automatico: ToolchainSetParams =
            serde_json::from_value(json!({ "role": "generator" })).unwrap();
        assert_eq!(automatico.role, ToolchainRole::Generator);
        assert_eq!(automatico.id, None);

        assert!(
            serde_json::from_value::<ToolchainSetParams>(json!({
                "role": "generator",
                "extra": 1,
            }))
            .is_err()
        );
    }
}
