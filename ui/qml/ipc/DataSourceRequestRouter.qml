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
}
