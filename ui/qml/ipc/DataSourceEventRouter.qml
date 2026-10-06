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

        function onDataSourceOdbcSourcesResolved(sources) { root.dataSourceController.odbc.handleSources(sources); }
        function onDataSourceOdbcAuthorized(name, identity, workspace) {
            root.dataSourceController.odbc.handleAuthorized(name, identity, workspace);
        }
        function onDataSourceDriverRequired(method, details) {
            root.dataSourceController.odbc.handleRequired(method, details);
        }

        function onDataSourceListResolved(profiles) {
            root.dataSourceController.handleList(profiles);
        }

        function onDataSourceConsoleResolved(path, created) {
            root.dataSourceController.consoles.handleResolved(path);
        }

        function onDataSourceTested(name, ok, serverVersion, message, secretRequired, clientContext) {
            root.dataSourceController.handleTested(name, ok, serverVersion, message,
                                                   secretRequired, clientContext);
        }

        function onDataSourceIntrospected(name, ok, schemas, collections, message, secretRequired, clientContext) {
            root.dataSourceController.handleIntrospected(name, ok, schemas, collections, message,
                                                         secretRequired, clientContext);
        }

        function onDataSourceQueried(outcome) {
            root.dataSourceController.handleQueried(outcome);
        }

        function onDataSourceOperationFailed(method, message, code, operation) {
            if (method === "datasource.query") root.dataSourceController.handleFailed(method, message, code, operation);
            else if (method === "datasource.test" || method === "datasource.introspect") root.dataSourceController.handleFailed(method, message, code, operation);
            else if (method === "datasource.destroy") root.dataSourceController.discovery.handleFailed(method, message, code, operation);
            else if (method === "datasource.impact") root.dataSourceController.impact.handleFailed(message, operation, code);
        }

        function onDataSourceImpactMeasured(impact) {
            root.dataSourceController.impact.handleMeasured(impact);
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

        function onDataSourceDestroyResolved(profiles, immediate, jobId, command, note, clientContext) {
            root.dataSourceController.discovery.handleDestroyResolved(profiles, immediate, jobId, command, note, clientContext);
        }

        function onDataSourceDestroyed(success, message, profiles, clientContext) {
            root.dataSourceController.discovery.handleDestroyed(success, message, profiles, clientContext);
        }

        function onRequestFailed(method, message, code) {
            if (["datasource.query", "datasource.impact", "datasource.test", "datasource.introspect", "datasource.destroy"].indexOf(method) >= 0) return;
            root.dataSourceController.handleFailed(method, message, code);
            root.dataSourceController.discovery.handleFailed(method, message);
        }
    }
}
