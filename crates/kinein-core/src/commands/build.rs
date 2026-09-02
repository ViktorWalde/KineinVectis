//! Comandos de configuracao e construcao: build, `CMake`, Cargo, jobs e as
//! Configuration Actions.

use kinein_protocol::CommandDescriptor;

pub(super) fn build_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "build.run".to_owned(),
            title: "Build Project".to_owned(),
            category: "Build".to_owned(),
            description: "Compila o projeto do workspace e emite erros estruturados".to_owned(),
            default_shortcut: Some("Ctrl+F9".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "test.run".to_owned(),
            title: "Run Tests".to_owned(),
            category: "Build".to_owned(),
            description: "Roda os testes do projeto (cargo test / ctest) com resultado por caso"
                .to_owned(),
            default_shortcut: Some("Ctrl+Shift+F9".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "quality.run".to_owned(),
            title: "Analyze (Lint)".to_owned(),
            category: "Build".to_owned(),
            description: "Roda a analise de qualidade (cargo clippy) e lista os avisos".to_owned(),
            default_shortcut: Some("Ctrl+Shift+L".to_owned()),
            requires_workspace: true,
        },
    ]
}

pub(super) fn cmake_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "cmake.configure".to_owned(),
            title: "CMake: Configure".to_owned(),
            category: "CMake".to_owned(),
            description: "Configura o projeto CMake em .kinein/build (gera compile_commands.json)"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "cmake.presets.list".to_owned(),
            title: "CMake: Presets".to_owned(),
            category: "CMake".to_owned(),
            description: "Lista os configure presets do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "cmake.targets.list".to_owned(),
            title: "CMake: Targets".to_owned(),
            category: "CMake".to_owned(),
            description: "Lista os targets do ultimo configure (file-api)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "cmake.status".to_owned(),
            title: "CMake: Status".to_owned(),
            category: "CMake".to_owned(),
            description: "Mostra se o projeto esta configurado e se ha compile_commands".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

pub(super) fn cargo_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "cargo.check".to_owned(),
            title: "Cargo: Check".to_owned(),
            category: "Cargo".to_owned(),
            description:
                "Roda cargo check (feedback rapido sem codegen); problemas na aba Problemas"
                    .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "cargo.metadata".to_owned(),
            title: "Cargo: Metadata".to_owned(),
            category: "Cargo".to_owned(),
            description: "Resumo do workspace Cargo (pacotes, targets, features)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

pub(super) fn jobs_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "job.list".to_owned(),
            title: "Jobs".to_owned(),
            category: "Jobs".to_owned(),
            description: "Lista os jobs (operacoes longas) conhecidos pelo core".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "job.cancel".to_owned(),
            title: "Cancel Job".to_owned(),
            category: "Jobs".to_owned(),
            description: "Sinaliza um job em execucao para cancelar".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
    ]
}

/// Configuration Actions (spec 9.1/9.2): a caixa de ferramentas contextual.
///
/// UM descriptor, nao dezesseis. A paleta anuncia a ENTRADA do painel; a lista
/// de acoes de verdade vem do `configAction.list`, que ja e filtrada pelo build
/// system ativo (spec 9.2 §25: a paleta respeita o escopo). Anunciar as 16 aqui
/// duplicaria o catalogo em dois lugares que envelheceriam separados.
pub(super) fn configaction_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "configAction.list".to_owned(),
        title: "Configuration Actions...".to_owned(),
        category: "Projeto".to_owned(),
        description: "Acoes de configuracao do projeto (CMake/Cargo) com preview e diff".to_owned(),
        default_shortcut: Some("Ctrl+Alt+P".to_owned()),
        requires_workspace: true,
    }]
}
