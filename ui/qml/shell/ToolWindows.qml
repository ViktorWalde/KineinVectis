pragma ComponentBehavior: Bound
import QtQuick

// As TOOL WINDOWS do trilho, como DADO (fatia V3, 2026-09-24).
//
// Antes, cada entrada custava tres lugares: uma propriedade `xActive` e um
// sinal `xRequested` no `SideRail`, mais o bloco do botao — e do outro lado, no
// `ShellWorkspaceHost`, o binding do `active` e o `onXRequested`. Sete entradas,
// duas fontes de verdade. O aceite da V3 e' justamente esse: acrescentar o
// Remote ao slot nao pode exigir branching nominal em varios arquivos ao mesmo
// tempo. Agora custa UMA entrada nesta lista.
//
// O que uma entrada declara (o `ToolWindowEntry` minimo da V3): id, titulo,
// icone, area, ordem, disponibilidade e ativo.
//
// O `componente` da V3 entrou em 2026-09-25, quando ganhou consumidor: os
// paineis de ambiente. O `ShellEnvironmentOverlays` tinha cinco blocos de nove
// linhas quase iguais — o proprio arquivo ja' dizia que "todos tem a mesma
// forma: moldura de dialogo sobre um controller com `panelVisible`, mesmo ciclo
// abrir/fechar, mesma folga de janela". Agora a entrada carrega o painel, com a
// fiacao especifica dele junto, e o host cuida so' do que e' igual.
//
// Biblioteca e Instalacao NAO ganharam `componente`: sao overlays de ambiente,
// mas nao sao entradas do trilho. Forcar a abstracao sobre quem nao e' tool
// window seria cerimonia.
//
// O slot ESQUERDO (explorer e Git) continua montado pelo `ShellLeftWindowHost`:
// sao duas janelas com fiacao inteiramente propria, e um `Loader` generico nao
// tem como supri-la sem um saco de propriedades. Quando houver a terceira, a
// conversa muda.
//
// NAO e' registry global nem API de plugin (a V3 proibe as duas): e' uma lista
// interna, estatica, com donos conhecidos em tempo de compilacao.
Item {
    id: root

    property var shellController: null
    property var embeddedController: null
    property var dataSourceController: null
    property var containerController: null
    property var grafanaController: null
    property var remoteController: null
    // O painel de embarcados edita o KIT (chip, alvo, depurador).
    property var toolchainController: null
    property bool workspaceOpen: false

    visible: false

    // Os PAINEIS das entradas. Cada um amarra o seu controller; o que e' igual
    // (ancoras, z, folga da janela) fica com quem os monta. `parent` aqui e' o
    // `Loader` do host, que preenche a janela.
    readonly property Component embeddedPanel: Component {
        EmbeddedPanelHost {
            controller: root.embeddedController
            toolchainController: root.toolchainController
            maxAvailableWidth: parent.width - 4 * Theme.spacingMedium
            maxAvailableHeight: parent.height - 4 * Theme.spacingMedium
            onDismissRequested: root.embeddedController.close()
        }
    }

    readonly property Component databasePanel: Component {
        DataSourcePanelHost {
            controller: root.dataSourceController
            maxAvailableWidth: parent.width - 4 * Theme.spacingMedium
            maxAvailableHeight: parent.height - 4 * Theme.spacingMedium
            onDismissRequested: root.dataSourceController.close()
        }
    }

    readonly property Component containersPanel: Component {
        ContainerPanelHost {
            controller: root.containerController
            maxAvailableWidth: parent.width - 4 * Theme.spacingMedium
            maxAvailableHeight: parent.height - 4 * Theme.spacingMedium
            onDismissRequested: root.containerController.close()
        }
    }

    readonly property Component remotePanel: Component {
        RemotePanelHost {
            controller: root.remoteController
            maxAvailableWidth: parent.width - 4 * Theme.spacingMedium
            maxAvailableHeight: parent.height - 4 * Theme.spacingMedium
            onDismissRequested: root.remoteController.close()
        }
    }

    readonly property Component observabilityPanel: Component {
        GrafanaPanelHost {
            controller: root.grafanaController
            maxAvailableWidth: parent.width - 4 * Theme.spacingMedium
            maxAvailableHeight: parent.height - 4 * Theme.spacingMedium
            onDismissRequested: root.grafanaController.close()
        }
    }

    // O GIT SAIU DO TRILHO em 2026-09-24, por observacao do autor: o widget do
    // cabecalho ja' abre o MESMO painel (`toggleBottomTab("git")`, conferido) e
    // mostra o que o icone nao mostrava — branch, ahead/behind e quantas
    // mudancas ha'. Dois caminhos para o mesmo gesto, um deles cego.
    //
    // Medido antes de tirar: `branchLabel` so' fica vazio quando NAO ha'
    // repositorio (ele cai para "HEAD" ou "<sha> (solto)" em qualquer repo
    // real), e sem repositorio o painel Git nao teria o que mostrar. A paleta
    // continua alcancando `git.status`, `git.commit` e `git.log`.
    //
    // Esta remocao custou UMA entrada aqui. Antes da V3 custaria cinco lugares
    // em dois arquivos — e' o aceite daquela fatia, exercitado.
    readonly property var entries: [
        {
            "id": "explorer", "label": qsTr("Projeto"), "icon": "project",
            "tooltip": qsTr("Projeto"), "area": "left", "order": 10,
            "available": root.workspaceOpen,
            "active": root.shellController !== null
                      && root.shellController.effectiveShowExplorer && root.workspaceOpen
        },
        // Embarcados e' por projeto (o kit mora no .kinein), como o Git.
        {
            "id": "embedded", "label": qsTr("Embarcados"), "icon": "embedded",
            "tooltip": qsTr("Embarcados — placa, projeto, gravar, kit (Ctrl+Alt+M)"),
            "area": "left", "order": 30, "available": root.workspaceOpen,
            "active": root.embeddedController !== null && root.embeddedController !== undefined
                      && root.embeddedController.panelVisible
            ,"panel": root.embeddedPanel
        },
        // Ferramentas NATIVAS com atalho visual (decisao do autor, 2026-09-12;
        // a ordem e o banco em 2026-09-13): banco, containers e observabilidade
        // abrem SEM projeto aberto — os perfis, o motor e o Grafana sao da
        // maquina, nao do workspace. "Ferramentas" fecha a lista.
        {
            "id": "database", "label": qsTr("Banco"), "icon": "database",
            "tooltip": qsTr("Banco de dados (Ctrl+Alt+J)"), "area": "left", "order": 40,
            "available": true,
            "active": root.dataSourceController !== null && root.dataSourceController !== undefined
                      && root.dataSourceController.panelVisible
            ,"panel": root.databasePanel
        },
        {
            "id": "containers", "label": "", "icon": "container",
            "tooltip": qsTr("Containers (Ctrl+Alt+W)"), "area": "left", "order": 50,
            "available": true,
            "active": root.containerController !== null && root.containerController !== undefined
                      && root.containerController.panelVisible
            ,"panel": root.containersPanel
        },
        // O REMOTE entrou no trilho na V4 (2026-09-25), e custou esta entrada —
        // era o aceite da V3, exercitado. Ele abre SEM projeto aberto pelo mesmo
        // motivo que o banco e os containers: o alvo e' da maquina, e o
        // workspace espelhado ate' nasce de escolher um.
        {
            "id": "remote", "label": qsTr("Remoto"), "icon": "remote",
            "tooltip": qsTr("Alvo remoto — Linux por SSH"), "area": "left", "order": 55,
            "available": true,
            "active": root.remoteController !== null && root.remoteController !== undefined
                      && root.remoteController.panelVisible
            ,"panel": root.remotePanel
        },
        {
            "id": "observability", "label": qsTr("Grafana"), "icon": "observability",
            "tooltip": qsTr("Observabilidade — Grafana (Ctrl+Alt+O)"), "area": "left",
            "order": 60, "available": true,
            "active": root.grafanaController !== null && root.grafanaController !== undefined
                      && root.grafanaController.panelVisible
            ,"panel": root.observabilityPanel
        },
        {
            "id": "tools", "label": "", "icon": "tools",
            "tooltip": qsTr("Ferramentas"), "area": "left", "order": 70,
            "available": true,
            "active": root.shellController !== null && root.shellController.showBottomPanel
                      && root.shellController.bottomTab === "tools"
        }
    ]

    // Um dono AUSENTE e' o mesmo caso de um id sem dono: resultado observavel,
    // nao excecao. O trilho existe antes dos controllers em teste e na abertura
    // da janela, e chamar `open()` de um `null` derrubava a funcao inteira.
    function openOwner(owner) {
        if (owner === null || owner === undefined) {
            return false;
        }
        owner.open();
        return true;
    }

    // As entradas que trazem painel de ambiente, na ordem do trilho.
    readonly property var overlayEntries: root.entries.filter(function(e) {
        return e.panel !== undefined;
    })

    // O UNICO lugar que sabe o que cada id faz. Antes eram sete
    // `onXRequested` espalhados pelo host.
    function activate(id) {
        switch (id) {
        case "explorer":
            if (shellController === null || shellController === undefined) {
                return false;
            }
            shellController.toggleExplorer();
            return true;
        case "embedded":
            return openOwner(embeddedController);
        case "database":
            return openOwner(dataSourceController);
        case "containers":
            return openOwner(containerController);
        case "observability":
            return openOwner(grafanaController);
        case "remote":
            return openOwner(remoteController);
        case "tools":
            if (shellController === null || shellController === undefined) {
                return false;
            }
            shellController.toggleBottomTab("tools");
            return true;
        default:
            // Mesma regra do CommandDispatcher: id sem dono e' resultado
            // observavel, nao silencio.
            return false;
        }
    }
}
