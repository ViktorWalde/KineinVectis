pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O campo do token da sessao.
//
// Ele so' existe enquanto a resposta esta' na tela: o texto nunca e' gravado,
// nunca vai para o perfil e some quando o painel fecha. Mesma regra da senha
// de banco (`DocsPublic/seguranca/40`).
Item {
    id: root

    signal accepted(string token)

    // POR QUE o campo esta' na tela — vem da acao primaria, que e' quem sabe
    // distinguir "o servidor pediu" de "o servidor recusou o que voce deu".
    property string reasonText: ""
    // O rotulo do botao tambem vem de la': quando ele e' O gesto primario, ele
    // fala a lingua do estado ("Tentar outro token"), nao "Sondar".
    property string labelText: qsTr("Sondar")
    // Ambar so' quando E' o gesto primario: uma acao primaria por tela.
    property bool primaryGesture: true

    // Piso mais o que o conteudo pedir — o dimensionamento decidido pelo autor
    // em 2026-09-04.
    implicitHeight: Math.max(52, linha.implicitHeight + aviso.implicitHeight
                                 + motivo.implicitHeight + 8)
    height: implicitHeight

    Text {
        id: motivo

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        visible: root.reasonText !== ""
        height: visible ? implicitHeight : 0
        text: root.reasonText
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeMicro
    }

    Text {
        id: aviso

        anchors.top: motivo.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("Cole o token da conta de serviço. Ele vive só nesta sessão.")
        color: Theme.warningSoft
        font.pixelSize: Theme.fontSizeCaption
    }

    Row {
        id: linha

        anchors.top: aviso.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        KvTextField {
            id: tokenInput

            width: parent.width - confirmar.width - Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            label: qsTr("Token da conta de serviço")
            pixelSize: Theme.fontSizeSmall
            // O token nao aparece na tela: um print de tela num chamado e'
            // o caminho mais banal de vazamento que existe.
            echoMode: TextInput.Password
            onAccepted: {
                root.accepted(tokenInput.text);
                tokenInput.text = "";
            }
        }

        KvButton {
            id: confirmar

            anchors.verticalCenter: parent.verticalCenter
            primary: root.primaryGesture
            text: root.labelText
            enabled: tokenInput.text !== ""
            onClicked: {
                root.accepted(tokenInput.text);
                tokenInput.text = "";
            }
        }
    }
}
