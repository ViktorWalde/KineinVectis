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

            selections.push(ToolchainSelection {
                role: *role,
                id: escolhido,
                resolved_path,
            });
            candidates.extend(disponiveis);
        }

        Self {
            preset: preset.to_owned(),
            sysroot: kit.sysroot,
            target_triple: kit.target_triple,
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

/// Fixa sysroot e/ou triple do kit. `Some("")` LIMPA; `None` preserva.
pub fn set_kit(
    root: &Path,
    tools: &[ToolInfo],
    preset: &str,
    sysroot: Option<&str>,
    target_triple: Option<&str>,
) -> Result<Toolchain, String> {
    let mut kits = store::load(root);
    let kit = kits.entry(preset.to_owned()).or_default();
    if let Some(valor) = sysroot {
        let limpo = valor.trim();
        kit.sysroot = (!limpo.is_empty()).then(|| limpo.to_owned());
    }
    if let Some(valor) = target_triple {
        let limpo = valor.trim();
        kit.target_triple = (!limpo.is_empty()).then(|| limpo.to_owned());
    }
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

    use super::{Toolchain, set, set_kit};

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
            Some("/opt/sysroots/arm"),
            Some("aarch64-unknown-linux-gnu"),
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
        let kit = set_kit(&root, &maquina(), "", None, Some("riscv64-esquisito-xyz")).unwrap();

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
            Some("/opt/arm"),
            Some("armv7-unknown-linux-gnueabihf"),
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
            Some("/opt/a"),
            Some("x86_64-unknown-linux-gnu"),
        )
        .unwrap();

        let so_sysroot = set_kit(&root, &maquina(), "", Some("/opt/b"), None).unwrap();
        assert_eq!(so_sysroot.sysroot(), Some("/opt/b"));
        assert_eq!(
            so_sysroot.target_triple(),
            Some("x86_64-unknown-linux-gnu"),
            "campo ausente apagou o alvo"
        );

        let limpo = set_kit(&root, &maquina(), "", Some("  "), None).unwrap();
        assert_eq!(limpo.sysroot(), None, "string em branco tinha que limpar");
    }
}
