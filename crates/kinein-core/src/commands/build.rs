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

/// Como instalar o que falta (2026-09-04): o passo a passo oficial.
///
/// `requires_workspace` e' FALSE, e a ausencia diz algo: quem esta comecando
/// abre a IDE antes de ter projeto, e e' exatamente ai' que este guia serve.
pub(super) fn setup_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "setup.list".to_owned(),
        title: "Instalar ferramentas...".to_owned(),
        category: "Projeto".to_owned(),
        description: "Passo a passo oficial para a sua distribuicao, com a fonte junto".to_owned(),
        default_shortcut: Some("Ctrl+Alt+H".to_owned()),
        requires_workspace: false,
    }]
}

/// Fontes de dados (roadmaps/35, etapa 26): o catalogo de conexoes.
///
/// UM descriptor, pelo mesmo motivo do `library` abaixo: a paleta anuncia a
/// ENTRADA do painel, e os perfis vem do `datasource.list`.
///
/// `requires_workspace` e' true porque o catalogo e' POR PROJETO — os perfis
/// vivem em `.kinein/datasources.json`. Sem projeto aberto nao ha' onde
/// guarda-los, e oferecer a acao seria oferecer um caminho que termina em nada.
pub(super) fn datasource_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "datasource.list".to_owned(),
        title: "Banco de dados...".to_owned(),
        category: "Projeto".to_owned(),
        description: "Conexoes a banco: o perfil fica salvo, a senha nunca".to_owned(),
        // Ctrl+Alt+J: o Ctrl+Alt+D que este comando anunciava ao nascer ja'
        // era um alias do `debug.start` na UI, entao o atalho da paleta
        // DEPURAVA em vez de abrir as fontes de dados.
        default_shortcut: Some("Ctrl+Alt+J".to_owned()),
        requires_workspace: true,
    }]
}

/// Bibliotecas C/C++ (roadmaps/35): o catalogo curado.
///
/// UM descriptor, nao treze — mesma razao do `configaction` acima. A paleta
/// anuncia a ENTRADA do painel; a lista de bibliotecas vem do `library.list`.
/// Anunciar as treze aqui duplicaria o catalogo em dois lugares que
/// envelheceriam separados, e o catalogo ja e' divida por ENTRADA.
///
/// `requires_workspace` e' true embora o dominio `library` seja stateless: sem
/// projeto aberto nao ha alvo de `CMake` para linkar, e oferecer a acao seria
/// oferecer um caminho que termina em nada.
pub(super) fn library_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "library.list".to_owned(),
        title: "Bibliotecas...".to_owned(),
        category: "Projeto".to_owned(),
        description: "Catalogo auditado: licenca, versao fixada e o que cada uma faz".to_owned(),
        // Ctrl+Alt+K, e NAO Ctrl+Alt+L: ate 2026-09-04 este comando anunciava
        // Ctrl+Alt+L, que o `format.text` tambem anunciava e que a UI liga em
        // FORMATAR. Quem apertava o atalho da paleta formatava o arquivo. Foi
        // relato de uso do autor que achou; o 17o gate impede a volta.
        default_shortcut: Some("Ctrl+Alt+K".to_owned()),
        requires_workspace: true,
    }]
}
