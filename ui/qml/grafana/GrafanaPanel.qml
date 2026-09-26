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

    readonly property var acao: root.controller
            ? root.controller.primaryAction
            : ({ "kind": "", "label": "", "hint": "", "section": "" })

    // O gesto primario NAO decide nada: ele leva ao dono do que o estado pede.
    function executarPrimaria() {
        if (root.controller === null) {
            return;
        }
        switch (root.acao.kind) {
        case "connect":
            // CONECTAR e' salvar e medir no mesmo gesto (§5.1). Salvar deixou
            // de ser pre-condicao visual para sondar.
            root.controller.save();
            break;
        case "refresh":
            root.controller.probe();
            break;
        case "fixUrl":
            campoUrl.forceActiveFocus();
            campoUrl.selectAll();
            break;
        case "provideToken":
            root.controller.setDraftField("tokenSource", "prompt");
            break;
        default:
            break;
        }
    }

    // O FORMULARIO MEDE O QUE PEDE, e so'. Com `anchors.fill: parent` ele
    // ocupava a altura inteira, e a Flickable abaixo — ancorada em
    // `coluna.bottom` — nascia com ZERO de altura: a lista de achados, que e' o
    // ponto do painel, nao tinha como aparecer nunca. Sem erro, sem warning;
    // so' um espaco preto onde deviam estar as fontes de dados. Achado na cena
    // de inspecao da V7, em 2026-09-26.
    Column {
        id: coluna

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
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

        GrafanaAuthSection {
            width: parent.width
            controller: root.controller
            draft: root.draft
            alturaCampo: root.alturaCampo
        }

        // O PEDIDO DE TOKEN aparece por decisao do CORE (`SECRET_REQUIRED`,
        // nunca por leitura de mensagem) — e, quando aparece, ELE E' A ACAO
        // PRIMARIA. Ate' 2026-09-26 o painel desenhava as duas coisas: este
        // campo com o seu proprio `Sondar` E um botao `Fornecer token` logo
        // acima, que so' trocava a politica. Dois gestos para a mesma
        // intencao e' exatamente o atrito que a §3 mede; a cena de inspecao
        // da V7 mostrou os dois lado a lado.
        GrafanaTokenPrompt {
            objectName: "grafanaTokenPrompt"

            width: parent.width
            visible: root.acao.kind === "provideToken"
            height: visible ? implicitHeight : 0
            reasonText: root.acao.hint
            labelText: root.acao.label
            onAccepted: token => root.controller.probeWithToken(token)
        }

        // UMA ACAO PRIMARIA, decidida pelo estado (GrafanaActionRules). Antes
        // eram tres com o mesmo peso — `Salvar · Sondar · Esquecer` —, e a §3
        // nomeia o atrito: salvar antes de sondar era exigencia do DESENHO, e
        // nao intencao de quem usa.
        Row {
            objectName: "grafanaPrimaria"

            width: parent.width
            spacing: Theme.spacingSmall
            // Quando o gesto primario E' o campo do token, quem o desenha e' o
            // `GrafanaTokenPrompt` acima: um botao aqui seria o segundo.
            visible: root.acao.kind !== "provideToken"
            height: visible ? implicitHeight : 0

            KvBarButton {
                labelText: root.acao.label
                enabled: root.acao.kind !== "waiting"
                onActivated: root.executarPrimaria()
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width - 140
                wrapMode: Text.WordWrap
                text: root.acao.hint
                color: Theme.textMuted
                font.pixelSize: 9
            }
        }

        // Os gestos raros continuam alcancaveis, e param de disputar a atencao
        // de quem so' quer ver um dashboard.
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

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.controller ? root.controller.freshness : ""
                color: Theme.textMuted
                font.pixelSize: 9
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
        objectName: "grafanaAchados"

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
