import QtQuick

// Roteia as respostas e o evento de `grafana.*` para o GrafanaController.
//
// Inclui o `requestFailed` porque neste dominio a recusa e' informacao de
// produto: "este Grafana pede o token a cada sessao" e' o que faz a UI abrir o
// campo — e ela decide isso pelo CODIGO, nunca pelo texto.
Item {
    id: root

    property var coreClient: null
    property var grafanaController: null

    visible: false

    Connections {
        target: root.coreClient

        function onGrafanaProfileResolved(profile, exists) {
            root.grafanaController.handleProfile(profile, exists);
        }

        function onGrafanaProbed(result) {
            root.grafanaController.handleProbed(result);
        }

        function onRequestFailed(method, message, code) {
            root.grafanaController.handleFailed(method, message, code);
        }
    }
}
