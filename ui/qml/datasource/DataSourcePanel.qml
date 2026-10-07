pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O DIALOGO DA CONEXAO do Banco (2026-10-03), no papel do "Data Sources" da
// JetBrains: a lista de conexoes e do que responde nesta maquina a esquerda,
// e a direita uma face por vez —
//
//   connection  o veredito do teste e o formulario (nova, ou editar)
//   create      um banco novo (SQLite, ou servidor em container)
//   destroy     remover o perfil (e, se pedido, os dados)
//
// Navegar pelas tabelas e escrever SQL nao moram aqui: a janela do Banco fica
// na area da esquerda e o SQL se escreve no editor (console), com o
// resultado no painel de baixo.
//
// Componente burro: recebe por property, pede por signal.
Item {
    id: root

    property var profiles: []
    property string selectedName: ""
    property var draft: null
    property var odbcSources: []
    property bool odbcLoading: false
    property string odbcMessage: ""
    signal odbcRefreshRequested()
    property string errorText: ""

    property bool testing: false
    property bool testOk: false
    property string serverVersion: ""
    property string testMessage: ""
    property bool secretRequired: false
    property bool documentEngine: false
    property string sessionPassword: ""
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
    // A remocao (0.129.0).
    property bool destroying: false
    property string destroyMessage: ""
    property bool destroyOk: false
    property string destroyNote: ""

    signal profileSelected(string name)
    signal candidateSelected(int index)
    signal discoverRequested()
    signal createSqliteRequested(string name, string path)
    signal createServerRequested(string engine, string name, int port)
    signal createDatabaseRequested(string name)
    signal destroyRequested(string name, bool data, var confirmation)
    signal newRequested()
    signal fieldEdited(string field, var value)
    signal passwordEdited(string text)
    signal saveRequested()
    signal testRequested()
    signal secretRetryRequested()
    signal closeRequested()

    property string face: "connection"
    readonly property bool draftNamed: root.draft !== null && root.draft.name !== ""
    readonly property bool savedSelected: root.selectedName !== ""
    readonly property bool draftSaved: root.profiles.some(profile => profile.name === root.selectedName
        && JSON.stringify(DataSourceKinds.cloneProfile(profile)) === JSON.stringify(root.draft))

    KvPanelHeader {
        id: header

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        title: root.face === "create" ? qsTr("Criar banco")
               : (root.savedSelected ? qsTr("Conexão %1").arg(root.selectedName) : qsTr("Conectar banco"))
        subtitle: qsTr("A IDE guarda o perfil, nunca a senha. Um PostgreSQL local por socket conecta sem senha nenhuma.")
        onCloseRequested: root.closeRequested()
    }

    DataSourceList {
        id: places

        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        width: Math.min(240, Math.round(parent.width * 0.3))

        profiles: root.profiles
        selectedName: root.selectedName
        candidates: root.candidates
        discovering: root.discovering
        discoverHint: root.discoverHint

        onProfileSelected: name => { root.profileSelected(name); root.face = "connection"; }
        onCandidateSelected: index => { root.candidateSelected(index); root.face = "connection"; }
        onDiscoverRequested: root.discoverRequested()
        onNewRequested: { root.newRequested(); root.face = "connection"; }
        onCreateRequested: root.face = "create"
    }

    Rectangle {
        anchors.top: places.top
        anchors.bottom: parent.bottom
        anchors.left: places.right
        anchors.leftMargin: Theme.spacingSmall
        width: 1
        color: Theme.borderSoft
    }

    Flickable {
        id: faceScroll

        anchors.top: header.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: places.right
        anchors.leftMargin: Theme.spacingMedium + 1
        anchors.right: parent.right
        anchors.bottom: actions.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        contentWidth: width
        contentHeight: faceColumn.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: faceColumn

            width: faceScroll.width
            spacing: Theme.spacingSmall

            DataSourceVerdict {
                width: parent.width
                visible: root.face === "connection"
                         && (root.testing || root.secretRequired || root.testMessage !== "")
                testing: root.testing
                ok: root.testOk
                serverVersion: root.serverVersion
                message: root.testMessage
                secretRequired: root.secretRequired
                password: root.sessionPassword
                onPasswordEdited: text => root.passwordEdited(text)
                onRetryRequested: root.secretRetryRequested()
            }

            DataSourceForm {
                width: parent.width
                visible: root.face === "connection"
                draft: root.draft
                mongo: root.documentEngine
                odbcSources: root.odbcSources
                odbcLoading: root.odbcLoading
                odbcMessage: root.odbcMessage
                onOdbcRefreshRequested: root.odbcRefreshRequested()
                onFieldEdited: (field, value) => root.fieldEdited(field, value)
            }

            DataSourceCreateBox {
                width: parent.width
                visible: root.face === "create"
                canServe: root.canServe
                containerEngine: root.containerEngine
                creating: root.creating
                command: root.createCommand
                message: root.createMessage
                ok: root.createOk
                serverProfileNamed: root.draftSaved && root.draft !== null && DataSourceKinds.isPostgres(root.draft.engine)
                                    && root.draft.readOnly !== true
                serverProfileName: root.selectedName
                onCreateSqliteRequested: (name, path) => root.createSqliteRequested(name, path)
                onCreateServerRequested: (engine, name, port) => root.createServerRequested(engine, name, port)
                onCreateDatabaseRequested: name => root.createDatabaseRequested(name)
            }

            DataSourceDestroyBox {
                width: parent.width
                visible: root.face === "destroy" && root.savedSelected
                profileName: root.selectedName
                database: root.draft ? root.draft.database : ""
                fileEngine: root.draft ? DataSourceKinds.isSqlite(root.draft.engine) : false
                profileOnly: root.draft ? DataSourceKinds.isOdbc(root.draft.engine) : false
                production: root.draft !== null && root.draft.production === true
                readOnly: root.draft !== null && root.draft.readOnly === true
                documentEngine: root.documentEngine
                destroying: root.destroying
                message: root.destroyMessage
                ok: root.destroyOk
                note: root.destroyNote
                onDestroyRequested: (name, data, confirmation) => root.destroyRequested(name, data, confirmation)
                onCloseRequested: root.face = "connection"
            }

            Text {
                width: parent.width
                visible: root.errorText !== ""
                wrapMode: Text.WordWrap
                text: root.errorText
                color: Theme.errorSoft
                font.pixelSize: Theme.fontSizeCaption
            }
        }
    }

    Row {
        id: actions

        anchors.bottom: parent.bottom
        anchors.left: faceScroll.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall
        visible: root.face === "connection"

        KvButton {
            activeFocusOnTab: true
            compact: true
            text: root.testing ? qsTr("Testando…") : qsTr("Testar")
            enabled: root.draftSaved && !root.testing
            onClicked: root.testRequested()
        }

        KvButton {
            activeFocusOnTab: true
            visible: root.savedSelected
            compact: true
            danger: true
            text: qsTr("Remover…")
            enabled: root.draftSaved
            onClicked: root.face = "destroy"
        }

        Item {
            width: Math.max(0, actions.width - x - saveButton.width - actions.spacing)
            height: 1
        }

        KvButton {
            id: saveButton

            activeFocusOnTab: true
            compact: true
            primary: true
            text: qsTr("Salvar")
            enabled: root.draftNamed
            onClicked: {
                root.saveRequested();
                root.closeRequested();
            }
        }
    }
}
