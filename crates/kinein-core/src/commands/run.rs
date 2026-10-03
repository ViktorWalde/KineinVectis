//! Comandos de execucao: run, run configurations e debug.

use kinein_protocol::CommandDescriptor;

pub(super) fn run_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "run.start".to_owned(),
            title: "Executar".to_owned(),
            category: "Run".to_owned(),
            description: "Executa o projeto ou um comando numa aba do terminal integrado"
                .to_owned(),
            default_shortcut: Some("Shift+F10".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "run.stop".to_owned(),
            title: "Parar".to_owned(),
            category: "Run".to_owned(),
            description: "Fecha a aba do terminal da execucao".to_owned(),
            default_shortcut: Some("Ctrl+F2".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "terminal.open".to_owned(),
            title: "Terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Abre o shell do usuario ($SHELL) num PTY na raiz do projeto".to_owned(),
            default_shortcut: Some("Alt+F12".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "terminal.input".to_owned(),
            title: "Entrada do terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Envia texto para o shell do terminal aberto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: true,
        },
        CommandDescriptor {
            id: "terminal.close".to_owned(),
            title: "Fechar terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Fecha a sessao de terminal do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
    ]
}

pub(super) fn runconfig_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "runConfig.list".to_owned(),
        title: "Configuracoes de execucao".to_owned(),
        category: "Run".to_owned(),
        description: "Configuracoes de execucao do projeto (seletor na toolbar)".to_owned(),
        default_shortcut: None,
        requires_workspace: true,
        internal: false,
    }]
}

pub(super) fn debug_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "debug.start".to_owned(),
        title: "Depurar".to_owned(),
        category: "Run".to_owned(),
        description: "Inicia sessao de debug (lldb-dap) no alvo do projeto".to_owned(),
        default_shortcut: Some("Shift+F9".to_owned()),
        requires_workspace: true,
        internal: false,
    }]
}
