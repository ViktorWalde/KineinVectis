//! Quem pode cumprir cada papel, e como esse papel vira argumento de comando.
//!
//! Tabela estatica, como o catalogo das Configuration Actions e pelo mesmo
//! motivo: nao ha registro dinamico neste projeto. Compilador novo e uma
//! entrada aqui — e uma entrada em [`crate::tools::KNOWN_TOOLS`], porque quem
//! DETECTA continua sendo o `ToolDetector`. Este modulo nao procura binario;
//! ele diz **quais** binarios interessam a cada papel e em que ordem.

use kinein_protocol::ToolchainRole;

/// Um candidato possivel para um papel.
pub(super) struct RoleCandidate {
    /// Id estavel; para compilador/ferramenta e o mesmo do `tools.detect`.
    pub(super) id: &'static str,
    /// Rotulo curto para a UI.
    pub(super) label: &'static str,
    /// Id em [`crate::tools::KNOWN_TOOLS`] que prova que ele existe aqui.
    ///
    /// O gerador tambem tem um: `Ninja` so aparece se o `ninja` existir, e
    /// `Unix Makefiles` so se o `make` existir. Oferecer um gerador que a
    /// maquina nao tem e' oferecer um configure que vai falhar.
    pub(super) tool_id: &'static str,
}

/// Candidatos de um papel, na ordem de preferencia.
///
/// A ordem importa: e' ela que a UI mostra, e o primeiro DETECTADO e o que a
/// dica de "automatico" descreve.
#[must_use]
pub(super) fn candidates_for(role: ToolchainRole) -> &'static [RoleCandidate] {
    match role {
        ToolchainRole::CCompiler => C_COMPILERS,
        ToolchainRole::CxxCompiler => CXX_COMPILERS,
        ToolchainRole::Generator => GENERATORS,
        ToolchainRole::Cmake => CMAKE,
        ToolchainRole::Cargo => CARGO,
    }
}

/// O candidato existe no catalogo do papel?
#[must_use]
pub(super) fn is_known(role: ToolchainRole, id: &str) -> bool {
    candidates_for(role).iter().any(|entry| entry.id == id)
}

static C_COMPILERS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "clang",
        label: "Clang",
        tool_id: "clang",
    },
    RoleCandidate {
        id: "gcc",
        label: "GCC",
        tool_id: "gcc",
    },
];

static CXX_COMPILERS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "clangxx",
        label: "Clang++",
        tool_id: "clangxx",
    },
    RoleCandidate {
        id: "gxx",
        label: "G++",
        tool_id: "gxx",
    },
];

static GENERATORS: &[RoleCandidate] = &[
    RoleCandidate {
        id: "Ninja",
        label: "Ninja",
        tool_id: "ninja",
    },
    RoleCandidate {
        id: "Unix Makefiles",
        label: "Unix Makefiles",
        tool_id: "make",
    },
];

static CMAKE: &[RoleCandidate] = &[RoleCandidate {
    id: "cmake",
    label: "CMake",
    tool_id: "cmake",
}];

static CARGO: &[RoleCandidate] = &[RoleCandidate {
    id: "cargo",
    label: "Cargo",
    tool_id: "cargo",
}];
