//! Comandos do shell da IDE: nucleo, workspace, arquivos e configuracoes.

use kinein_protocol::CommandDescriptor;

pub(super) fn core_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "core.ping".to_owned(),
            title: "Ping do core".to_owned(),
            category: "Core".to_owned(),
            description: "Verifica se o core esta respondendo".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: true,
        },
        CommandDescriptor {
            id: "core.shutdown".to_owned(),
            title: "Encerrar o core".to_owned(),
            category: "Core".to_owned(),
            description: "Solicita encerramento limpo do core".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: true,
        },
        CommandDescriptor {
            id: "command.list".to_owned(),
            title: "Paleta de comandos".to_owned(),
            category: "Core".to_owned(),
            description: "Lista comandos registrados no core".to_owned(),
            default_shortcut: Some("Ctrl+Shift+A".to_owned()),
            requires_workspace: false,
            internal: true,
        },
        CommandDescriptor {
            id: "tools.detect".to_owned(),
            title: "Detectar ferramentas".to_owned(),
            category: "Tools".to_owned(),
            description: "Detecta ferramentas externas e sugere instalacao quando faltarem"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
        CommandDescriptor {
            id: "tools.status".to_owned(),
            title: "Estado das ferramentas".to_owned(),
            category: "Tools".to_owned(),
            description: "Mostra o ultimo status conhecido das ferramentas externas".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
        CommandDescriptor {
            id: "environment.scan".to_owned(),
            title: "Verificar ambiente".to_owned(),
            category: "Tools".to_owned(),
            description: "Executa scan de ambiente/toolchain em background como job".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
    ]
}

pub(super) fn settings_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "settings.get".to_owned(),
            title: "Configuracoes".to_owned(),
            category: "IDE".to_owned(),
            description: "Abre as configuracoes (fonte do editor, format-on-save, auto-close)"
                .to_owned(),
            default_shortcut: Some("Ctrl+Alt+S".to_owned()),
            requires_workspace: false,
            internal: false,
        },
        // O painel de areas do trilho (0.3.7): fixar, ocultar e abrir as areas
        // pelo teclado e pela paleta, nao so' pelo mouse (roadmap 53 R4).
        CommandDescriptor {
            id: "view.areas".to_owned(),
            title: "Areas da IDE...".to_owned(),
            category: "IDE".to_owned(),
            description: "Mostra as areas do trilho: fixar, ocultar e abrir".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
        // O modo Foco e o "voltar ao editor" (0.3.9 F4, roadmap 53 §5.8): mais
        // espaco para o codigo num gesto, e o caminho de volta sempre a mao.
        CommandDescriptor {
            id: "view.focusMode".to_owned(),
            title: "Modo Foco".to_owned(),
            category: "IDE".to_owned(),
            description: "Recolhe os paineis em volta do editor; de novo, restaura como estavam"
                .to_owned(),
            default_shortcut: Some("Ctrl+Shift+F12".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "view.cycleFocus".to_owned(),
            title: "Proxima area".to_owned(),
            category: "IDE".to_owned(),
            description: "Leva o teclado para a proxima area: esquerda, editor, painel de baixo"
                .to_owned(),
            default_shortcut: Some("Ctrl+F6".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "view.cycleFocusBack".to_owned(),
            title: "Area anterior".to_owned(),
            category: "IDE".to_owned(),
            description: "Leva o teclado para a area anterior".to_owned(),
            default_shortcut: Some("Ctrl+Shift+F6".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "view.returnToEditor".to_owned(),
            title: "Voltar ao editor".to_owned(),
            category: "IDE".to_owned(),
            description: "Devolve o foco do teclado ao editor".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
    ]
}

pub(super) fn project_command_descriptors() -> Vec<CommandDescriptor> {
    let mut descriptors = workspace_command_descriptors();
    descriptors.extend(file_command_descriptors());
    descriptors.extend(search_command_descriptors());
    descriptors
}

pub(super) fn workspace_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "workspace.open".to_owned(),
            title: "Abrir projeto...".to_owned(),
            category: "Workspace".to_owned(),
            description: "Abre uma pasta como projeto e identifica o tipo dele".to_owned(),
            default_shortcut: Some("Ctrl+O".to_owned()),
            requires_workspace: false,
            internal: false,
        },
        CommandDescriptor {
            id: "workspace.browse".to_owned(),
            title: "Navegar pastas".to_owned(),
            category: "Workspace".to_owned(),
            description: "Lista diretorios para o seletor de pastas da IDE".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: true,
        },
        CommandDescriptor {
            id: "workspace.createFolder".to_owned(),
            title: "Nova pasta".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria uma pasta pelo seletor de pastas da IDE".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
        CommandDescriptor {
            id: "workspace.createProject".to_owned(),
            title: "Criar projeto...".to_owned(),
            category: "Workspace".to_owned(),
            description: "Cria um projeto inicial e o abre".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: false,
        },
        CommandDescriptor {
            id: "workspace.status".to_owned(),
            title: "Estado do projeto".to_owned(),
            category: "Workspace".to_owned(),
            description: "Mostra o projeto aberto no momento".to_owned(),
            default_shortcut: None,
            requires_workspace: false,
            internal: true,
        },
        CommandDescriptor {
            id: "workspace.close".to_owned(),
            title: "Fechar projeto".to_owned(),
            category: "Workspace".to_owned(),
            description: "Fecha o projeto atual".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "workspace.saveSession".to_owned(),
            title: "Salvar sessao".to_owned(),
            category: "Workspace".to_owned(),
            description: "Persiste as abas abertas do projeto (.kinein/session.json)".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: true,
        },
    ]
}

pub(super) fn file_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "fs.list".to_owned(),
            title: "Listar pasta".to_owned(),
            category: "Files".to_owned(),
            description: "Lista um diretorio dentro do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: true,
        },
        CommandDescriptor {
            id: "fs.read".to_owned(),
            title: "Ler arquivo".to_owned(),
            category: "Files".to_owned(),
            description: "Le um arquivo de texto dentro do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: true,
        },
        CommandDescriptor {
            id: "fs.createFile".to_owned(),
            title: "Novo arquivo".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo arquivo de texto dentro do projeto sem sobrescrever"
                .to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.createDirectory".to_owned(),
            title: "Nova pasta no projeto".to_owned(),
            category: "Files".to_owned(),
            description: "Cria um novo diretorio dentro do projeto sem sobrescrever".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.write".to_owned(),
            title: "Salvar arquivo".to_owned(),
            category: "Files".to_owned(),
            description: "Salva um arquivo existente dentro do projeto".to_owned(),
            default_shortcut: Some("Ctrl+S".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.rename".to_owned(),
            title: "Renomear".to_owned(),
            category: "Files".to_owned(),
            description: "Renomeia ou move um arquivo ou diretorio dentro do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.delete".to_owned(),
            title: "Excluir".to_owned(),
            category: "Files".to_owned(),
            description: "Remove um arquivo ou diretorio dentro do projeto".to_owned(),
            default_shortcut: None,
            requires_workspace: true,
            internal: false,
        },
    ]
}

