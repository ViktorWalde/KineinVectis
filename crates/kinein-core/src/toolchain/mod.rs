//! Toolchain: qual executavel cumpre cada papel neste workspace.
//!
//! # A pergunta que nao tinha dono
//!
//! Ate 2026-09-02 todo processo externo do core nascia de
//! `Command::new("<nome>")` — 28 chamadas, todas resolvidas pelo `PATH` do
//! processo. Trocar de compilador significava editar `CMakeLists.txt` a mao ou
//! exportar `CC`/`CXX` antes de abrir a IDE; o "kit" que `CLion` e Qt Creator
//! expoem nao existia (`roadmaps/29` §5d, B2 do TR2).
//!
//! Este dominio responde **qual binario**, e so isso. Ele nao executa nada, nao
//! detecta nada (quem detecta e o [`crate::tools::ToolDetector`], que ja era
//! agnostico) e nao decide POLITICA de build — ele traduz uma escolha do
//! usuario em caminho e em argumento de `cmake`.
//!
//! # O padrao continua sendo o PATH
//!
//! Escolha ausente = AUTOMATICO = exatamente o comportamento historico. Isso e'
//! deliberado: a fatia acrescenta uma capacidade, nao muda o que ja funcionava
//! na maquina de ninguem. Quem nunca abrir o seletor nao ve diferenca nenhuma.
//!
//! # Organizacao interna
//!
//! - [`catalog`]: quais binarios interessam a cada papel (tabela estatica);
//! - [`store`]: `.kinein/toolchain.json`, com `schemaVersion`.

mod catalog;
mod store;

use std::path::{Path, PathBuf};

use kinein_protocol::{
    ToolInfo, ToolStatus, ToolchainCandidate, ToolchainResult, ToolchainRole, ToolchainSelection,
};

/// A toolchain de um workspace, ja cruzada com o que existe na maquina.
#[derive(Debug, Clone)]
pub struct Toolchain {
    /// Kit lido; vazio = o padrao do workspace.
    preset: String,
    selections: Vec<ToolchainSelection>,
    candidates: Vec<ToolchainCandidate>,
    sysroot: Option<String>,
    target_triple: Option<String>,
    chip: Option<String>,
    remote_target: Option<String>,
    debug_server: Option<String>,
    preset_toolchain_file: Option<String>,
}

impl Toolchain {
    /// Le a escolha do workspace e cruza com as ferramentas detectadas.
    ///
    /// `tools` vem do `ToolDetector`; passar a lista (em vez de detectar aqui)
    /// mantem este modulo sem I/O de processo e deixa o teste hermetico.
    #[must_use]
    pub fn resolve(root: &Path, tools: &[ToolInfo]) -> Self {
        Self::resolve_kit(root, tools, "")
    }

    /// Idem, para um KIT especifico (nome do preset; `""` = o padrao).
    ///
    /// Um kit desconhecido nao e' erro: ele simplesmente ainda nao tem escolha,
    /// e tudo cai no `PATH` — o mesmo comportamento de um workspace novo.
    #[must_use]
    pub fn resolve_kit(root: &Path, tools: &[ToolInfo], preset: &str) -> Self {
        let kits = store::load(root);
        let kit = kits.get(preset).cloned().unwrap_or_default();
        let escolhas = kit.selections;
        let mut candidates = Vec::new();
        let mut selections = Vec::new();

        for role in ToolchainRole::all() {
            let disponiveis = detectados(*role, tools);
            let escolhido = escolhas
                .get(role.as_str())
                .filter(|id| catalog::is_known(*role, id))
                .cloned();

            let resolved_path = escolhido.as_ref().map_or_else(
                // Automatico: nada e fixado, e o `PATH` decide na hora de
                // executar. O caminho do primeiro detectado entra so como
                // INFORMACAO para a UI dizer o que vai acontecer.
                || disponiveis.first().and_then(|entry| entry.path.clone()),
                |id| {
                    disponiveis
                        .iter()
                        .find(|entry| entry.id == *id)
                        .and_then(|entry| entry.path.clone())
                },
            );

            // A MESCLA que o autor decidiu em 2026-09-04: sem escolha fixada,
            // o core PEGA o primeiro candidato detectado — a ordem do catalogo
            // ja' e' a preferencia — e marca que a escolha foi dele, nao do
            // autor. A IDE nao para esperando o menu ser aberto, e tambem nao
            // finge que o autor decidiu.
            let automatic = escolhido.is_none();
            let effective_id = escolhido
                .clone()
                .or_else(|| disponiveis.first().map(|entry| entry.id.clone()));

            selections.push(ToolchainSelection {
                role: *role,
                id: escolhido,
                resolved_path,
                effective_id,
                automatic,
            });
            candidates.extend(disponiveis);
        }

        Self {
            preset: preset.to_owned(),
            sysroot: kit.sysroot,
            target_triple: kit.target_triple,
            chip: kit.chip,
            remote_target: kit.remote_target,
            debug_server: kit.debug_server,
            preset_toolchain_file: preset_toolchain_file(root, preset),
            selections,
            candidates,
        }
    }

