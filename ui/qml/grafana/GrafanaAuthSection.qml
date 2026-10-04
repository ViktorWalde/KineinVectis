pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// ONDE PROCURAR O TOKEN — e so' isso. Nunca qual ele e': o token da sessao
// vive no controller e nao passa por aqui.
//
// A secao inteira aparece POR DECISAO DO ESTADO (`authVisible`, de
// GrafanaActionRules): ate' 2026-09-26 as tres politicas ficavam na tela desde
// a primeira abertura, antes de a API dizer se alguma era precisa — o atrito
// que a §3 da especificacao mede. Estar fora quando nao e' necessaria e' o
// comportamento, nao uma economia de pixel.
Column {
    id: root

    property var controller: null
    property var draft: ({ url: "", tokenSource: "none" })

    // Uma condicao, um dono: os filhos so' carregam o que os distingue.
    visible: root.controller ? root.controller.authVisible : false
    spacing: Theme.spacingSmall


    Row {
        width: parent.width
        spacing: Theme.spacingSmall

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Token")
            color: Theme.textSecondary
            font.pixelSize: Theme.fontSizeSmall
        }

        Repeater {
            // SO' AS POLITICAS QUE PODEM DAR CERTO. `none` era a terceira
            // opcao aqui, e a secao inteira so' aparece depois que o servidor
            // EXIGIU uma credencial: oferecer "sem token" nesse ponto e'
            // oferecer o caminho que acabou de falhar. Ela continua sendo o
            // padrao do perfil — e' o que o `emptyProfile` devolve.
            model: [
                { valor: "environment", rotulo: qsTr("Variável de ambiente") },
                { valor: "prompt", rotulo: qsTr("Pedir na sessão") }
            ]

            delegate: KvButton {
                id: chipPolitica

                required property var modelData

                compact: true
                text: chipPolitica.modelData.rotulo
                selected: root.draft.tokenSource === chipPolitica.modelData.valor
                onClicked: root.controller.setDraftField("tokenSource",
                                                         chipPolitica.modelData.valor)
            }
        }
    }

    // POR QUE O SERVIDOR PEDIU: enquanto nenhuma politica foi escolhida, esta
    // linha diz o que se ganha e o que se perde — e evita procurar uma
    // credencial para ver o que ja' esta' visivel.
    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        visible: root.draft.tokenSource === "none"
        text: qsTr("Sem token dá para ver a versão e a saúde. O que há dentro exige uma conta de serviço.")
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeMicro
    }

    KvTextField {
        width: parent.width
        visible: root.controller ? root.controller.draftUsesVariable : false
        label: qsTr("Variável de ambiente com o token")
        pixelSize: Theme.fontSizeSmall
        text: root.draft.tokenVariable !== undefined ? root.draft.tokenVariable : ""
        placeholder: "GRAFANA_TOKEN"
        onEdited: (text) => root.controller.setDraftField("tokenVariable", text)
    }
}
