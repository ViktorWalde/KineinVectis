//! Comandos de execucao: run, run configurations e debug.

use kinein_protocol::CommandDescriptor;

pub(super) fn run_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "run.start".to_owned(),
            title: "Run".to_owned(),
            category: "Run".to_owned(),
            description: "Executa o projeto ou um comando no workspace, com saida ao vivo"
                .to_owned(),
            default_shortcut: Some("Shift+F10".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "run.stdin".to_owned(),
            title: "Send Input".to_owned(),
            category: "Run".to_owned(),
            description: "Envia texto para o stdin do processo em execucao".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "run.stop".to_owned(),
            title: "Stop".to_owned(),
            category: "Run".to_owned(),
            description: "Encerra o processo em execucao".to_owned(),
            default_shortcut: Some("Ctrl+F2".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.open".to_owned(),
            title: "Terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Abre o shell do usuario ($SHELL) num PTY na raiz do workspace".to_owned(),
            default_shortcut: Some("Alt+F12".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.input".to_owned(),
            title: "Terminal Input".to_owned(),
            category: "Terminal".to_owned(),
            description: "Envia texto para o shell do terminal aberto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "terminal.close".to_owned(),
            title: "Close Terminal".to_owned(),
            category: "Terminal".to_owned(),
            description: "Fecha a sessao de terminal do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

pub(super) fn runconfig_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "runConfig.list".to_owned(),
        title: "Run Configurations".to_owned(),
        category: "Run".to_owned(),
        description: "Configuracoes de execucao do workspace (seletor na toolbar)".to_owned(),
        default_shortcut: None,
        requires_workspace: true,
    }]
}

pub(super) fn debug_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "debug.start".to_owned(),
        title: "Debug".to_owned(),
        category: "Run".to_owned(),
        description: "Inicia sessao de debug (lldb-dap) no alvo do workspace".to_owned(),
        default_shortcut: Some("Shift+F9".to_owned()),
        requires_workspace: true,
    }]
}
