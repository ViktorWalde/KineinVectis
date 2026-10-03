pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os paineis de ambiente que as entradas do trilho abrem (Embarcados, Banco,
// Containers, Remoto, Observabilidade). Saiu do ToolWindows na 0.3.9: la' fica
// QUAIS areas existem e o que cada uma faz; aqui, COMO cada painel nasce. Quem
// cria os paineis e' o ShellEnvironmentOverlays (createObject, uma vez).
QtObject {
    id: root

    property var embeddedController: null
    property var toolchainController: null
    property var dataSourceController: null
    property var containerController: null
    property var remoteController: null
    property var grafanaController: null

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
}
