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
        text: qsTr("A IDE conversa com o seu Grafana pela API dele. Os dashboards abrem no navegador ou, com a opção ligada, na aba Web.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
    }

    // Padrao novo de frontend (2026-10-04): o rotulo mora no campo.
    KvTextField {
        id: campoUrl

        width: parent.width
        label: qsTr("Endereço do Grafana")
        pixelSize: Theme.fontSizeSmall
        text: root.draft.url
        onEdited: (text) => root.controller.setDraftField("url", text)
    }

    // Os gestos raros continuam alcancaveis, e param de disputar a atencao de
    // quem so' quer ver um dashboard.
    Row {
        width: parent.width
        spacing: Theme.spacingSmall
        visible: root.controller ? root.controller.hasInstance : false

        KvButton {
            compact: true
            text: qsTr("Salvar")
            enabled: root.controller ? root.controller.editing : false
            onClicked: root.controller.save()
        }

        KvButton {
            compact: true
            danger: true
            text: qsTr("Esquecer este Grafana")
            onClicked: root.controller.forget()
        }

        // §7.2 regra 4: apagar a credencial e' gesto proprio, alcancavel ATE'
        // AUTENTICADO. Antes isso acontecia sozinho ao fechar o painel, o que
        // escondia a decisao dentro de outro gesto.
        KvButton {
            compact: true
            text: qsTr("Esquecer o token")
            visible: root.controller ? root.controller.hasSessionToken : false
            onClicked: root.controller.forgetCredential()
        }
    }
}
