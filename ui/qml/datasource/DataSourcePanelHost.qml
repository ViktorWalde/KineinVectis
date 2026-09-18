pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de fontes de dados na moldura comum (KvPanelFrame, F8): o
// CONTEUDO (DataSourcePanel, burro) separado do CHROME.
KvPanelFrame {
    id: root

    property var controller: null

    panelWidth: 720
    panelHeight: 560

    DataSourcePanel {
        anchors.fill: parent

        profiles: root.controller ? root.controller.profiles : []
        selectedName: root.controller ? root.controller.selectedName : ""
        draft: root.controller ? root.controller.draft : null
        errorText: root.controller ? root.controller.errorText : ""
        testing: root.controller ? root.controller.testing : false
        testOk: root.controller ? root.controller.testOk : false
        serverVersion: root.controller ? root.controller.serverVersion : ""
        testMessage: root.controller ? root.controller.testMessage : ""
        secretRequired: root.controller ? root.controller.secretRequired : false
        schemas: root.controller ? root.controller.schemas : []
        collections: root.controller ? root.controller.collections : []
        documentEngine: root.controller ? root.controller.documentEngine : false
        reading: root.controller ? root.controller.reading : false
        sessionPassword: root.controller ? root.controller.sessionPassword : ""
        sql: root.controller ? root.controller.sql : ""
        querying: root.controller ? root.controller.querying : false
        writeConfirmationRequired: root.controller ? root.controller.writeConfirmationRequired : false
        queryColumns: root.controller ? root.controller.queryColumns : []
        queryRows: root.controller ? root.controller.queryRows : []
        queryStatus: root.controller ? root.controller.queryStatus : ""

        onProfileSelected: name => root.controller.select(name)
        onNewRequested: root.controller.startNew()
        onFieldEdited: (field, value) => root.controller.editDraft(field, value)
        onPasswordEdited: text => root.controller.sessionPassword = text
        onSaveRequested: root.controller.save()
        onRemoveRequested: root.controller.remove()
        onTestRequested: root.controller.test()
        onIntrospectRequested: root.controller.introspect()
        onSqlEdited: text => root.controller.sql = text
        onQueryRequested: confirmWrite => root.controller.runQuery(confirmWrite)
        onCloseRequested: root.dismissRequested()
    }
}
