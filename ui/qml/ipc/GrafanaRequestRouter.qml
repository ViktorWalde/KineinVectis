import QtQuick

// Leva os pedidos do GrafanaController ao CoreClient.
//
// O TOKEN passa por aqui e nao para em lugar nenhum: ele vai como parametro do
// `grafana.probe` e o cliente o redige antes de escrever o log.
Item {
    id: root

    property var coreClient: null
    property var grafanaController: null

    visible: false

    Connections {
        target: root.grafanaController

        function onGetRequested() {
            root.coreClient.grafanaGet();
        }

        function onSaveRequested(profile) {
            root.coreClient.grafanaSave(profile);
        }

        function onForgetRequested() {
            root.coreClient.grafanaForget();
        }

        function onProbeRequested(token) {
            root.coreClient.grafanaProbe(token);
        }
    }
}
