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
    property bool destroying: false
    property string message: ""
    property bool ok: false
    property string note: ""

    signal destroyRequested(string name, bool data)
    signal closeRequested()

    property bool withData: false

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
                color: Theme.textPrimary
                font.pixelSize: 12
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
            text: qsTr("O perfil sai do projeto. Os dados só vão junto se você marcar abaixo.")
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            KvToggleChip {
                anchors.verticalCenter: parent.verticalCenter
                labelText: root.withData ? qsTr("com os dados") : qsTr("só o perfil")
                active: root.withData
                onToggled: root.withData = !root.withData
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - x
                wrapMode: Text.WordWrap
                text: root.dataLabel
                color: root.withData ? Theme.errorSoft : Theme.textMuted
                font.pixelSize: 10
            }
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
            primary: true
            text: root.withData ? qsTr("Remover perfil E dados") : qsTr("Remover perfil")
            enabled: !root.destroying && root.profileName !== ""
            onClicked: root.destroyRequested(root.profileName, root.withData)
        }
    }
}
