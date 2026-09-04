pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O formulario do perfil. Burro: recebe o rascunho, devolve edicoes.
//
// NAO HA' CAMPO DE SENHA AQUI, e e' deliberado. O que se escolhe neste
// formulario e' DE ONDE a senha vem, nunca qual e' ela — o perfil e' o que o
// core persiste, e senha nao vai para o disco (`docs/seguranca/40`). O campo
// de senha, quando aparece, vive no veredito do teste e some com a sessao.
Item {
    id: root

    property var draft: null

    signal fieldEdited(string field, var value)

    readonly property string secretSource:
        root.draft ? (root.draft.secretSource || "automatic") : "automatic"

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        DataSourceField {
            width: parent.width
            label: qsTr("Nome")
            placeholder: qsTr("como a IDE vai chamar esta fonte")
            value: root.draft ? root.draft.name : ""
            onEdited: text => root.fieldEdited("name", text)
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Host ou diretório de socket")
            placeholder: qsTr("/var/run/postgresql, ou db.exemplo.com")
            value: root.draft ? root.draft.host : ""
            onEdited: text => root.fieldEdited("host", text)
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: (parent.width - Theme.spacingSmall) / 3
                label: qsTr("Porta")
                numeric: true
                value: root.draft ? String(root.draft.port) : ""
                onEdited: text => root.fieldEdited("port", parseInt(text, 10) || 0)
            }

            DataSourceField {
                width: (parent.width - Theme.spacingSmall) * 2 / 3
                label: qsTr("Banco")
                value: root.draft ? root.draft.database : ""
                onEdited: text => root.fieldEdited("database", text)
            }
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Usuário")
            placeholder: qsTr("o papel que conecta")
            value: root.draft ? root.draft.user : ""
            onEdited: text => root.fieldEdited("user", text)
        }

        Text {
            width: parent.width
            text: qsTr("De onde vem a senha")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            spacing: Theme.spacingXSmall

            KvToggleChip {
                width: 84
                labelText: qsTr("Automático")
                active: root.secretSource === "automatic"
                onToggled: root.fieldEdited("secretSource", "automatic")
            }

            KvToggleChip {
                width: 84
                labelText: qsTr("Ambiente")
                active: root.secretSource === "environment"
                onToggled: root.fieldEdited("secretSource", "environment")
            }

            KvToggleChip {
                width: 84
                labelText: qsTr("Perguntar")
                active: root.secretSource === "prompt"
                onToggled: root.fieldEdited("secretSource", "prompt")
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            color: Theme.textMuted
            font.pixelSize: 10
            text: {
                if (root.secretSource === "environment") {
                    return qsTr("A IDE lê a variável na hora de conectar. "
                                + "Só o NOME dela é salvo.");
                }
                if (root.secretSource === "prompt") {
                    return qsTr("A senha é pedida a cada sessão e vive só em "
                                + "memória.");
                }
                return qsTr("Não manda senha: o servidor decide. Cobre socket "
                            + "unix com peer, trust local e o ~/.pgpass.");
            }
        }

        DataSourceField {
            width: parent.width
            visible: root.secretSource === "environment"
            label: qsTr("Variável de ambiente")
            placeholder: "PGPASSWORD"
            value: root.draft ? (root.draft.secretVariable || "") : ""
            onEdited: text => root.fieldEdited("secretVariable", text)
        }
    }
}
