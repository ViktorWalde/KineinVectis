import QtQuick

// Espelho do DataSourceEventRouter: leva ao core o que o controller pede.
//
// A senha da sessao passa por aqui uma vez, no `datasource.test`, e nao fica.
Item {
    id: root

    property var coreClient: null
    property var dataSourceController: null
    property var editorAppendController: null

    visible: false

    Connections {
        target: root.dataSourceController

        function onListRequested() {
            root.coreClient.dataSourceList();
        }

        function onSaveRequested(profile) {
            root.coreClient.dataSourceSave(profile);
        }

        function onRemoveRequested(name) {
            root.coreClient.dataSourceRemove(name);
        }

        function onTestRequested(name, password, context) {
            root.coreClient.dataSourceTest(name, password, context);
        }

        function onIntrospectRequested(name, password, context) {
            root.coreClient.dataSourceIntrospect(name, password, context);
        }

        function onQueryRequested(name, password, sql, confirmWrite, maxRows, context, confirmation) {
            root.coreClient.dataSourceQuery(name, password, sql, maxRows, confirmWrite, context, confirmation);
        }
    }

    Connections {
        target: root.dataSourceController ? root.dataSourceController.previews : null
        function onDecisionRequested(operation) { root.coreClient.dataSourcePreviewDecide(operation); }
        function onCancelExecutionRequested(jobId) { root.coreClient.cancelJob(jobId); }
    }

    Connections {
        target: root.dataSourceController ? root.dataSourceController.sessions : null
        function onRequested(name, context) { root.coreClient.dataSourceDisconnect(name, context); }
    }

    Connections {
        target: root.dataSourceController ? root.dataSourceController.odbc : null

        function onSourcesRequested() { root.coreClient.dataSourceOdbcSources(); }
        function onAuthorizeRequested(name, identity, workspace) {
            root.coreClient.dataSourceOdbcAuthorize(name, identity, workspace);
        }
    }

    // O console no editor (0.149.0) e' do filho `consoles`: o core garante o
    // arquivo; abrir e' o mesmo `fs.read` que a arvore usa para abrir arquivo.
    Connections {
        target: root.dataSourceController ? root.dataSourceController.consoles : null

        function onConsoleRequested(name, context) {
            root.coreClient.dataSourceConsole(name, context);
        }

        function onStatementRequested(operation) { root.coreClient.dataSourceConsoleStatement(operation); }

        function onOpenFileRequested(path) {
            root.editorAppendController.request(path, "");
        }
        function onAppendRequested(path, text, operation) { root.editorAppendController.request(path, text, operation); }
    }

    // O impacto de uma escrita (0.150.0) e' do filho `impact`: medir antes de
    // pedir a confirmacao, com a senha da sessao.
    Connections {
        target: root.dataSourceController ? root.dataSourceController.impact : null

        function onImpactRequested(name, sql, context) {
            root.coreClient.dataSourceImpact(name, root.dataSourceController.passwordFor(name), sql, context);
        }
    }

    // A descoberta e a criacao (0.124.0) sao do controller FILHO `discovery`
    // — ligar ao pai seria a Connections sem sinal que o gate binario-abre
    // passou a reprovar (40 §7.63).
    Connections {
        target: root.dataSourceController ? root.dataSourceController.discovery : null

        function onDiscoverRequested() {
            root.coreClient.dataSourceDiscover();
        }

        function onCreateSqliteRequested(name, path) {
            root.coreClient.dataSourceCreateSqlite(name, path);
        }

        function onCreateServerRequested(engine, name, port) {
            root.coreClient.dataSourceCreateServer(engine, name, port);
        }

        function onDestroyRequested(name, data, context, confirmation) {
            root.coreClient.dataSourceDestroy(name, data, root.dataSourceController.passwordFor(name), context, confirmation);
        }
    }
}
