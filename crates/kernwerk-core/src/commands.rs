//! Command descriptors advertised to the UI (command palette, menus, shortcuts).
//!
//! Pure data: every function returns `CommandDescriptor` values and never
//! touches `Core` state. Split out of `lib.rs` to keep the core surface small.

use kernwerk_protocol::CommandDescriptor;

/// Returns every command descriptor advertised to the UI via `command.list`.
pub(crate) fn command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = core_command_descriptors();
    descriptors.extend(project_command_descriptors());
    descriptors.extend(run_command_descriptors());
    descriptors.extend(lsp_command_descriptors());
    descriptors
}

fn run_command_descriptors() -> Vec<CommandDescriptor> {
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

fn core_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "core.ping".to_owned(),
            title: "Ping Core".to_owned(),
            category: "Core".to_owned(),
            description: "Verifica se o core esta respondendo".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "core.shutdown".to_owned(),
            title: "Shutdown Core".to_owned(),
            category: "Core".to_owned(),
            description: "Solicita encerramento limpo do core".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "command.list".to_owned(),
            title: "List Commands".to_owned(),
            category: "Core".to_owned(),
            description: "Lista comandos registrados no core".to_owned(),
            default_shortcut: Some("Ctrl+Shift+A".to_owned()),
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "tools.detect".to_owned(),
            title: "Detect Tools".to_owned(),
            category: "Tools".to_owned(),
            description: "Detecta ferramentas externas e sugere instalacao quando faltarem"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "tools.status".to_owned(),
            title: "Tools Status".to_owned(),
            category: "Tools".to_owned(),
            description: "Mostra o ultimo status conhecido das ferramentas externas".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
    ]
}

fn project_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = workspace_command_descriptors();
    descriptors.extend(file_command_descriptors());
    descriptors
}

fn workspace_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "workspace.open".to_owned(),
            title: "Open Workspace".to_owned(),
            category: "Workspace".to_owned(),
            description: "Abre uma pasta como workspace e identifica o tipo de projeto".to_owned(),
            default_shortcut: Some("Ctrl+O".to_owned()),
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.browse".to_owned(),
            title: "Browse Workspace Folders".to_owned(),
            category: "Workspace".to_owned(),
            description: "Lista diretorios para o seletor proprio de workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.createFolder".to_owned(),
            title: "New Folder".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria uma pasta pelo seletor proprio de workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.createProject".to_owned(),
            title: "New Project".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria um projeto inicial e abre como workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.status".to_owned(),
            title: "Workspace Status".to_owned(),
            category: "Workspace".to_owned(),
            description: "Mostra o workspace aberto no momento".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
        },
        CommandDescriptor {
            id: "workspace.close".to_owned(),
            title: "Close Workspace".to_owned(),
            category: "Workspace".to_owned(),
            description: "Fecha o workspace atual".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

fn file_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "fs.list".to_owned(),
            title: "List Directory".to_owned(),
            category: "Files".to_owned(),
            description: "Lista um diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.read".to_owned(),
            title: "Read File".to_owned(),
            category: "Files".to_owned(),
            description: "Le um arquivo de texto dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.createFile".to_owned(),
            title: "New File".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo arquivo de texto dentro do workspace sem sobrescrever"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.createDirectory".to_owned(),
            title: "New Directory".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo diretorio dentro do workspace sem sobrescrever".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.write".to_owned(),
            title: "Save File".to_owned(),
            category: "Files".to_owned(),
            description: "Salva um arquivo existente dentro do workspace".to_owned(),
            default_shortcut: Some("Ctrl+S".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.rename".to_owned(),
            title: "Rename".to_owned(),
            category: "Files".to_owned(),
            description: "Renomeia ou move um arquivo ou diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.delete".to_owned(),
            title: "Delete".to_owned(),
            category: "Files".to_owned(),
            description: "Remove um arquivo ou diretorio dentro do workspace".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.search".to_owned(),
            title: "Find in Files".to_owned(),
            category: "Files".to_owned(),
            description: "Busca texto em todos os arquivos do workspace".to_owned(),
            default_shortcut: Some("Ctrl+Shift+F".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "fs.findFiles".to_owned(),
            title: "Find File".to_owned(),
            category: "Files".to_owned(),
            description: "Busca arquivos por nome usando fd".to_owned(),
            default_shortcut: Some("Ctrl+Shift+N".to_owned()),
            requires_workspace: true,
        },
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

fn lsp_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "lsp.semanticTokens".to_owned(),
            title: "Semantic Highlighting".to_owned(),
            category: "LSP".to_owned(),
            description: "Resolve cores semanticas do arquivo aberto via LSP".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.didChange".to_owned(),
            title: "Sync Editor Buffer".to_owned(),
            category: "LSP".to_owned(),
            description: "Sincroniza o buffer aberto com o servidor LSP gerenciado pelo core"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.definition".to_owned(),
            title: "Go to Definition".to_owned(),
            category: "LSP".to_owned(),
            description: "Resolve a definicao do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Ctrl+B".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.hover".to_owned(),
            title: "Quick Documentation".to_owned(),
            category: "LSP".to_owned(),
            description: "Mostra informacao rapida do simbolo na posicao atual do editor"
                .to_owned(),
            default_shortcut: Some("Ctrl+Q".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.completion".to_owned(),
            title: "Code Completion".to_owned(),
            category: "LSP".to_owned(),
            description: "Lista completions do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Ctrl+Space".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.references".to_owned(),
            title: "Find Usages".to_owned(),
            category: "LSP".to_owned(),
            description: "Lista os usos do simbolo na posicao atual do editor".to_owned(),
            default_shortcut: Some("Alt+F7".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.rename".to_owned(),
            title: "Rename Symbol".to_owned(),
            category: "LSP".to_owned(),
            description: "Renomeia o simbolo em todos os arquivos do workspace via LSP".to_owned(),
            default_shortcut: Some("Shift+F6".to_owned()),
            requires_workspace: true,
        },
    ]
}
