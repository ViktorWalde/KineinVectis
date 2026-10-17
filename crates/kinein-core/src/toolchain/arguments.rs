//! A toolchain traduzida em ARGUMENTOS para as ferramentas que rodam: o
//! `cmake` (compiladores fixados, sysroot, arquivo de toolchain, sistema e
//! processador do triple, o teste de compilador de bare metal), o `clangd`
//! (`--query-driver` do cross) e as binutils do cross (`arm-none-eabi-` de
//! `arm-none-eabi-gcc`). Saiu do `mod.rs` em 2026-09-13, quando o
//! `toolchainFile` do kit o levou a 512 linhas: resolver a escolha e
//! expressa-la em linha de comando sao responsabilidades diferentes.

use kinein_protocol::ToolchainRole;

use super::Toolchain;

impl Toolchain {
    /// O prefixo das binutils do cross, de `arm-none-eabi-gcc` -> `arm-none-eabi-`.
    ///
    /// De onde sai o `arm-none-eabi-size`, `arm-none-eabi-objcopy` etc. Vem do
    /// compilador C/C++ EFETIVO; `None` quando o compilador e' nativo, e ai o
    /// `size` do sistema serve. Regra: o id cross termina em `-gcc`/`-g++`, e o
    /// prefixo e' tudo ate' o ultimo `-` inclusive.
    #[must_use]
    pub fn binutils_prefix(&self) -> Option<String> {
        for role in [ToolchainRole::CCompiler, ToolchainRole::CxxCompiler] {
            let (id, _) = self.effective_program(role)?;
            for sufixo in ["-gcc", "-g++", "-gxx"] {
                if let Some(base) = id.strip_suffix(sufixo) {
                    return Some(format!("{base}-"));
                }
            }
        }
        None
    }

    /// Argumentos do `clangd` para ESTE kit.
    ///
    /// Sempre `--background-index`. E, quando o compilador C ou C++ efetivo e'
    /// um CROSS (`arm-none-eabi-gcc` e os que vierem), um `--query-driver` com
    /// o caminho resolvido dele.
    ///
    /// POR QUE (medido em 2026-09-11). Sem o `--query-driver`, o clangd 22 do
    /// Fedora acha `<stdint.h>` do cross sozinho, mas NAO os cabecalhos de
    /// `libstdc++` do GCC ARM (`<array>`, `<cstdint>` em
    /// `/usr/lib/gcc/arm-none-eabi/.../c++`): um `.cpp` de bare metal fica com
    /// 4 erros vermelhos. Com o driver na allowlist, o clangd pergunta ao GCC
    /// seus `-isystem` e os 4 somem. O clangd EXIGE a allowlist explicita por
    /// seguranca — rodar um driver arbitrario e' risco —, entao so' se passa o
    /// compilador que o proprio usuario escolheu, nunca um glob aberto.
    ///
    /// E `--clang-tidy` (P5 do 40 §4.1, 2026-09-17): o clangd 21 desta
    /// maquina lista a flag ("Enable clang-tidy diagnostics") e le o
    /// `.clang-tidy` do projeto sozinho — os avisos do tidy entram no mesmo
    /// canal dos diagnosticos, arquivo aberto a arquivo aberto, sem job. O
    /// projeto inteiro e' o `quality.run` (`build/tidy.rs`).
    #[must_use]
    pub fn clangd_args(&self) -> Vec<String> {
        // Ids nativos: o clangd ja' os entende sem ajuda. Qualquer outro e'
        // cross — a regra e' por EXCLUSAO para nao envelhecer a cada alvo novo.
        const NATIVOS: [&str; 4] = ["clang", "gcc", "clangxx", "gxx"];
        let mut args = vec!["--background-index".to_owned(), "--clang-tidy".to_owned()];
        let mut drivers: Vec<String> = Vec::new();
        for role in [ToolchainRole::CCompiler, ToolchainRole::CxxCompiler] {
            if let Some((id, path)) = self.effective_program(role)
                && !NATIVOS.contains(&id)
                && !drivers.iter().any(|d| d == path)
            {
                drivers.push(path.to_owned());
            }
        }
        if !drivers.is_empty() {
            args.push(format!("--query-driver={}", drivers.join(",")));
        }
        args
    }

    /// Argumentos de `cmake` que materializam a escolha: o `-G` e as
    /// [`Self::cmake_definitions`] como `-D`.
    #[must_use]
    pub fn cmake_arguments(&self) -> Vec<String> {
        let mut argumentos = Vec::new();
        if let Some(generator) = self.chosen(ToolchainRole::Generator) {
            argumentos.push("-G".to_owned());
            argumentos.push(generator.to_owned());
        }
        argumentos.extend(
            self.cmake_definitions()
                .into_iter()
                .map(|(name, value)| format!("-D{name}={value}")),
        );
        argumentos
    }

