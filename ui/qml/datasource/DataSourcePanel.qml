pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As fontes de dados na tela: escolher, editar, salvar e TESTAR.
//
// Componente burro: recebe por property, pede por signal. Compoe as quatro
// areas — lista, formulario, veredito e acoes —, cada uma com dono proprio.
//
// "Testar" e' a acao central, e por isso e' a primaria: sem ela o autor
// descobre que o perfil esta errado so' quando for usar o banco, longe daqui.
Item {
    id: root

    property var profiles: []
    property string selectedName: ""
    property var draft: null
    property string errorText: ""

    property bool testing: false
    property bool testOk: false
    property string serverVersion: ""
    property string testMessage: ""
    property bool secretRequired: false
    property var schemas: []
    property bool reading: false
    property string sessionPassword: ""

    signal profileSelected(string name)
    signal newRequested()
    signal fieldEdited(string field, var value)
    signal passwordEdited(string text)
    signal saveRequested()
    signal removeRequested()
    signal testRequested()
    signal introspectRequested()
    signal closeRequested()

    readonly property bool draftNamed: root.draft !== null && root.draft.name !== ""

    Text {
        id: titulo

        anchors.top: parent.top
        anchors.left: parent.left
        text: qsTr("Banco de dados")
        color: Theme.textPrimary
        font.pixelSize: 12
        font.weight: Font.DemiBold
    }

    Text {
        id: subtitulo

        anchors.top: titulo.bottom
        anchors.topMargin: 2
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("A IDE guarda o perfil, nunca a senha. Um PostgreSQL local "
                   + "por socket conecta sem senha nenhuma.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    DataSourceList {
        id: lista

        anchors.top: subtitulo.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.bottom: acoes.top
        anchors.bottomMargin: Theme.spacingSmall
        width: Math.round(parent.width * 0.38)

        profiles: root.profiles
        selectedName: root.selectedName

        onProfileSelected: name => root.profileSelected(name)
        onNewRequested: root.newRequested()
    }

    Flickable {
        id: rolagem

        anchors.top: lista.top
        anchors.left: lista.right
        anchors.leftMargin: Theme.spacingMedium
        anchors.right: parent.right
        anchors.bottom: lista.bottom
        clip: true
        contentWidth: width
        contentHeight: conteudo.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: conteudo

            width: rolagem.width
            spacing: Theme.spacingSmall

            DataSourceForm {
                width: parent.width
                draft: root.draft
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
            }

            DataSourceVerdict {
                width: parent.width
                testing: root.testing
                ok: root.testOk
                serverVersion: root.serverVersion
                message: root.testMessage
                secretRequired: root.secretRequired
                password: root.sessionPassword

                onPasswordEdited: text => root.passwordEdited(text)
                onRetryRequested: root.testRequested()
            }

            DataSourceStructure {
                width: parent.width
                schemas: root.schemas
                loading: root.reading
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                wrapMode: Text.WordWrap
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: 10
            }
        }
    }

    Row {
        id: acoes

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall
        layoutDirection: Qt.RightToLeft

        KvButton {
            text: qsTr("Fechar")
            compact: true
            onClicked: root.closeRequested()
        }

        KvButton {
            text: qsTr("Ler estrutura")
            compact: true
            enabled: root.draftNamed && !root.reading
            onClicked: root.introspectRequested()
        }

        KvButton {
            text: qsTr("Testar")
            primary: true
            compact: true
            enabled: root.draftNamed && !root.testing
            onClicked: root.testRequested()
        }

        KvButton {
            text: qsTr("Salvar")
            compact: true
            enabled: root.draftNamed
            onClicked: root.saveRequested()
        }

        KvButton {
            text: qsTr("Remover")
            compact: true
            enabled: root.selectedName !== ""
            onClicked: root.removeRequested()
        }
    }
}
