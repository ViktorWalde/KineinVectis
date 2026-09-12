import QtQuick

// Os atalhos que ABREM UM PAINEL DE CONFIGURACAO.
//
// POR QUE ESTE ARQUIVO EXISTE (2026-09-04). O `GlobalShortcuts` cruzou o
// limite ao ganhar o atalho da observabilidade, e a catraca mandou olhar. Os
// tres suspeitos da §4 regra 9, na ordem: a mudanca (um Shortcut legitimo — o
// gatilho), a categoria (o arquivo nao desenha nada, mas tambem nao e'
// controller nem composicao) e o arquivo.
//
// Foi o arquivo, e o vocabulario misturado estava a' vista. O `GlobalShortcuts`
// liga tecla a ACAO — salvar, compilar, buscar, ir para a definicao —, coisas
// que acontecem no que ja' esta' aberto. Estes seis fazem outra coisa: ABREM
// UMA TELA de configuracao do projeto. Sao os mesmos seis do menu "Ambiente",
// e tem os mesmos donos.
//
// A prova do corte: as seis propriedades de controller DEIXARAM DE EXISTIR no
// arquivo de origem — elas nao mudaram de lugar, e por isso o `grep` de
// `settingsController` no `GlobalShortcuts.qml` volta vazio.
//
// A regra do repositorio continua valendo aqui: toda acao de atalho precisa de
// acionamento manual, e todas as seis tem — o menu "Ambiente" e a paleta.
Item {
    id: root

    property var settingsController: null
    property var configActionController: null
    property var libraryController: null
    property var dataSourceController: null
    property var grafanaController: null
    property var embeddedController: null
    property var setupController: null
    property var containerController: null

    visible: false

    Shortcut {
        // comando: settings.get
        sequence: "Ctrl+Alt+S"
        onActivated: root.settingsController.openDialog()
    }

    Shortcut {
        // comando: library.list
        // Ctrl+Alt+K, e NAO Ctrl+Alt+L: ate' 2026-09-04 este painel anunciava
        // Ctrl+Alt+L na paleta e nao tinha Shortcut nenhum — quem apertava
        // FORMATAVA o arquivo, porque o `format.text` anunciava o mesmo
        // atalho e era esse que a UI ligava. Achado por relato de uso.
        sequence: "Ctrl+Alt+K"
        onActivated: root.libraryController.open()
    }

    Shortcut {
        // comando: datasource.list
        // Ctrl+Alt+J pelo mesmo motivo: o Ctrl+Alt+D que este comando
        // anunciava ao nascer ja' era alias do `debug.start` na UI.
        sequence: "Ctrl+Alt+J"
        onActivated: root.dataSourceController.open()
    }

    Shortcut {
        // comando: grafana.get
        // Ctrl+Alt+O de Observabilidade. O Ctrl+Alt+G, que seria o obvio, ja'
        // e' alias do F3 (proxima ocorrencia) — e um atalho que a paleta
        // anuncia e a UI usa para outra coisa e' exatamente o defeito que o
        // `verificar-atalhos.sh` nasceu para pegar.
        sequence: "Ctrl+Alt+O"
        onActivated: root.grafanaController.open()
    }

    Shortcut {
        // comando: probe.list
        // Ctrl+Alt+M de eMbarcados: o Ctrl+Alt+E ja' e' de outra coisa na UI.
        sequence: "Ctrl+Alt+M"
        onActivated: root.embeddedController.open()
    }

    Shortcut {
        // comando: setup.list
        sequence: "Ctrl+Alt+H"
        onActivated: root.setupController.open()
    }

    Shortcut {
        // comando: container.list
        // Ctrl+Alt+W: o C (Containers) e' Continuar do debug, o D (Docker) e'
        // alias do debug.start — as duas letras obvias ja' tem dono na UI, e um
        // atalho que a paleta anuncia e a UI usa para outra coisa e' o defeito
        // que o `verificar-atalhos.sh` pega. W de "whale", o simbolo do Docker.
        sequence: "Ctrl+Alt+W"
        onActivated: root.containerController.open()
    }

    Shortcut {
        // comando: configAction.list
        sequence: "Ctrl+Alt+P"
        onActivated: root.configActionController.openDialog()
    }
}
