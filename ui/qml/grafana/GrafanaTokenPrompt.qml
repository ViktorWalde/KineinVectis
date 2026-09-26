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
        font.pixelSize: 9
    }

    Text {
        id: aviso

        anchors.top: motivo.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        wrapMode: Text.WordWrap
        text: qsTr("Cole o token da conta de serviço. Ele vive só nesta sessão.")
        color: Theme.warningSoft
        font.pixelSize: 10
    }

    Row {
        id: linha

        anchors.top: aviso.bottom
        anchors.topMargin: Theme.spacingXSmall
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        Rectangle {
            width: parent.width - confirmar.width - Theme.spacingSmall
            height: 26
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: entrada.activeFocus ? Theme.accent : Theme.borderSoft

            TextInput {
                id: entrada

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                // O token nao aparece na tela: um print de tela num chamado e'
                // o caminho mais banal de vazamento que existe.
                echoMode: TextInput.Password
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 12
                clip: true
                onAccepted: {
                    root.accepted(entrada.text);
                    entrada.text = "";
                }
            }
        }

        KvBarButton {
            id: confirmar

            labelText: root.labelText
            onActivated: {
                root.accepted(entrada.text);
                entrada.text = "";
            }
        }
    }
}
