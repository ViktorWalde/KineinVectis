//! Comandos do editor: formatacao, busca no arquivo e os `lsp.*`.

use kinein_protocol::CommandDescriptor;

pub(super) fn format_command_descriptors() -> Vec<CommandDescriptor> {
    vec![CommandDescriptor {
        id: "format.text".to_owned(),
        title: "Reformat File".to_owned(),
        category: "Editor".to_owned(),
        description: "Formata o arquivo atual com rustfmt/clang-format".to_owned(),
        default_shortcut: Some("Ctrl+Alt+L".to_owned()),
        requires_workspace: true,
    }]
}

/// Find/Replace dentro do arquivo aberto (D1b, DocsPublic/roadmaps/24).
///
/// Sao comandos de UI PURA: o buffer vive no editor, entao a busca nao passa
/// pelo core (diferente do `fs.search`, que roda ripgrep no DISCO). O core so
/// os ANUNCIA aqui para que aparecam na paleta de comandos (`command.list`);
/// quem executa e o `CommandDispatcher` da UI, como ja acontece com
/// `settings.get`.
pub(super) fn editor_find_command_descriptors() -> Vec<CommandDescriptor> {
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

pub(super) fn lsp_command_descriptors() -> Vec<CommandDescriptor> {
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

pub(super) fn lsp_core_command_descriptors() -> Vec<CommandDescriptor> {
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
            // "Alt+Return", nao "Alt+Enter": no Qt sao teclas DIFERENTES —
            // `Key_Enter` e' o do teclado numerico. A UI sempre ligou
            // Alt+Return; a paleta e' que anunciava a outra.
            default_shortcut: Some("Alt+Return".to_owned()),
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

pub(super) fn lsp_workspace_edit_command_descriptors() -> Vec<CommandDescriptor> {
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
