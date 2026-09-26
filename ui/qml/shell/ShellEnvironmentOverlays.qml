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
    // As tool windows do trilho trazem o proprio painel (fatia V3, campo
    // `componente`): aqui fica so' o que e' igual em todos — ancora, z e a
    // folga da janela. Acrescentar uma janela nao toca mais neste arquivo.
    property var toolWindows: null
    property var libraryController: null
    property var embeddedController: null
    property var setupController: null
    // Nao e' painel de ambiente: e' quem recebe o plano que a biblioteca produz.
    property var configActionController: null
    // Nao e' painel de ambiente: e' o terminal onde o comando de instalacao
    // aparece. Nada roda escondido.
    property var runtimeController: null

    Repeater {
        model: root.toolWindows === null ? [] : root.toolWindows.overlayEntries

        Loader {
            required property var modelData

            anchors.fill: parent
            z: 99
            sourceComponent: modelData.panel
            // `visible`, e nao `active`: carregar o painel so' ao abrir seria
            // ganho de memoria, mas muda o ciclo de vida de cinco paineis —
            // decisao de outra fatia, nao contrabando desta.
            visible: modelData.active === true
        }
    }

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





    // O passo de permissao (E2) vai para o TERMINAL DA IDE, visivel, pelo
    // mesmo caminho do painel de instalacao. Nada roda escondido.
    Connections {
        target: root.embeddedController ? root.embeddedController.access : null

        function onCommandRequested(comando) {
            root.runtimeController.submitShellInput(comando);
        }
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
