import QtQuick

// Espelho do DataSourceEventRouter: leva ao core o que o controller pede.
//
// A senha da sessao passa por aqui uma vez, no `datasource.test`, e nao fica.
Item {
    id: root

    property var coreClient: null
    property var dataSourceController: null

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

        function onTestRequested(name, password) {
            root.coreClient.dataSourceTest(name, password);
        }

        function onIntrospectRequested(name, password) {
            root.coreClient.dataSourceIntrospect(name, password);
        }

        function onQueryRequested(name, password, sql, confirmWrite) {
            root.coreClient.dataSourceQuery(name, password, sql, 0, confirmWrite);
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

        function onDestroyRequested(name, data) {
            root.coreClient.dataSourceDestroy(name, data);
        }
    }
}
