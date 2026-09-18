import QtQuick
import KineinVectis

// Os sete paineis de AMBIENTE DO PROJETO: bibliotecas, banco, alvo remoto
// (SSH, 2026-09-17), observabilidade, embarcados, containers e instalacao de
// ferramentas (a simulacao saiu do produto em 2026-09-12).
//
// Nasceu em 2026-09-05, quando a catraca reprovou o `ShellOverlays` ao ganhar
// mais um painel. O corte e' por RESPONSABILIDADE: todos tem a mesma
// forma — moldura de dialogo sobre um controller com `panelVisible`, mesmo
// ciclo abrir/fechar, mesma folga de janela — e sao o agrupamento que o menu
// ja' chama de "Ambiente do projeto". Cortar por tamanho teria juntado coisas
// que nao se parecem.
Item {
    id: root

    property real hostWidth: 0
    property real hostHeight: 0
    property var libraryController: null
    property var dataSourceController: null
    property var remoteController: null
    property var grafanaController: null
    property var embeddedController: null
    // O painel de embarcados edita o KIT (chip, alvo, depurador), que mora aqui.
    property var toolchainController: null
    property var setupController: null
    property var containerController: null
    // Nao e' painel de ambiente: e' quem recebe o plano que a biblioteca produz.
    property var configActionController: null
    // Nao e' painel de ambiente: e' o terminal onde o comando de instalacao
    // aparece. Nada roda escondido.
    property var runtimeController: null

    LibraryPanelHost {
        anchors.fill: parent
        visible: root.libraryController.panelVisible
        z: 99
        controller: root.libraryController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.libraryController.close()
        onApplyStepRequested: function(actionId, params) {
            root.libraryController.close();
            root.configActionController.openWith(actionId, params);
        }
    }

    DataSourcePanelHost {
        anchors.fill: parent
        visible: root.dataSourceController.panelVisible
        z: 99
        controller: root.dataSourceController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.dataSourceController.close()
    }

    RemotePanelHost {
        anchors.fill: parent
        visible: root.remoteController.panelVisible
        z: 99
        controller: root.remoteController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.remoteController.close()
    }

    GrafanaPanelHost {
        anchors.fill: parent
        visible: root.grafanaController.panelVisible
        z: 99
        controller: root.grafanaController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.grafanaController.close()
    }

    EmbeddedPanelHost {
        anchors.fill: parent
        visible: root.embeddedController.panelVisible
        z: 99
        controller: root.embeddedController
        toolchainController: root.toolchainController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.embeddedController.close()
    }

    // O passo de permissao (E2) vai para o TERMINAL DA IDE, visivel, pelo
    // mesmo caminho do painel de instalacao. Nada roda escondido.
    Connections {
        target: root.embeddedController ? root.embeddedController.access : null

        function onCommandRequested(comando) {
            root.runtimeController.submitShellInput(comando);
        }
    }

    ContainerPanelHost {
        anchors.fill: parent
        visible: root.containerController.panelVisible
        z: 99
        controller: root.containerController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.containerController.close()
    }

    SetupPanelHost {
        anchors.fill: parent
        visible: root.setupController.panelVisible
        z: 99
        controller: root.setupController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.setupController.close()
        // O comando vai para o TERMINAL DA IDE, visivel. Nada roda escondido.
        onCommandRequested: comando => root.runtimeController.submitShellInput(comando)
    }
}
