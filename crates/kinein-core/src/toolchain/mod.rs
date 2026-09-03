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
    selections: Vec<ToolchainSelection>,
    candidates: Vec<ToolchainCandidate>,
}

impl Toolchain {
    /// Le a escolha do workspace e cruza com as ferramentas detectadas.
    ///
    /// `tools` vem do `ToolDetector`; passar a lista (em vez de detectar aqui)
    /// mantem este modulo sem I/O de processo e deixa o teste hermetico.
    #[must_use]
    pub fn resolve(root: &Path, tools: &[ToolInfo]) -> Self {
        let escolhas = store::load(root);
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
            selections,
            candidates,
        }
    }

    /// O payload que os tres metodos `toolchain.*` respondem.
    #[must_use]
    pub fn to_result(&self) -> ToolchainResult {
        ToolchainResult {
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
        argumentos
    }
}

/// Fixa (ou libera) a escolha de um papel e devolve a toolchain resultante.
pub fn set(
    root: &Path,
    tools: &[ToolInfo],
    role: ToolchainRole,
    id: Option<&str>,
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

    let mut escolhas = store::load(root);
    match id {
        Some(id) => {
            escolhas.insert(role.as_str().to_owned(), id.to_owned());
        }
        None => {
            escolhas.remove(role.as_str());
        }
    }
    store::save(root, &escolhas)?;
    Ok(Toolchain::resolve(root, tools))
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

    use super::{Toolchain, set};

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
        let escolhida = set(&root, &maquina(), ToolchainRole::CxxCompiler, Some("gxx")).unwrap();
        let com_gerador = set(&root, &maquina(), ToolchainRole::Generator, Some("Ninja")).unwrap();

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
        set(&root, &maquina(), ToolchainRole::CCompiler, Some("gcc")).unwrap();

        let relido = Toolchain::resolve(&root, &maquina());
        assert_eq!(relido.chosen(ToolchainRole::CCompiler), Some("gcc"));

        let liberado = set(&root, &maquina(), ToolchainRole::CCompiler, None).unwrap();
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
                Some("Unix Makefiles")
            )
            .is_err()
        );
        assert!(set(&root, &maquina(), ToolchainRole::CCompiler, Some("tcc")).is_err());
    }

    /// A escolha some da maquina: a IDE nao pode fixar um caminho inexistente.
    #[test]
    fn a_choice_whose_tool_vanished_stops_pinning_anything() {
        let root = temp_root("sumiu");
        set(&root, &maquina(), ToolchainRole::CxxCompiler, Some("gxx")).unwrap();

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
}
