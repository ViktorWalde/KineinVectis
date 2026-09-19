import QtQuick

// Roteia as respostas de `datasource.*` do CoreClient para o controller.
//
// Inclui o `requestFailed` porque neste dominio a recusa e' informacao de
// produto: "informe o host" e "nao ha perfil chamado X" sao coisas que o autor
// precisa ler, nao erros internos.
//
// O veredito do teste chega por EVENTO, nao por resposta: conectar espera a
// rede, entao o pedido devolve um job e o resultado vem depois
// (`arquitetura/04` §5).
Item {
    id: root

    property var coreClient: null
    property var dataSourceController: null

    visible: false

    Connections {
        target: root.coreClient

        function onDataSourceListResolved(profiles) {
            root.dataSourceController.handleList(profiles);
        }

        function onDataSourceTested(name, ok, serverVersion, message, secretRequired) {
            root.dataSourceController.handleTested(name, ok, serverVersion, message,
                                                   secretRequired);
        }

        function onDataSourceIntrospected(name, ok, schemas, collections, message, secretRequired) {
            root.dataSourceController.handleIntrospected(name, ok, schemas, collections, message,
                                                         secretRequired);
        }

        function onDataSourceQueried(outcome) {
            root.dataSourceController.handleQueried(outcome);
        }

        function onDataSourceDiscovered(candidates, containerEngine, hint) {
            root.dataSourceController.discovery.handleDiscovered(candidates, containerEngine, hint);
        }

        function onDataSourceCreateResolved(profile, jobId, command) {
            root.dataSourceController.discovery.handleCreateResolved(profile, jobId, command);
        }

        function onDataSourceCreated(success, profile, message) {
            root.dataSourceController.discovery.handleCreated(success, profile, message);
        }

        function onDataSourceDestroyResolved(profiles, immediate, jobId, command, note) {
            root.dataSourceController.discovery.handleDestroyResolved(profiles, immediate, jobId, command, note);
        }

        function onDataSourceDestroyed(success, message, profiles) {
            root.dataSourceController.discovery.handleDestroyed(success, message, profiles);
        }

        function onRequestFailed(method, message, code) {
            root.dataSourceController.handleFailed(method, message, code);
            root.dataSourceController.discovery.handleFailed(method, message);
        }
    }
}
