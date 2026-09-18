pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O que aconteceu no ultimo teste de conexao — a faixa comum (KvVerdict,
// F8) — e, quando for o caso, o campo de senha.
//
// O CAMPO DE SENHA APARECE POR `secretRequired`, NUNCA POR TEXTO. O core decide
// isso pelo `SQLSTATE`, que nao muda de idioma; a mensagem do servidor vem
// localizada, e ler ela aqui acoplaria a UI ao idioma do banco de quem roda
// (medido em 2026-09-04, `DocsPublic/roadmaps/35` §9.2).
//
// A senha digitada aqui vive na sessao e some ao trocar de perfil ou fechar o
// painel — ela nunca entra no perfil, que e' o que vai para o disco.
Item {
    id: root

    property bool testing: false
    property bool ok: false
    property string serverVersion: ""
    property string message: ""
    property bool secretRequired: false
    property string password: ""

    signal passwordEdited(string text)
    signal retryRequested()

    implicitHeight: coluna.implicitHeight

    Column {
        id: coluna

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingXSmall

        KvVerdict {
            width: parent.width
            busy: root.testing
            busyText: qsTr("Conectando...")
            ok: root.ok
            message: root.ok ? root.serverVersion : root.message
        }

        DataSourceField {
            width: parent.width
            visible: root.secretRequired
            label: qsTr("Senha desta sessão")
            placeholder: qsTr("o servidor pediu — não será salva")
            secret: true
            value: root.password
            onEdited: text => root.passwordEdited(text)
            onAccepted: root.retryRequested()
        }
    }
}
