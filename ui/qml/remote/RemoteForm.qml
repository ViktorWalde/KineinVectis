pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O perfil do alvo SSH: nome, host, usuario, porta, chave e pasta de deploy.
// NAO HA' CAMPO DE SENHA, por desenho: a chave e' do usuario (`ssh-copy-id`),
// e o que o `ssh` precisar perguntar, pergunta no terminal da IDE.
Item {
    id: root

    property var draft: null
    property string program: ""
    property string deploySource: ""

    signal fieldEdited(string field, var value)
    signal programEdited(string text)
    signal deploySourceEdited(string text)

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        DataSourceField {
            width: parent.width
            label: qsTr("Nome")
            placeholder: qsTr("pi, bancada…")
            value: root.draft ? root.draft.name : ""
            onEdited: text => root.fieldEdited("name", text)
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: parent.width - porta.width - parent.spacing
                label: qsTr("Host ou IP")
                placeholder: qsTr("192.168.0.42, raspberrypi.local")
                value: root.draft ? root.draft.host : ""
                onEdited: text => root.fieldEdited("host", text)
            }

            DataSourceField {
                id: porta

                width: 64
                label: qsTr("Porta")
                numeric: true
                value: root.draft && root.draft.port ? String(root.draft.port) : ""
                onEdited: text => root.fieldEdited("port", text)
            }
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Usuário (vazio = o do ssh)")
            placeholder: "pi"
            value: root.draft ? root.draft.user : ""
            onEdited: text => root.fieldEdited("user", text)
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Chave privada (vazio = agente / ~/.ssh/config)")
            placeholder: "~/.ssh/id_ed25519"
            value: root.draft ? root.draft.identityFile : ""
            onEdited: text => root.fieldEdited("identityFile", text)
        }

        DataSourceField {
            width: parent.width
            label: qsTr("Pasta de deploy no alvo (vazio = ~/kinein/<projeto>)")
            placeholder: "/opt/app"
            value: root.draft ? root.draft.deployDir : ""
            onEdited: text => root.fieldEdited("deployDir", text)
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            text: qsTr("Sem senha, por desenho: copie sua chave com ssh-copy-id; "
                       + "o que o ssh perguntar, pergunta no terminal da IDE.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            DataSourceField {
                width: Math.round((parent.width - parent.spacing) / 2)
                label: qsTr("Origem do deploy (vazio = build/)")
                placeholder: "build/app"
                value: root.deploySource
                onEdited: text => root.deploySourceEdited(text)
            }

            DataSourceField {
                width: Math.round((parent.width - parent.spacing) / 2)
                label: qsTr("Programa no alvo (relativo = na pasta de deploy)")
                placeholder: "app, main.py, /opt/app/bin"
                value: root.program
                onEdited: text => root.programEdited(text)
            }
        }
    }
}
