pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// As fontes de dados na tela: escolher, editar, salvar e TESTAR.
//
// Componente burro: recebe por property, pede por signal. Compoe as quatro
// areas — lista, formulario, veredito e acoes —, cada uma com dono proprio.
//
// "Testar" e' a acao central, e por isso e' a primaria — no cabecalho comum
// (F8): sem ela o autor descobre que o perfil esta errado so' quando for
// usar o banco, longe daqui. O veredito vem ANTES do formulario.
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
    property var collections: []
    property bool documentEngine: false
    property bool reading: false
    property string sessionPassword: ""
    property string sql: ""
    property bool querying: false
    property bool writeConfirmationRequired: false
    property var queryColumns: []
    property var queryRows: []
    property string queryStatus: ""
    // A descoberta e a criacao (0.124.0).
    property var candidates: []
    property bool discovering: false
    property string discoverHint: ""
    property bool canServe: false
    property string containerEngine: ""
    property bool creating: false
    property string createCommand: ""
    property string createMessage: ""
    property bool createOk: false
    property bool createVisible: false

    signal profileSelected(string name)
    signal candidateSelected(int index)
    signal discoverRequested()
    signal createSqliteRequested(string name, string path)
    signal createServerRequested(string engine, string name, int port)
    signal createDatabaseRequested(string name)
    signal newRequested()
    signal fieldEdited(string field, var value)
    signal passwordEdited(string text)
    signal saveRequested()
    signal removeRequested()
    signal testRequested()
    signal introspectRequested()
    signal sqlEdited(string text)
    signal queryRequested(bool confirmWrite)
    signal closeRequested()

    readonly property bool draftNamed: root.draft !== null && root.draft.name !== ""

    // A primeira linha comum dos paineis de ambiente (F8): titulo, uma
    // linha, a acao primaria — "Testar" — e o x.
    KvPanelHeader {
        id: cabecalho

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: qsTr("Banco de dados")
        subtitle: qsTr("A IDE guarda o perfil, nunca a senha. Um PostgreSQL local por socket conecta sem senha nenhuma.")
        primaryLabel: qsTr("Testar")
        primaryEnabled: root.draftNamed
        primaryBusy: root.testing
        onPrimaryRequested: root.testRequested()
        onCloseRequested: root.closeRequested()
    }

    DataSourceList {
        id: lista

        anchors.top: cabecalho.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.bottom: acoes.top
        anchors.bottomMargin: Theme.spacingSmall
        width: Math.round(parent.width * 0.38)

        profiles: root.profiles
        selectedName: root.selectedName
        candidates: root.candidates
        discovering: root.discovering
        discoverHint: root.discoverHint

        onProfileSelected: name => root.profileSelected(name)
        onCandidateSelected: index => root.candidateSelected(index)
        onDiscoverRequested: root.discoverRequested()
        onNewRequested: root.newRequested()
        onCreateRequested: root.createVisible = !root.createVisible
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

            DataSourceCreateBox {
                width: parent.width
                visible: root.createVisible
                canServe: root.canServe
                containerEngine: root.containerEngine
                creating: root.creating
                command: root.createCommand
                message: root.createMessage
                ok: root.createOk
                serverProfileNamed: root.selectedName !== "" && root.draft !== null && root.draft.engine === "postgres"
                serverProfileName: root.selectedName
                onCreateSqliteRequested: (name, path) => root.createSqliteRequested(name, path)
                onCreateServerRequested: (engine, name, port) => root.createServerRequested(engine, name, port)
                onCreateDatabaseRequested: name => root.createDatabaseRequested(name)
                onCloseRequested: root.createVisible = false
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

            DataSourceForm {
                width: parent.width
                draft: root.draft
                mongo: root.documentEngine
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
            }

            // A consulta (0.121.0) fica ENTRE o veredito e a estrutura: a
            // estrutura e' o que se le para escrever a consulta.
            DataSourceQuery {
                width: parent.width
                sql: root.sql
                documentEngine: root.documentEngine
                querying: root.querying
                writeConfirmationRequired: root.writeConfirmationRequired
                columns: root.queryColumns
                rows: root.queryRows
                status: root.queryStatus
                canRun: root.draftNamed
                onSqlEdited: text => root.sqlEdited(text)
                onRunRequested: confirmWrite => root.queryRequested(confirmWrite)
            }

            // DUAS FORMAS, NUNCA AS DUAS AO MESMO TEMPO. A visao e' escolhida
            // pelo MOTOR, e nao por "qual lista veio vazia": uma coleção que
            // de fato nao tem campo nenhum continua sendo Mongo.
            DataSourceStructure {
                width: parent.width
                visible: !root.documentEngine
                schemas: root.schemas
                loading: root.reading && !root.documentEngine
            }

            DataSourceCollections {
                width: parent.width
                visible: root.documentEngine
                collections: root.collections
                loading: root.reading && root.documentEngine
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
            text: qsTr("Ler estrutura")
            compact: true
            enabled: root.draftNamed && !root.reading
            onClicked: root.introspectRequested()
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
