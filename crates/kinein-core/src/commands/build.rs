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
            description:
                "Roda a analise de qualidade (cargo clippy / ruff / clang-tidy) e lista os avisos"
                    .to_owned(),
            default_shortcut: Some("Ctrl+Shift+L".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "coverage.run".to_owned(),
            title: "Cobertura dos testes".to_owned(),
            category: "Build".to_owned(),
            description: "Roda os testes com cobertura (cargo llvm-cov / coverage.py) e pinta a \
                          calha do editor"
                .to_owned(),
            default_shortcut: None,
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

/// O alvo Linux por SSH (P6 fatia 1 do roadmaps/42, 2026-09-17): a Pi, a
/// placa com imagem propria.
///
/// UM descriptor, como o `datasource`: a paleta anuncia a ENTRADA do painel;
/// sondar, enviar e compor comandos sao gestos DENTRO dele. POR PROJETO
/// (`.kinein/remotes.json`), logo `requires_workspace`. Sem atalho: os
/// Ctrl+Alt+<letra> livres ja' sao poucos, e o painel nasce sem uso medido.
pub(super) fn remote_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "remote.list".to_owned(),
        title: "Alvo remoto (SSH)...".to_owned(),
        category: "Projeto".to_owned(),
        description: "Linux embarcado por SSH: sondar, enviar, rodar e depurar; sem senha em disco"
            .to_owned(),
        default_shortcut: None,
        requires_workspace: true,
    }]
}

/// Observabilidade (roadmaps/35, etapa 27): o Grafana deste projeto.
///
/// UM descriptor, e o id e' `grafana.get` porque e' o metodo que a acao chama:
/// o painel abre lendo a instancia salva. `grafana.probe` NAO entra na paleta —
/// sondar e' o que se faz DENTRO do painel, com o token da sessao em maos.
///
/// `requires_workspace` e' true porque a instancia e' POR PROJETO: ela vive em
/// `.kinein/grafana.json`, e a pergunta que o painel responde — *"o Grafana ja'
/// observa os bancos deste projeto?"* — nao existe sem projeto.
pub(super) fn grafana_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "grafana.get".to_owned(),
        title: "Observabilidade...".to_owned(),
        category: "Projeto".to_owned(),
        description: "O Grafana deste projeto: versao, fontes de dados e dashboards".to_owned(),
        // Ctrl+Alt+O de Observabilidade. O Ctrl+Alt+G, que seria o obvio, ja'
        // e' alias do F3 na UI — e atalho anunciado que faz outra coisa e' o
        // defeito que o `verificar-atalhos.sh` existe para pegar.
        default_shortcut: Some("Ctrl+Alt+O".to_owned()),
        requires_workspace: true,
    }]
}

/// Embarcados (roadmaps/35 §5.7, 2026-09-11): a sonda, o alvo e o depurador.
///
/// UM descriptor, e o id e' `probe.list` porque e' o metodo que a acao chama:
/// o painel abre perguntando a ferramenta o que esta' no USB. Ate' esta data o
/// `probe.list` era roteado no core e NENHUMA tela o pedia (`roadmaps/40`
/// §8.2) — o mesmo buraco que deixou o motor vetorial um dia sem porta.
///
/// `requires_workspace` e' true porque o painel edita o KIT (chip, alvo,
/// sysroot), e kit e' por projeto: sem workspace nao ha' `.kinein/toolchain.json`
/// para escrever.
pub(super) fn probe_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "probe.list".to_owned(),
        title: "Embarcados...".to_owned(),
        category: "Projeto".to_owned(),
        description: "A sonda conectada, o chip do alvo e o depurador do kit".to_owned(),
        // Ctrl+Alt+M de eMbarcados. O Ctrl+Alt+E ja' e' usado pela UI.
        default_shortcut: Some("Ctrl+Alt+M".to_owned()),
        requires_workspace: true,
    }]
}

/// Containers (roadmaps/28 §0: Docker e Podman NATIVOS; priorizado em
/// 2026-09-12): o painel do motor, dos containers, das imagens e do compose.
///
/// UM descriptor, e o id e' `container.list` porque e' a pergunta que o painel
/// faz ao abrir. `requires_workspace` e' FALSE: o motor e' da maquina, e ver
/// o que esta' rodando nao precisa de projeto — so' o compose precisa, e o
/// painel diz isso ao desabilitar os dois botoes.
pub(super) fn container_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "container.list".to_owned(),
        title: "Containers...".to_owned(),
        category: "Ambiente".to_owned(),
        description: "Docker ou Podman: o motor, os containers, as imagens e o compose".to_owned(),
        // Ctrl+Alt+W de "whale": C e' Continuar do debug, D e' alias do
        // debug.start — as letras obvias ja' tem dono na UI.
        default_shortcut: Some("Ctrl+Alt+W".to_owned()),
        requires_workspace: false,
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