    /// O payload que os tres metodos `toolchain.*` respondem.
    #[must_use]
    pub fn to_result(&self) -> ToolchainResult {
        ToolchainResult {
            preset: self.preset.clone(),
            sysroot: self.sysroot.clone(),
            target_triple: self.target_triple.clone(),
            chip: self.chip.clone(),
            remote_target: self.remote_target.clone(),
            debug_server: self.debug_server.clone(),
            preset_toolchain_file: self.preset_toolchain_file.clone(),
            selections: self.selections.clone(),
            candidates: self.candidates.clone(),
        }
    }

    /// Id escolhido para um papel; `None` quando e automatico.
    #[must_use]
    pub fn chosen(&self, role: ToolchainRole) -> Option<&str> {
        self.selections
            .iter()
            .find(|selection| selection.role == role)
            .and_then(|selection| selection.id.as_deref())
    }

    /// Caminho a EXECUTAR para um papel, quando o usuario fixou um.
    ///
    /// `None` significa "use o nome do binario e deixe o `PATH` resolver" — e
    /// e' por isso que quem chama continua com `Command::new` no caso padrao,
    /// sem ramo novo por ferramenta.
    #[must_use]
    pub fn program_for(&self, role: ToolchainRole) -> Option<PathBuf> {
        let selection = self
            .selections
            .iter()
            .find(|selection| selection.role == role)?;
        selection.id.as_ref()?;
        selection.resolved_path.as_ref().map(PathBuf::from)
    }

