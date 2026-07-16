//! Command descriptors advertised to the UI (command palette, menus, shortcuts).
//!
//! Pure data: every function returns `CommandDescriptor` values and never
//! touches `Core` state. Split out of `lib.rs` to keep the core surface small.

use kinein_protocol::CommandDescriptor;

/// Returns every command descriptor advertised to the UI via `command.list`.
pub(crate) fn command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = core_command_descriptors();
    descriptors.extend(project_command_descriptors());
    descriptors.extend(run_command_descriptors());
    descriptors.extend(lsp_command_descriptors());
    descriptors.extend(jobs_command_descriptors());
    descriptors.extend(format_command_descriptors());
    descriptors.extend(editor_find_command_descriptors());
    descriptors.extend(cmake_command_descriptors());
    descriptors.extend(cargo_command_descriptors());
    descriptors.extend(runconfig_command_descriptors());
    descriptors.extend(debug_command_descriptors());
    descriptors.extend(git_command_descriptors());
    descriptors.extend(settings_command_descriptors());
    descriptors
}

fn settings_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "settings.get".to_owned(),
        title: "Configuracoes".to_owned(),
        category: "IDE".to_owned(),
        description: "Abre as configuracoes (fonte do editor, format-on-save, auto-close)"
            .to_owned(),
        default_shortcut: Some("Ctrl+Alt+S".to_owned()),
        requires_workspace: false,
    }]
}

fn git_command_descriptors() -> Vec<CommandDescriptor> {
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

fn debug_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "debug.start".to_owned(),
        title: "Debug".to_owned(),
        category: "Run".to_owned(),
        description: "Inicia sessao de debug (lldb-dap) no alvo do workspace".to_owned(),
        default_shortcut: Some("Shift+F9".to_owned()),
        requires_workspace: true,
    }]
}

fn runconfig_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "runConfig.list".to_owned(),
        title: "Run Configurations".to_owned(),
        category: "Run".to_owned(),
        description: "Configuracoes de execucao do workspace (seletor na toolbar)".to_owned(),
        default_shortcut: None,
        requires_workspace: true,
    }]
}

fn cargo_command_descriptors() -> Vec<CommandDescriptor> {
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

fn cmake_command_descriptors() -> Vec<CommandDescriptor> {
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

fn format_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "format.text".to_owned(),
        title: "Reformat File".to_owned(),
        category: "Editor".to_owned(),
        description: "Formata o arquivo atual com rustfmt/clang-format".to_owned(),
        default_shortcut: Some("Ctrl+Alt+L".to_owned()),
        requires_workspace: true,
    }]
}

/// Find/Replace dentro do arquivo aberto (D1b, docs/roadmaps/24).
///
/// Sao comandos de UI PURA: o buffer vive no editor, entao a busca nao passa
/// pelo core (diferente do `fs.search`, que roda ripgrep no DISCO). O core so
/// os ANUNCIA aqui para que aparecam na paleta de comandos (`command.list`);
/// quem executa e o `CommandDispatcher` da UI, como ja acontece com
/// `settings.get`.
fn editor_find_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "editor.find".to_owned(),
            title: "Localizar no arquivo".to_owned(),
            category: "Editor".to_owned(),
            description: "Busca texto no arquivo aberto (case, palavra inteira, regex)".to_owned(),
            default_shortcut: Some("Ctrl+F".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "editor.replace".to_owned(),
            title: "Substituir no arquivo".to_owned(),
            category: "Editor".to_owned(),
            description: "Busca e substitui no arquivo aberto (um a um ou tudo)".to_owned(),
            default_shortcut: Some("Ctrl+H".to_owned()),
            requires_workspace: true,
        },
    ]
}

fn jobs_command_descriptors() -> Vec<CommandDescriptor> {
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
        CommandDescriptor {
            id: "environment.scan".to_owned(),
            title: "Scan Environment".to_owned(),
            category: "Tools".to_owned(),
            description: "Executa scan de ambiente/toolchain em background como job".to_owned(),
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
        CommandDescriptor {
            id: "workspace.saveSession".to_owned(),
            title: "Save Session".to_owned(),
            category: "Workspace".to_owned(),
            description: "Persiste as abas abertas do workspace (.kinein/session.json)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}

fn file_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = vec![
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
            id: "fs.replace".to_owned(),
            title: "Replace in Files".to_owned(),
            category: "Files".to_owned(),
            description: "Substitui texto no workspace com rollback multi-arquivo".to_owned(),
            default_shortcut: Some("Ctrl+Shift+H".to_owned()),
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
    ];
    descriptors.extend(build_command_descriptors());
    descriptors
}

fn build_command_descriptors() -> Vec<CommandDescriptor> {
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

fn lsp_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = lsp_core_command_descriptors();
    descriptors.extend(lsp_workspace_edit_command_descriptors());
    descriptors.push(CommandDescriptor {
        id: "lsp.switchSourceHeader".to_owned(),
        title: "C/C++: Alternar header/source".to_owned(),
        category: "LSP".to_owned(),
        description: "Alterna entre o header e o source do componente C/C++ (clangd)".to_owned(),
        default_shortcut: Some("Alt+O".to_owned()),
        requires_workspace: true,
    });
    descriptors.push(CommandDescriptor {
        id: "lsp.restart".to_owned(),
        title: "LSP: Reiniciar servidor".to_owned(),
        category: "LSP".to_owned(),
        description: "Reinicia os servidores de linguagem (clangd/rust-analyzer) quando travam"
            .to_owned(),
        default_shortcut: None,
        requires_workspace: true,
    });
    descriptors
}

fn lsp_core_command_descriptors() -> Vec<CommandDescriptor> {
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
            id: "lsp.codeActions".to_owned(),
            title: "Code Actions".to_owned(),
            category: "LSP".to_owned(),
            description: "Lista quick fixes e refactors do LSP no ponto do cursor".to_owned(),
            default_shortcut: Some("Alt+Enter".to_owned()),
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.documentSymbols".to_owned(),
            title: "File Structure".to_owned(),
            category: "LSP".to_owned(),
            description: "Simbolos do arquivo atual (prefixo @ no Search Everywhere)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.workspaceSymbols".to_owned(),
            title: "Go to Symbol".to_owned(),
            category: "LSP".to_owned(),
            description: "Busca simbolos no workspace (prefixo # no Search Everywhere)".to_owned(),
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
            description: "Prepara a previa de rename em todos os arquivos via LSP".to_owned(),
            default_shortcut: Some("Shift+F6".to_owned()),
            requires_workspace: true,
        },
    ]
}

fn lsp_workspace_edit_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "lsp.applyCodeAction".to_owned(),
            title: "Preview Code Action".to_owned(),
            category: "LSP".to_owned(),
            description: "Prepara a previa de uma acao da ultima consulta".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.workspaceEdit.apply".to_owned(),
            title: "Apply Workspace Edit".to_owned(),
            category: "LSP".to_owned(),
            description: "Confirma uma transacao LSP com validacao e rollback".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "lsp.workspaceEdit.cancel".to_owned(),
            title: "Cancel Workspace Edit".to_owned(),
            category: "LSP".to_owned(),
            description: "Descarta uma transacao LSP sem alterar arquivos".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
        CommandDescriptor {
            id: "syntaxTree.update".to_owned(),
            title: "Update Local Syntax Tree".to_owned(),
            category: "Language".to_owned(),
            description: "Atualiza highlight, folding, outline e locals estruturais".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
        },
    ]
}
