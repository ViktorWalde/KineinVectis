pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// "Remover" (0.129.0): o perfil sai; com "apagar tambem os dados" vai junto
// o arquivo SQLite (so' dentro do projeto), o container `kinein-<nome>` ou
// o banco dentro do PostgreSQL (DROP DATABASE). O que NAO se apaga e' dito
// antes do clique. Burro: estado por property, intencao por signal.
Item {
    id: root

    property string profileName: ""
    property string database: ""
    // Os fatos vem de fora (o controller ja' os deriva): arquivo ou documento.
    property bool fileEngine: false
    property bool documentEngine: false
    property bool profileOnly: false
    property bool production: false
    property bool readOnly: false
    property bool destroying: false
    property string message: ""
    property bool ok: false
    property string note: ""

    signal destroyRequested(string name, bool data, var confirmation)
    signal closeRequested()

    property bool withData: false
    property string typedConnection: ""
    property string typedDatabase: ""
    readonly property bool removesData: root.withData && !root.profileOnly && !root.readOnly
    readonly property bool canRemove: !root.destroying && root.profileName !== ""
        && (!root.removesData || !root.production || root.typedConnection === root.profileName && root.typedDatabase === root.database)
    onProfileNameChanged: { root.withData = false; root.typedConnection = ""; root.typedDatabase = ""; }
    onVisibleChanged: { root.withData = false; root.typedConnection = ""; root.typedDatabase = ""; }
    onReadOnlyChanged: if (root.readOnly) root.withData = false

    readonly property string dataLabel: {
        if (fileEngine) return qsTr("apagar também o arquivo %1 (só se estiver dentro do projeto)").arg(database);
        if (documentEngine) return qsTr("apagar também os dados — no MongoDB a IDE só remove um servidor que ela mesma subiu (container kinein-%1)").arg(profileName);
        return qsTr("apagar também os dados — o container kinein-%1 se existir, ou DROP DATABASE \"%2\" no servidor").arg(profileName).arg(database);
    }

    implicitHeight: coluna.implicitHeight + 2 * Theme.spacingSmall

    Rectangle {
        anchors.fill: parent
        radius: Theme.radius
        color: Theme.background2
        border.width: 1
        border.color: Theme.errorSoft
        opacity: 0.95
    }

    Column {
        id: coluna

        x: Theme.spacingSmall
        y: Theme.spacingSmall
        width: parent.width - 2 * Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Remover %1").arg(root.profileName)
                textFormat: Text.PlainText
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeBody
                font.bold: true
            }

            Item { width: parent.width - x - fechar.width; height: 1 }

            KvIconButton {
                id: fechar

                compact: true
                iconName: "close"
                tooltip: qsTr("Cancelar")
                onClicked: root.closeRequested()
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: root.profileOnly ? qsTr("O perfil sai do projeto. O DSN, o driver e os dados permanecem no sistema.")
                                  : qsTr("O perfil sai do projeto. Os dados só vão junto se você marcar abaixo.")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        KvSegmentedControl {
            width: parent.width
            visible: !root.profileOnly
            current: root.removesData ? "data" : "profile"
            options: [ { value: "profile", label: qsTr("Só o perfil") },
                       { value: "data", label: qsTr("Perfil e dados"), enabled: !root.readOnly } ]
            onSelected: value => root.withData = value === "data"
        }

        Text {
            width: parent.width
            visible: !root.profileOnly
            wrapMode: Text.WordWrap
            textFormat: Text.PlainText
            text: root.readOnly ? qsTr("Somente leitura: a IDE permite remover o perfil, mas recusa apagar os dados.") : root.dataLabel
            color: root.removesData ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        Text {
            width: parent.width
            visible: root.production && root.removesData
            text: qsTr("PRODUÇÃO — os dados serão apagados. Confira os dois nomes.")
            wrapMode: Text.WordWrap
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeSmall
        }

        DataSourceField {
            width: parent.width
            visible: root.production && root.removesData
            label: qsTr("Nome da conexão: %1").arg(root.profileName)
            value: root.typedConnection
            onEdited: text => root.typedConnection = text
        }

        DataSourceField {
            width: parent.width
            visible: root.production && root.removesData
            label: qsTr("Banco ou arquivo: %1").arg(root.database)
            value: root.typedDatabase
            onEdited: text => root.typedDatabase = text
            onAccepted: if (root.canRemove) root.destroyRequested(root.profileName, root.removesData,
                { connection: root.typedConnection, target: root.typedDatabase })
        }

        KvVerdict {
            width: parent.width
            busy: root.destroying
            busyText: root.message !== "" ? root.message : qsTr("removendo…")
            ok: root.ok
            neutral: root.note !== "" && root.ok
            message: root.note !== "" ? root.note : root.message
        }

        KvButton {
            compact: true
            activeFocusOnTab: true
            danger: root.removesData
            primary: !root.removesData
            text: root.removesData ? qsTr("Remover perfil e dados") : qsTr("Remover perfil")
            enabled: root.canRemove
            onClicked: root.destroyRequested(root.profileName, root.removesData,
                { connection: root.typedConnection, target: root.typedDatabase })
        }
    }
}
