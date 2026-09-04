pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de observabilidade: qual Grafana observa este projeto, e o que ele
// ja' sabe sobre os bancos daqui.
//
// Burro de proposito, como o `DataSourcePanel`: recebe estado e emite pedidos.
// Quem guarda e' o controller; quem decide e' o core.
Item {
    id: root

    property var controller: null

    readonly property var draft: root.controller ? root.controller.draft
                                                 : ({ url: "", tokenSource: "none" })

    // Uma linha de rotulo + campo, medida pelo conteudo com piso — o
    // dimensionamento que o autor pediu em 2026-09-04.
    readonly property int alturaCampo: 26

    Column {
        id: coluna

        anchors.fill: parent
        spacing: Theme.spacingSmall

        Text {
            width: parent.width
            text: qsTr("Observabilidade")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            // O QUE A TELA PROMETE E' O QUE ELA FAZ. A IDE conversa com um
            // Grafana que e' processo do usuario; ela nao o instala, nao o
            // embute e nao o desenha aqui dentro.
            text: qsTr("A IDE conversa com o seu Grafana pela API dele. Os painéis abrem no navegador.")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                width: 78
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Endereço")
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            Rectangle {
                width: parent.width - 78 - Theme.spacingSmall
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
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    text: root.draft.url
                    onTextEdited: root.controller.setDraftField("url", text)
                }
            }
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                width: 78
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Token")
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            Repeater {
                // A politica diz ONDE procurar o token, nunca qual ele e'.
                model: [
                    { valor: "none", rotulo: qsTr("Sem token") },
                    { valor: "environment", rotulo: qsTr("Variável de ambiente") },
                    { valor: "prompt", rotulo: qsTr("Pedir na sessão") }
                ]

                delegate: KvToggleChip {
                    id: chipPolitica

                    required property var modelData

                    height: 22
                    labelText: chipPolitica.modelData.rotulo
                    active: root.draft.tokenSource === chipPolitica.modelData.valor
                    onToggled: root.controller.setDraftField("tokenSource",
                                                             chipPolitica.modelData.valor)
                }
            }
        }

        // SEM TOKEN NAO E' ERRO: e' um estado com resposta propria, e dize-lo
        // evita o autor procurar uma credencial que nao precisa ter.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.draft.tokenSource === "none"
            text: qsTr("Sem token dá para ver a versão e a saúde. O que há dentro exige uma conta de serviço.")
            color: Theme.textMuted
            font.pixelSize: 9
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall
            visible: root.controller ? root.controller.draftUsesVariable : false

            Text {
                width: 78
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Variável")
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            Rectangle {
                width: parent.width - 78 - Theme.spacingSmall
                height: root.alturaCampo
                radius: Theme.radius
                color: Theme.background0
                border.width: 1
                border.color: campoVariavel.activeFocus ? Theme.accent : Theme.borderSoft

                TextInput {
                    id: campoVariavel

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    text: root.draft.tokenVariable !== undefined ? root.draft.tokenVariable : ""
                    onTextEdited: root.controller.setDraftField("tokenVariable", text)
                }

                Text {
                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    visible: campoVariavel.text === ""
                    text: "GRAFANA_TOKEN"
                    color: Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                }
            }
        }

        // O PEDIDO DE TOKEN aparece por decisao do CORE, nunca por leitura de
        // mensagem: o campo `SECRET_REQUIRED` e' quem manda.
        GrafanaTokenPrompt {
            width: parent.width
            visible: root.controller ? root.controller.tokenRequired : false
            onAccepted: token => root.controller.probeWithToken(token)
        }

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            KvBarButton {
                labelText: qsTr("Salvar")
                onActivated: root.controller.save()
            }

            KvBarButton {
                labelText: root.controller && root.controller.probing
                           ? qsTr("Sondando…") : qsTr("Sondar")
                enabled: root.controller ? root.controller.hasInstance
                                           && !root.controller.probing : false
                onActivated: root.controller.probe()
            }

            KvBarButton {
                labelText: qsTr("Esquecer")
                enabled: root.controller ? root.controller.hasInstance : false
                onActivated: root.controller.forget()
            }
        }

        GrafanaVerdict {
            width: parent.width
            controller: root.controller
        }
    }

    // A LISTA ROLA, o formulario nao. Um Grafana com quarenta dashboards nao
    // pode empurrar os botoes para fora da tela.
    Flickable {
        anchors.top: coluna.bottom
        anchors.topMargin: Theme.spacingSmall
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        contentHeight: achados.implicitHeight
        contentWidth: width
        boundsBehavior: Flickable.StopAtBounds
        clip: true

        GrafanaFindings {
            id: achados

            width: parent.width
            matches: root.controller ? root.controller.matches : []
            dataSources: root.controller ? root.controller.dataSources : []
            dashboards: root.controller ? root.controller.dashboards : []
            authenticated: root.controller ? root.controller.authenticated : false

            onDashboardActivated: caminho =>
                Qt.openUrlExternally(root.controller.dashboardUrl(caminho))
        }
    }
}
