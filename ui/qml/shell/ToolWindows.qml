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
// O `componente` que a V3 tambem lista NAO esta' aqui, de proposito: o slot
// esquerdo ainda e' montado pelo `ShellLeftWindowHost`, e trocar isso e'
// trabalho da V4 — onde o Remote de fato vira janela lateral. Campo sem
// consumidor seria dado morto fingindo desenho.
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
    property bool workspaceOpen: false

    visible: false

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
        },
        {
            "id": "containers", "label": "", "icon": "container",
            "tooltip": qsTr("Containers (Ctrl+Alt+W)"), "area": "left", "order": 50,
            "available": true,
            "active": root.containerController !== null && root.containerController !== undefined
                      && root.containerController.panelVisible
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
        },
        {
            "id": "observability", "label": qsTr("Grafana"), "icon": "observability",
            "tooltip": qsTr("Observabilidade — Grafana (Ctrl+Alt+O)"), "area": "left",
            "order": 60, "available": true,
            "active": root.grafanaController !== null && root.grafanaController !== undefined
                      && root.grafanaController.panelVisible
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
