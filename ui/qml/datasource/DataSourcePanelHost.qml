pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O dialogo da conexao do Banco na moldura comum (KvPanelFrame, F8): o
// CONTEUDO (DataSourcePanel, burro) separado do CHROME.
KvPanelFrame {
    id: root

    property var controller: null

    panelWidth: 820
    panelHeight: 600

    DataSourcePanel {
        anchors.fill: parent

        profiles: root.controller ? root.controller.profiles : []
        odbcSources: root.controller ? root.controller.odbc.sources : []
        odbcLoading: root.controller ? root.controller.odbc.loading : false
        odbcMessage: root.controller ? root.controller.odbc.message : ""
        onOdbcRefreshRequested: root.controller.odbc.refresh()
        selectedName: root.controller ? root.controller.selectedName : ""
        draft: root.controller ? root.controller.draft : null
        errorText: root.controller ? root.controller.errorText : ""
        testing: root.controller ? root.controller.testing : false
        testOk: root.controller ? root.controller.testOk : false
        serverVersion: root.controller ? root.controller.serverVersion : ""
        testMessage: root.controller ? root.controller.testMessage : ""
        secretRequired: root.controller ? root.controller.secretRequired : false
        documentEngine: root.controller ? root.controller.documentEngine : false
        sessionPassword: root.controller ? root.controller.sessionPassword : ""
        candidates: root.controller ? root.controller.discovery.candidates : []
        discovering: root.controller ? root.controller.discovery.discovering : false
        discoverHint: root.controller ? root.controller.discovery.hint : ""
        canServe: root.controller ? root.controller.discovery.canServe : false
        containerEngine: root.controller ? root.controller.discovery.containerEngine : ""
        creating: root.controller ? root.controller.discovery.creating : false
        createCommand: root.controller ? root.controller.discovery.createCommand : ""
        createMessage: root.controller ? root.controller.discovery.createMessage : ""
        createOk: root.controller ? root.controller.discovery.createOk : false
        destroying: root.controller ? root.controller.discovery.destroying : false
        destroyMessage: root.controller ? root.controller.discovery.destroyMessage : ""
        destroyOk: root.controller ? root.controller.discovery.destroyOk : false
        destroyNote: root.controller ? root.controller.discovery.destroyNote : ""

        onProfileSelected: name => root.controller.select(name)
        onCandidateSelected: index => root.controller.discovery.adopt(index)
        onDiscoverRequested: root.controller.discovery.discover()
        onCreateSqliteRequested: (name, path) => root.controller.discovery.createSqlite(name, path)
        onCreateServerRequested: (engine, name, port) => root.controller.discovery.createServer(engine, name, port)
        onCreateDatabaseRequested: name => root.controller.createDatabaseOnServer(name)
        onDestroyRequested: (name, data) => root.controller.discovery.destroyProfile(name, data)
        onNewRequested: root.controller.startNew()
        onFieldEdited: (field, value) => root.controller.editDraft(field, value)
        onPasswordEdited: text => root.controller.sessionPassword = text
        onSaveRequested: root.controller.save()
        onTestRequested: root.controller.test()
        onCloseRequested: root.dismissRequested()
    }
}