    /// O caminho EFETIVO de um papel (o que sera' usado), fixado ou automatico.
    ///
    /// Difere de [`Self::program_for`], que so' devolve caminho quando o autor
    /// FIXOU: aqui vale tambem a escolha que o core fez sozinho, porque o
    /// clangd precisa saber o compilador que de fato roda, nao so' o que foi
    /// digitado no menu.
    fn effective_program(&self, role: ToolchainRole) -> Option<(&str, &str)> {
        let selection = self.selections.iter().find(|s| s.role == role)?;
        let id = selection.effective_id.as_deref()?;
        let path = selection.resolved_path.as_deref()?;
        Some((id, path))
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
    #[must_use]
    pub fn clangd_args(&self) -> Vec<String> {
        // Ids nativos: o clangd ja' os entende sem ajuda. Qualquer outro e'
        // cross — a regra e' por EXCLUSAO para nao envelhecer a cada alvo novo.
        const NATIVOS: [&str; 4] = ["clang", "gcc", "clangxx", "gxx"];
        let mut args = vec!["--background-index".to_owned()];
        let mut drivers: Vec<String> = Vec::new();
        for role in [ToolchainRole::CCompiler, ToolchainRole::CxxCompiler] {
            if let Some((id, path)) = self.effective_program(role) {
                if !NATIVOS.contains(&id) && !drivers.iter().any(|d| d == path) {
                    drivers.push(path.to_owned());
                }
            }
        }
        if !drivers.is_empty() {
            args.push(format!("--query-driver={}", drivers.join(",")));
        }
        args
    }

    /// Argumentos de `cmake` que materializam a escolha.
    ///
    /// So entra o que o usuario FIXOU. Emitir `-DCMAKE_CXX_COMPILER` com o que
    /// o `PATH` resolveria hoje congelaria uma escolha que o usuario nao fez —
    /// e um cache do `CMake` guarda compilador para sempre.
    #[must_use]
    pub fn cmake_arguments(&self) -> Vec<String> {
        let mut argumentos = Vec::new();
        if let Some(generator) = self.chosen(ToolchainRole::Generator) {
            argumentos.push("-G".to_owned());
            argumentos.push(generator.to_owned());
        }
        for (role, variavel) in [
            (ToolchainRole::CCompiler, "CMAKE_C_COMPILER"),
            (ToolchainRole::CxxCompiler, "CMAKE_CXX_COMPILER"),
        ] {
            if let Some(caminho) = self.program_for(role) {
                argumentos.push(format!("-D{variavel}={}", caminho.display()));
            }
        }
        if let Some(sysroot) = self.sysroot.as_deref() {
            argumentos.push(format!("-DCMAKE_SYSROOT={sysroot}"));
        }
        // Cross-compilacao: `CMAKE_SYSTEM_NAME` e' o que faz o CMake entrar em
        // modo cross (documentacao do CMake: defini-lo liga `CMAKE_CROSSCOMPILING`).
        // O nome do sistema sai do triple, que e a unica coisa que o usuario
        // digitou — inventar mais que isso seria adivinhar o alvo dele.
        if let Some(sistema) = self.cmake_system_name() {
            argumentos.push(format!("-DCMAKE_SYSTEM_NAME={sistema}"));
            if let Some(processador) = self.cmake_system_processor() {
                argumentos.push(format!("-DCMAKE_SYSTEM_PROCESSOR={processador}"));
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
            if sistema == "Generic" {
                argumentos.push("-DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY".to_owned());
            }
        }
        argumentos
    }

    /// Triple do alvo, quando escolhido — o `--target` do cargo.
    #[must_use]
    pub fn target_triple(&self) -> Option<&str> {
        self.target_triple.as_deref()
    }

    /// Raiz do sistema alvo, quando escolhida.
    #[must_use]
    pub fn sysroot(&self) -> Option<&str> {
        self.sysroot.as_deref()
    }

    /// Chip do alvo, quando escolhido — vai no `launch` do DAP.
    #[must_use]
    pub fn chip(&self) -> Option<&str> {
        self.chip.as_deref()
    }

    /// `host:porta` do servidor GDB, quando o kit e' remoto — vai no `attach`.
    #[must_use]
    pub fn remote_target(&self) -> Option<&str> {
        self.remote_target.as_deref()
    }

    /// Comando do servidor que a IDE sobe antes de conectar, quando declarado.
    #[must_use]
    pub fn debug_server(&self) -> Option<&str> {
        self.debug_server.as_deref()
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

/// Fixa (ou libera) a escolha de um papel e devolve a toolchain resultante.
pub fn set(
    root: &Path,
    tools: &[ToolInfo],
    role: ToolchainRole,
    id: Option<&str>,
    preset: &str,
) -> Result<Toolchain, String> {
    if let Some(id) = id {
        if !catalog::is_known(role, id) {
            return Err(format!(
                "{id} nao e um candidato conhecido para {}",
                role.as_str()
            ));
        }
        if !detectados(role, tools).iter().any(|entry| entry.id == id) {
            return Err(format!(
                "{id} nao foi detectado nesta maquina; instale-o ou escolha outro"
            ));
        }
    }

    let mut kits = store::load(root);
    let kit = kits.entry(preset.to_owned()).or_default();
    match id {
        Some(id) => {
            kit.selections
                .insert(role.as_str().to_owned(), id.to_owned());
        }
        None => {
            kit.selections.remove(role.as_str());
        }
    }
    store::save(root, &kits)?;
    Ok(Toolchain::resolve_kit(root, tools, preset))
}

/// Os campos do kit que `toolchain.setKit` pode mudar. `Some("")` LIMPA;
/// `None` preserva — um por campo, para mexer num nao apagar os outros.
#[derive(Debug, Clone, Copy, Default)]
pub struct KitUpdate<'a> {
    /// Raiz do sistema alvo (`CMAKE_SYSROOT`).
    pub sysroot: Option<&'a str>,
    /// Triple do alvo (`--target` do cargo, `CMAKE_SYSTEM_*`).
    pub target_triple: Option<&'a str>,
    /// Chip do alvo, para o `launch` do adaptador de embarcado.
    pub chip: Option<&'a str>,
    /// `host:porta` do servidor GDB (`target remote`).
    pub remote_target: Option<&'a str>,
    /// Comando do servidor que a IDE sobe antes de conectar.
    pub debug_server: Option<&'a str>,
}

/// Fixa os campos do kit que vieram em `update`.
pub fn set_kit(
    root: &Path,
    tools: &[ToolInfo],
    preset: &str,
    update: KitUpdate<'_>,
) -> Result<Toolchain, String> {
    let mut kits = store::load(root);
    let kit = kits.entry(preset.to_owned()).or_default();
    // Aparado, e vazio vira ausente: e' o contrato de "string vazia limpa".
    let aplica = |campo: &mut Option<String>, valor: Option<&str>| {
        if let Some(valor) = valor {
            let limpo = valor.trim();
            *campo = (!limpo.is_empty()).then(|| limpo.to_owned());
        }
    };
    aplica(&mut kit.sysroot, update.sysroot);
    aplica(&mut kit.target_triple, update.target_triple);
    aplica(&mut kit.chip, update.chip);
    aplica(&mut kit.remote_target, update.remote_target);
    aplica(&mut kit.debug_server, update.debug_server);
    store::save(root, &kits)?;
    Ok(Toolchain::resolve_kit(root, tools, preset))
}

/// O `toolchainFile` que o preset declara, se declarar.
///
/// Le o `CMakePresets.json`/`CMakeUserPresets.json` cru: e' informacao para a
/// tela, nao entrada de decisao. O campo existe no schema de presets desde a
/// versao 3 (`CMake` 3.21) e tem precedencia sobre `CMAKE_TOOLCHAIN_FILE`, entao
/// quando ele esta la' o compilador efetivo pode NAO ser o que o usuario
/// escolheu aqui — e a IDE precisa dizer isso em vez de deixar procurar.
fn preset_toolchain_file(root: &Path, preset: &str) -> Option<String> {
    if preset.is_empty() {
        return None;
    }
    for arquivo in ["CMakePresets.json", "CMakeUserPresets.json"] {
        let Ok(body) = std::fs::read_to_string(root.join(arquivo)) else {
            continue;
        };
        let Ok(valor) = serde_json::from_str::<serde_json::Value>(&body) else {
            continue;
        };
        let encontrado = valor
            .get("configurePresets")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
            .find(|entry| entry.get("name").and_then(serde_json::Value::as_str) == Some(preset))
            .and_then(|entry| entry.get("toolchainFile"))
            .and_then(serde_json::Value::as_str);
        if let Some(caminho) = encontrado {
            return Some(caminho.to_owned());
        }
    }
    None
}

/// Candidatos de um papel que existem nesta maquina, na ordem do catalogo.
fn detectados(role: ToolchainRole, tools: &[ToolInfo]) -> Vec<ToolchainCandidate> {
    catalog::candidates_for(role)
        .iter()
        .filter_map(|entry| {
            let detectado = tools.iter().find(|tool| {
                tool.id == entry.tool_id && matches!(tool.status, ToolStatus::Detected)
            })?;
            Some(ToolchainCandidate {
                role,
                id: entry.id.to_owned(),
                label: entry.label.to_owned(),
                path: detectado.path.clone(),
                version: detectado.version.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use kinein_protocol::{ToolInfo, ToolStatus, ToolchainRole};

    use super::{KitUpdate, Toolchain, set, set_kit};

    fn detectado(id: &str, caminho: &str) -> ToolInfo {
        ToolInfo {
            id: id.to_owned(),
            display_name: id.to_owned(),
            status: ToolStatus::Detected,
            path: Some(caminho.to_owned()),
            version: Some("1.0".to_owned()),
            suggested_install: None,
            message: None,
        }
    }

    fn ausente(id: &str) -> ToolInfo {
        ToolInfo {
            id: id.to_owned(),
            display_name: id.to_owned(),
            status: ToolStatus::Missing,
            path: None,
            version: None,
            suggested_install: None,
            message: None,
        }
    }

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("kinein-toolchain")
            .join(format!("{}-{name}", std::process::id()));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).unwrap();
        }
        std::fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    /// Uma maquina que TEM o cross ARM alem dos nativos.
    fn maquina_com_cross() -> Vec<ToolInfo> {
        let mut tools = maquina();
        tools.push(detectado("arm-none-eabi-gcc", "/usr/bin/arm-none-eabi-gcc"));
        tools.push(detectado("arm-none-eabi-gxx", "/usr/bin/arm-none-eabi-g++"));
        tools
    }

    /// Sem cross, o clangd so' ganha o `--background-index` — nada de
    /// `--query-driver`, que o clangd 22 nem precisa para os nativos.
    #[test]
    fn clangd_args_nativo_nao_tem_query_driver() {
        let root = temp_root("clangd-nativo");
        let tc = Toolchain::resolve(&root, &maquina());
        let args = tc.clangd_args();
        assert_eq!(args, vec!["--background-index".to_owned()]);
    }

    /// Com o compilador C++ fixado no cross, o clangd ganha `--query-driver`
    /// apontando o caminho RESOLVIDO dele. E' o que faz `<array>`/`<cstdint>`
    /// do libstdc++ ARM pararem de ficar vermelhos (medido, roadmaps/35 §5.7).
    #[test]
    fn clangd_args_cross_ganha_query_driver_com_o_caminho_resolvido() {
        let root = temp_root("clangd-cross");
        set(
            &root,
            &maquina_com_cross(),
            ToolchainRole::CxxCompiler,
            Some("arm-none-eabi-gxx"),
            "",
        )
        .unwrap();
        let args = Toolchain::resolve(&root, &maquina_com_cross()).clangd_args();
        assert!(args.contains(&"--background-index".to_owned()), "{args:?}");
        assert!(
            args.iter()
                .any(|a| a == "--query-driver=/usr/bin/arm-none-eabi-g++"),
            "o clangd nao recebeu o driver cross: {args:?}"
        );
    }

    /// C e C++ cross ao mesmo tempo: os dois caminhos entram, sem repetir, numa
    /// lista unica separada por virgula — que e' a forma que o clangd aceita.
    #[test]
    fn clangd_args_junta_c_e_cxx_cross_sem_repetir() {
        let root = temp_root("clangd-cross-dois");
        set(
            &root,
            &maquina_com_cross(),
            ToolchainRole::CCompiler,
            Some("arm-none-eabi-gcc"),
            "",
        )
        .unwrap();
        set(
            &root,
            &maquina_com_cross(),
            ToolchainRole::CxxCompiler,
            Some("arm-none-eabi-gxx"),
            "",
        )
        .unwrap();
        let args = Toolchain::resolve(&root, &maquina_com_cross()).clangd_args();
        let driver: Vec<&String> = args
            .iter()
            .filter(|a| a.starts_with("--query-driver="))
            .collect();
        assert_eq!(driver.len(), 1, "mais de um --query-driver: {args:?}");
        assert_eq!(
            driver[0],
            "--query-driver=/usr/bin/arm-none-eabi-gcc,/usr/bin/arm-none-eabi-g++"
        );
    }

    fn maquina() -> Vec<ToolInfo> {
        vec![
            detectado("clang", "/usr/bin/clang"),
            detectado("clangxx", "/usr/bin/clang++"),
            detectado("gcc", "/usr/bin/gcc"),
            detectado("gxx", "/usr/bin/g++"),
            detectado("ninja", "/usr/bin/ninja"),
            detectado("cmake", "/usr/bin/cmake"),
            detectado("cargo", "/usr/bin/cargo"),
            ausente("make"),
        ]
    }

    /// Sem escolha, NADA e' fixado: o `PATH` continua decidindo.
    #[test]
    fn automatic_pins_nothing_on_the_command_line() {
        let root = temp_root("automatico");
        let toolchain = Toolchain::resolve(&root, &maquina());

        assert!(toolchain.cmake_arguments().is_empty());
        assert_eq!(toolchain.program_for(ToolchainRole::CxxCompiler), None);
        assert_eq!(toolchain.chosen(ToolchainRole::CxxCompiler), None);
    }

    /// A escolha vira argumento de `cmake` — e' o que a torna real.
    #[test]
    fn a_choice_becomes_a_cmake_argument() {
        let root = temp_root("escolha");
        let escolhida = set(
            &root,
            &maquina(),
            ToolchainRole::CxxCompiler,
            Some("gxx"),
            "",
        )
        .unwrap();
        let com_gerador = set(
            &root,
            &maquina(),
            ToolchainRole::Generator,
            Some("Ninja"),
            "",
        )
        .unwrap();

        assert_eq!(
            escolhida.program_for(ToolchainRole::CxxCompiler),
            Some(PathBuf::from("/usr/bin/g++"))
        );
        assert_eq!(
            com_gerador.cmake_arguments(),
            vec![
                "-G".to_owned(),
                "Ninja".to_owned(),
                "-DCMAKE_CXX_COMPILER=/usr/bin/g++".to_owned(),
            ]
        );
    }

    /// A escolha sobrevive a releitura: e' estado do workspace, nao da sessao.
    #[test]
    fn the_choice_survives_a_reload_and_can_be_released() {
        let root = temp_root("persiste");
        set(&root, &maquina(), ToolchainRole::CCompiler, Some("gcc"), "").unwrap();

        let relido = Toolchain::resolve(&root, &maquina());
        assert_eq!(relido.chosen(ToolchainRole::CCompiler), Some("gcc"));

        let liberado = set(&root, &maquina(), ToolchainRole::CCompiler, None, "").unwrap();
        assert_eq!(liberado.chosen(ToolchainRole::CCompiler), None);
        assert!(liberado.cmake_arguments().is_empty());
    }

    /// So se oferece o que EXISTE: gerador sem `make` nao entra na lista.
    #[test]
    fn only_detected_candidates_are_offered_or_accepted() {
        let root = temp_root("detectado");
        let toolchain = Toolchain::resolve(&root, &maquina());
        let resultado = toolchain.to_result();
        let geradores: Vec<&str> = resultado
            .candidates
            .iter()
            .filter(|candidato| candidato.role == ToolchainRole::Generator)
            .map(|candidato| candidato.id.as_str())
            .collect();

        assert_eq!(geradores, ["Ninja"], "sem `make`, sem Unix Makefiles");
        assert!(
            set(
                &root,
                &maquina(),
                ToolchainRole::Generator,
                Some("Unix Makefiles"),
                ""
            )
            .is_err()
        );
        assert!(set(&root, &maquina(), ToolchainRole::CCompiler, Some("tcc"), "").is_err());
    }

    /// A escolha some da maquina: a IDE nao pode fixar um caminho inexistente.
    #[test]
    fn a_choice_whose_tool_vanished_stops_pinning_anything() {
        let root = temp_root("sumiu");
        set(
            &root,
            &maquina(),
            ToolchainRole::CxxCompiler,
            Some("gxx"),
            "",
        )
        .unwrap();

        let sem_gxx: Vec<ToolInfo> = maquina()
            .into_iter()
            .map(|tool| {
                if tool.id == "gxx" {
                    ausente("gxx")
                } else {
                    tool
                }
            })
            .collect();
        let toolchain = Toolchain::resolve(&root, &sem_gxx);

        assert_eq!(toolchain.chosen(ToolchainRole::CxxCompiler), Some("gxx"));
        assert_eq!(toolchain.program_for(ToolchainRole::CxxCompiler), None);
        assert!(
            toolchain.cmake_arguments().is_empty(),
            "sem binario, nada de -DCMAKE_CXX_COMPILER apontando para o vazio"
        );
    }

    /// Sysroot e triple viram argumento de `cmake` e `--target` do cargo.
    #[test]
    fn sysroot_and_target_become_build_arguments() {
        let root = temp_root("cross");
        let kit = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: Some("/opt/sysroots/arm"),
                target_triple: Some("aarch64-unknown-linux-gnu"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();

        let argumentos = kit.cmake_arguments();
        assert!(
            argumentos.contains(&"-DCMAKE_SYSROOT=/opt/sysroots/arm".to_owned()),
            "sysroot nao virou argumento: {argumentos:?}"
        );
        // `CMAKE_SYSTEM_NAME` e o que liga o modo cross do CMake; sem ele o
        // sysroot sozinho nao muda a decisao de compilador.
        assert!(argumentos.contains(&"-DCMAKE_SYSTEM_NAME=Linux".to_owned()));
        assert!(argumentos.contains(&"-DCMAKE_SYSTEM_PROCESSOR=aarch64".to_owned()));
        assert_eq!(kit.target_triple(), Some("aarch64-unknown-linux-gnu"));
    }

    /// Triple que o mapa nao conhece NAO vira palpite de `CMAKE_SYSTEM_NAME`.
    #[test]
    fn unknown_triple_does_not_guess_a_system_name() {
        let root = temp_root("exotico");
        let kit = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: None,
                target_triple: Some("riscv64-esquisito-xyz"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();

        let argumentos = kit.cmake_arguments();
        assert!(
            !argumentos
                .iter()
                .any(|a| a.starts_with("-DCMAKE_SYSTEM_NAME")),
            "adivinhou o sistema de um triple desconhecido: {argumentos:?}"
        );
        // O triple continua valendo para o cargo, que sabe o que fazer com ele.
        assert_eq!(kit.target_triple(), Some("riscv64-esquisito-xyz"));
    }

    /// Kits sao independentes: mexer num nao mexe no outro.
    #[test]
    fn kits_do_not_leak_into_each_other() {
        let root = temp_root("kits-isolados");
        set_kit(
            &root,
            &maquina(),
            "cross",
            KitUpdate {
                sysroot: Some("/opt/arm"),
                target_triple: Some("armv7-unknown-linux-gnueabihf"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();
        set(
            &root,
            &maquina(),
            ToolchainRole::CxxCompiler,
            Some("gxx"),
            "",
        )
        .unwrap();

        let padrao = Toolchain::resolve_kit(&root, &maquina(), "");
        assert_eq!(
            padrao.sysroot(),
            None,
            "o sysroot do cross vazou para o padrao"
        );
        assert_eq!(padrao.chosen(ToolchainRole::CxxCompiler), Some("gxx"));

        let cross = Toolchain::resolve_kit(&root, &maquina(), "cross");
        assert_eq!(cross.sysroot(), Some("/opt/arm"));
        assert_eq!(
            cross.chosen(ToolchainRole::CxxCompiler),
            None,
            "a escolha do kit padrao vazou para o cross"
        );
    }

    /// `Some("")` LIMPA; `None` preserva. Sem isso, mexer no sysroot apagaria
    /// o alvo — e o usuario so' descobriria no proximo build.
    #[test]
    fn absent_field_preserves_and_empty_clears() {
        let root = temp_root("preserva");
        set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: Some("/opt/a"),
                target_triple: Some("x86_64-unknown-linux-gnu"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();

        let so_sysroot = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: Some("/opt/b"),
                ..KitUpdate::default()
            },
        )
        .unwrap();
        assert_eq!(so_sysroot.sysroot(), Some("/opt/b"));
        assert_eq!(
            so_sysroot.target_triple(),
            Some("x86_64-unknown-linux-gnu"),
            "campo ausente apagou o alvo"
        );

        let limpo = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: Some("  "),
                ..KitUpdate::default()
            },
        )
        .unwrap();
        assert_eq!(limpo.sysroot(), None, "string em branco tinha que limpar");
    }

    /// Bare metal ganha `CMAKE_TRY_COMPILE_TARGET_TYPE`; cross "com sistema
    /// operacional" NAO. Medido exercitando o `arm-none-eabi-gcc` de verdade.
    #[test]
    fn bare_metal_skips_the_linker_in_the_compiler_test() {
        let root = temp_root("baremetal");
        let bare = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: None,
                target_triple: Some("thumbv7em-none-eabihf"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();
        let argumentos = bare.cmake_arguments();
        assert!(
            argumentos.contains(&"-DCMAKE_SYSTEM_NAME=Generic".to_owned()),
            "{argumentos:?}"
        );
        assert!(
            argumentos.contains(&"-DCMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY".to_owned()),
            "sem isto o configure de bare metal falha em `undefined reference to _exit`: \
             {argumentos:?}"
        );

        // Cross para Linux LINKA normalmente. Forcar biblioteca esconderia um
        // toolchain quebrado de verdade.
        let linux = set_kit(
            &root,
            &maquina(),
            "",
            KitUpdate {
                sysroot: None,
                target_triple: Some("aarch64-unknown-linux-gnu"),
                chip: None,
                ..KitUpdate::default()
            },
        )
        .unwrap();
        assert!(
            !linux
                .cmake_arguments()
                .iter()
                .any(|a| a.contains("TRY_COMPILE_TARGET_TYPE")),
            "cross com SO nao deve pular o linker"
        );
    }
}
