//! Comandos do dominio Git.

use kinein_protocol::CommandDescriptor;

pub(super) fn git_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "git.status".to_owned(),
            title: "Git: Atualizar status".to_owned(),
            category: "Git".to_owned(),
            description: "Reconsulta o git status do workspace (branch e mudancas)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.commit".to_owned(),
            title: "Git: Commit...".to_owned(),
            category: "Git".to_owned(),
            description: "Abre a aba Git para stage e commit das mudancas".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.fileDiff".to_owned(),
            title: "Git: Diff do arquivo".to_owned(),
            category: "Git".to_owned(),
            description: "Mostra o diff do arquivo atual contra o HEAD".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.blame".to_owned(),
            title: "Git: Blame do arquivo".to_owned(),
            category: "Git".to_owned(),
            description: "Liga/desliga autor e idade de cada linha na gutter do editor".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.log".to_owned(),
            title: "Git: Historico".to_owned(),
            category: "Git".to_owned(),
            description: "Abre o historico de commits na aba Git (diff por clique)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.branches".to_owned(),
            title: "Git: Trocar branch...".to_owned(),
            category: "Git".to_owned(),
            description: "Lista branches locais e permite trocar a branch ativa".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.pull".to_owned(),
            title: "Git: Pull (fast-forward)".to_owned(),
            category: "Git".to_owned(),
            description: "Atualiza a branch via job usando apenas fast-forward".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.push".to_owned(),
            title: "Git: Push".to_owned(),
            category: "Git".to_owned(),
            description: "Envia a branch atual ao upstream configurado via job".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "git.stash".to_owned(),
            title: "Git: Stash...".to_owned(),
            category: "Git".to_owned(),
            description: "Guarda ou restaura mudancas locais incluindo untracked".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}
