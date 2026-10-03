pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CADASTRO DA INSTANCIA — que e' assunto do primeiro dia, nao do dia a dia.
//
// Ate' 2026-09-26 isto era o painel inteiro, sempre na tela: endereco, tres
// politicas de token e `Salvar · Sondar · Esquecer` com o mesmo peso. A §5.2
// da especificacao separa as duas superficies, e esta recolhe assim que ha'
// uma instancia que responde — voltando sozinha quando o proximo gesto mora
// aqui (endereco que nao respondeu), ou quando o autor pede por `configurar…`.
Column {
    id: root

    property var controller: null
    property var draft: ({ url: "", tokenSource: "none" })
    property int alturaCampo: 26

    readonly property int larguraRotulo: 78

    spacing: Theme.spacingSmall

    function focusUrl() {
        campoUrl.forceActiveFocus();
        campoUrl.selectAll();
    }

    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        // O QUE A TELA PROMETE E' O QUE ELA FAZ. A IDE conversa com um Grafana
        // que e' processo do usuario; ela nao o instala, nao o embute e nao o
        // desenha aqui dentro.
        text: qsTr("A IDE conversa com o seu Grafana pela API dele. Os painéis abrem no navegador.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    Row {
        width: parent.width
        spacing: Theme.spacingSmall

        Text {
            width: root.larguraRotulo
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Endereço")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        Rectangle {
            width: parent.width - root.larguraRotulo - Theme.spacingSmall
            height: root.alturaCampo
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: campoUrl.activeFocus ? Theme.accent : Theme.borderSoft

            TextInput {
                id: campoUrl

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeBody
                clip: true
                selectByMouse: true
                text: root.draft.url
                onTextEdited: root.controller.setDraftField("url", text)
            }
        }
    }

    // Os gestos raros continuam alcancaveis, e param de disputar a atencao de
    // quem so' quer ver um dashboard.
    Row {
        width: parent.width
        spacing: Theme.spacingSmall
        visible: root.controller ? root.controller.hasInstance : false

        KvBarButton {
            labelText: qsTr("Salvar endereço")
            enabled: root.controller ? root.controller.editing : false
            onActivated: root.controller.save()
        }

        KvBarButton {
            labelText: qsTr("Esquecer")
            onActivated: root.controller.forget()
        }

        // §7.2 regra 4: apagar a credencial e' gesto proprio, alcancavel ATE'
        // AUTENTICADO. Antes isso acontecia sozinho ao fechar o painel, o que
        // escondia a decisao dentro de outro gesto.
        KvBarButton {
            labelText: qsTr("Esquecer credencial")
            visible: root.controller ? root.controller.hasSessionToken : false
            onActivated: root.controller.forgetCredential()
        }
    }
}
