pragma ComponentBehavior: Bound
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
    // O dialogo da conexao do Banco (2026-10-03): a janela do Banco mora na
    // area da esquerda; criar e editar conexao abre aqui, como o "Data
    // Sources" da JetBrains.
    property var dataSourceController: null
    property var embeddedController: null
    property var setupController: null
    // Nao e' painel de ambiente: e' quem recebe o plano que a biblioteca produz.
    property var configActionController: null
    // Nao e' painel de ambiente: e' o terminal onde o comando de instalacao
    // aparece. Nada roda escondido.
    property var runtimeController: null

    Repeater {
        // O modelo e' a CONTAGEM, nao a lista. `overlayEntries` vem de
        // `ToolWindows.entries`, que se refaz a cada mudanca de estado (um
        // `panelVisible`, a aba de baixo, o recolher dos Simbolos); com a
        // lista como modelo, o Repeater destruia e recriava os cinco paineis
        // a cada vez — ~45 ms e o que estivesse digitado num painel aberto
        // (medido em 2026-10-03, 40.7 §7.195). Um numero igual nao reinicia o
        // modelo: cada vaga cria o painel uma vez e le' o `active` pelo indice.
        model: root.toolWindows === null ? 0 : root.toolWindows.overlayEntries.length

        // `createObject`, e NAO `Loader`: os paineis sao componentes "bound"
        // do `ToolWindows`, e o Loader do Qt 6.4 (o do AppImage, Debian 12) os
        // instancia no contexto DELE e recusa ("Cannot instantiate bound
        // component outside its creation context") — os cinco paineis de
        // ambiente nao abriam no pacote, so' no checkout com Qt mais novo.
        // `createObject` usa o contexto de criacao do componente nas duas.
        // Criado na PRIMEIRA abertura e guardado (o estado do painel fica):
        // criar os cinco na abertura da IDE pesava no primeiro quadro
        // (qmlprofiler, 40.7 §7.203).
        Item {
            id: panelSlot

            required property int index
            readonly property var entry: root.toolWindows === null
                                         ? undefined : root.toolWindows.overlayEntries[panelSlot.index]
            property Item panel: null

            anchors.fill: parent
            z: 99
            visible: panelSlot.entry !== undefined && panelSlot.entry.active === true

            function ensurePanel() {
                if (panelSlot.panel !== null || !panelSlot.visible) return;
                const panel = panelSlot.entry.panel.createObject(panelSlot);
                panel.width = Qt.binding(() => panelSlot.width);
                panel.height = Qt.binding(() => panelSlot.height);
                panelSlot.panel = panel;
            }

            onVisibleChanged: panelSlot.ensurePanel()
            Component.onCompleted: panelSlot.ensurePanel()
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

    DataSourcePanelHost {
        anchors.fill: parent
        visible: root.dataSourceController !== null && root.dataSourceController.panelVisible
        z: 99
        controller: root.dataSourceController
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.dataSourceController.close()
    }

    // A pergunta antes de uma escrita, com o impacto medido (0.150.0): por
    // cima de tudo, inclusive do dialogo da conexao.
    SqlImpactDialog {
        anchors.fill: parent
        visible: root.dataSourceController !== null && root.dataSourceController.impact.open
        z: 100
        impact: root.dataSourceController ? root.dataSourceController.impact : null
        engineLabel: {
            if (root.dataSourceController === null) return "";
            const name = root.dataSourceController.impact.name;
            const profile = root.dataSourceController.profiles.find(p => p.name === name);
            return profile === undefined ? "" : DataSourceKinds.engineName(profile.engine);
        }
        maxAvailableWidth: root.hostWidth - 4 * Theme.spacingMedium
        maxAvailableHeight: root.hostHeight - 4 * Theme.spacingMedium
        onDismissRequested: root.dataSourceController.impact.cancel()
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