/// A busca no projeto: texto, substituicao, arquivo e simbolo.
pub(super) fn search_command_descriptors() -> Vec<CommandDescriptor> {
    vec![
        CommandDescriptor {
            id: "fs.search".to_owned(),
            title: "Buscar no projeto".to_owned(),
            category: "Files".to_owned(),
            description: "Busca texto em todos os arquivos do projeto".to_owned(),
            default_shortcut: Some("Ctrl+Shift+F".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.replace".to_owned(),
            title: "Substituir no projeto".to_owned(),
            category: "Files".to_owned(),
            description: "Substitui texto no projeto com rollback multi-arquivo".to_owned(),
            default_shortcut: Some("Ctrl+Shift+H".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "fs.findFiles".to_owned(),
            title: "Ir para arquivo".to_owned(),
            category: "Files".to_owned(),
            description: "Busca arquivos por nome usando fd".to_owned(),
            default_shortcut: Some("Ctrl+Shift+N".to_owned()),
            requires_workspace: true,
            internal: false,
        },
        CommandDescriptor {
            id: "index.symbols".to_owned(),
            title: "Simbolos".to_owned(),
            category: "Files".to_owned(),
            description:
                "Abre a aba Simbolos (a direita) e busca declaracoes por nome no indice do projeto"
                    .to_owned(),
            default_shortcut: Some("Alt+7".to_owned()),
            requires_workspace: true,
            internal: false,
        },
    ]
}
