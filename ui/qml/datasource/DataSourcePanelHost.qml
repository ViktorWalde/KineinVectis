pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de fontes de dados dentro de um dialogo, como os outros overlays.
//
// Separa o CONTEUDO (DataSourcePanel, burro) do CHROME (fundo, moldura,
// dispensar por clique fora). Mesmo desenho do LibraryPanelHost.
Item {
    id: root

    property var controller: null
    property real maxAvailableWidth: 720
    property real maxAvailableHeight: 520

    signal dismissRequested()

    // Clique fora dispensa. Dentro, nao — senao escolher um perfil fecharia
    // o painel.
    MouseArea {
        anchors.fill: parent
        onClicked: root.dismissRequested()
    }

    Rectangle {
        id: moldura

        anchors.centerIn: parent
        width: Math.min(620, root.maxAvailableWidth)
        height: Math.min(480, root.maxAvailableHeight)
        radius: Theme.radius
        color: Theme.background1
        border.width: 1
        border.color: Theme.borderStrong

        MouseArea {
            anchors.fill: parent
        }

        DataSourcePanel {
            anchors.fill: parent
            anchors.margins: Theme.spacingMedium

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

            onProfileSelected: name => root.controller.select(name)
            onNewRequested: root.controller.startNew()
            onFieldEdited: (field, value) => root.controller.editDraft(field, value)
            onPasswordEdited: text => root.controller.sessionPassword = text
            onSaveRequested: root.controller.save()
            onRemoveRequested: root.controller.remove()
            onTestRequested: root.controller.test()
            onIntrospectRequested: root.controller.introspect()
            onCloseRequested: root.dismissRequested()
        }
    }
}