    /// As variaveis do `CMake` que o kit fixa, na ordem da linha. E' a fonte
    /// unica: a linha do configure e o toolchain file exportado (0.4.1a) saem
    /// daqui.
    ///
    /// So entra o que o usuario FIXOU. Emitir `-DCMAKE_CXX_COMPILER` com o que
    /// o `PATH` resolveria hoje congelaria uma escolha que o usuario nao fez —
    /// e um cache do `CMake` guarda compilador para sempre.
    #[must_use]
    pub fn cmake_definitions(&self) -> Vec<(&'static str, String)> {
        let mut definitions = Vec::new();
        for (role, variable) in [
            (ToolchainRole::CCompiler, "CMAKE_C_COMPILER"),
            (ToolchainRole::CxxCompiler, "CMAKE_CXX_COMPILER"),
        ] {
            if let Some(path) = self.program_for(role) {
                definitions.push((variable, path.display().to_string()));
            }
        }
        if let Some(sysroot) = self.sysroot.as_deref() {
            definitions.push(("CMAKE_SYSROOT", sysroot.to_owned()));
        }
        // O file de toolchain do KIT (o do Buildroot, o do SDK Yocto): so'
        // quando o preset nao declara o seu — o do preset e' do
        // CMakePresets.json e vence, como a tela ja' diz.
        if let (Some(file), None) = (
            self.kit_toolchain_file.as_deref(),
            self.preset_toolchain_file.as_deref(),
        ) {
            definitions.push(("CMAKE_TOOLCHAIN_FILE", file.to_owned()));
        }
        // Cross-compilacao: `CMAKE_SYSTEM_NAME` e' o que faz o CMake entrar em
        // modo cross (documentacao do CMake: defini-lo liga `CMAKE_CROSSCOMPILING`).
        // O nome do system sai do triple, que e a unica coisa que o usuario
        // digitou — inventar mais que isso seria adivinhar o alvo dele.
        if let Some(system) = self.cmake_system_name() {
            definitions.push(("CMAKE_SYSTEM_NAME", system.to_owned()));
            if let Some(processor) = self.cmake_system_processor() {
                definitions.push(("CMAKE_SYSTEM_PROCESSOR", processor.to_owned()));
            }
            // BARE METAL PRECISA DISTO, e sem ele o configure falha SEMPRE.
            //
            // Medido em 2026-09-03 exercitando `thumbv7em-none-eabihf` com o
            // `arm-none-eabi-gcc` real: o `CMAKE_SYSTEM_NAME=Generic` sozinho
            // nao basta. O `CMake` ainda tenta LINKAR um executavel no teste de
            // compilador, e bare metal nao tem os stubs do newlib —
            // `undefined reference to _exit`. O projeto do usuario nem chega a
            // ser configurado.
            //
            // A doc do `CMake` diz que `STATIC_LIBRARY` existe exatamente para
            // isto: "to avoid running the linker and is intended for use with
            // cross-compiling toolchains that cannot link without custom flags
            // or linker scripts".
            //
            // So entra em `Generic` — que e' como o `CMake` chama bare metal.
            // Em cross para Linux/Windows o link funciona, e forcar o teste a
            // virar biblioteca esconderia um toolchain de verdade quebrado.
            if system == "Generic" {
                definitions.push(("CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY".to_owned()));
            }
            definitions.extend(self.find_root_modes());
        }
        definitions
    }

    /// Com sysroot, o que os `find_*` acham vem DELE (0.4.1a, `roadmaps/52`
    /// §14.7). Medido em 2026-10-17 (`CMake` 4.2.3): com o padrao, o
    /// `find_package(ZLIB)` de um build com sysroot achou o `/usr/include` do
    /// host. Os programas continuam os do host (`NEVER`). Sem sysroot, nada:
    /// no multiarch do Debian os cabecalhos dos `:arm64` moram no `/usr/include`.
    /// Com toolchain file (do preset ou do kit), quem decide e' ele.
    fn find_root_modes(&self) -> Vec<(&'static str, String)> {
        if self.sysroot.is_none()
            || self.kit_toolchain_file.is_some()
            || self.preset_toolchain_file.is_some()
        {
            return Vec::new();
        }
        [
            ("CMAKE_FIND_ROOT_PATH_MODE_PROGRAM", "NEVER"),
            ("CMAKE_FIND_ROOT_PATH_MODE_LIBRARY", "ONLY"),
            ("CMAKE_FIND_ROOT_PATH_MODE_INCLUDE", "ONLY"),
            ("CMAKE_FIND_ROOT_PATH_MODE_PACKAGE", "ONLY"),
        ]
        .into_iter()
        .map(|(name, value)| (name, value.to_owned()))
        .collect()
    }

    /// `CMAKE_SYSTEM_NAME` derivado do triple (`<arch>-<vendor>-<os>-<abi>`).
    ///
    /// So' traduz o que e' inequivoco. Triple que este mapa nao conhece nao
    /// vira palpite: fica `None`, o `CMake` nao entra em modo cross e o usuario
    /// continua podendo usar um `toolchainFile` do preset, que e' o caminho
    /// oficial para alvos exoticos.
    fn cmake_system_name(&self) -> Option<&'static str> {
        let triple = self.target_triple.as_deref()?;
        let baixo = triple.to_ascii_lowercase();
        if baixo.contains("linux") {
            Some("Linux")
        } else if baixo.contains("windows") || baixo.contains("mingw") {
            Some("Windows")
        } else if baixo.contains("darwin") || baixo.contains("apple") {
            Some("Darwin")
        } else if baixo.contains("android") {
            Some("Android")
        } else if baixo.contains("none") || baixo.contains("-eabi") {
            // Bare metal: e' o que o CMake chama de Generic.
            Some("Generic")
        } else {
            None
        }
    }

    /// Arquitetura do triple: o primeiro campo, e so' ele.
    fn cmake_system_processor(&self) -> Option<&str> {
        self.target_triple
            .as_deref()?
            .split('-')
            .next()
            .filter(|arch| !arch.is_empty())
    }
}
