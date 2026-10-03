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

    signal closeRequested()

    // ONDE O CURSOR CAI AO ABRIR. Primeiro uso: o campo do endereco, que e' o
    // unico que importa. Uso diario: o filtro, que e' por onde se acha o
    // dashboard. Sem isto, abrir por atalho ainda exigia pegar o mouse para
    // digitar a primeira letra.
    onVisibleChanged: {
        if (root.visible) {
            Qt.callLater(root.takeFocus);
        }
    }

    function takeFocus() {
        if (!root.visible) {
            return;
        }
        if (ajustes.visible) {
            ajustes.focusUrl();
        } else if (root.temAchados) {
            campoFiltro.forceActiveFocus();
        }
    }

    readonly property var draft: root.controller ? root.controller.draft
                                                 : ({ url: "", tokenSource: "none" })

    // Uma linha de rotulo + campo, medida pelo conteudo com piso — o
    // dimensionamento que o autor pediu em 2026-09-04.
    readonly property int alturaCampo: 26

    // O autor abriu `configurar…` com tudo em ordem. E' escolha dele, e por
    // isso mora na tela e nao na regra.
    property bool setupPinned: false

    readonly property bool temAchados: root.controller !== null
            && (root.controller.dashboards.length > 0
                || root.controller.dataSources.length > 0)

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
            // de ser pre-condicao visual para sondar — e o botao que promete
            // "testa o endereco" passa a testar mesmo.
            root.controller.connect();
            break;
        case "refresh":
            root.controller.probe();
            break;
        case "fixUrl":
            // LEVAR ATE' O CAMPO, e nao so' acender um botao: e' para isso que
            // a acao carrega `section`.
            root.setupPinned = true;
            ajustes.focusUrl();
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

        // A MESMA PRIMEIRA LINHA DOS OUTROS QUATRO PAINEIS de ambiente. Eu
        // tinha escrito um cabecalho proprio para o Grafana — titulo,
        // subtitulo e botoes —, que e' exatamente o que o `KvPanelHeader` ja'
        // fazia para banco, remoto, embarcados e containers. Painel que comeca
        // diferente dos irmaos custa uma leitura a mais a cada abertura, e a
        // §9 da especificacao pede coerencia com estes componentes.
        KvPanelHeader {
            width: parent.width
            title: qsTr("Observabilidade")
            subtitle: root.controller ? root.controller.statusPhrase : ""
            // Rotulo vazio esconde o botao: quando o gesto primario E' o campo
            // do token, quem o desenha e' o `GrafanaTokenPrompt`.
            primaryLabel: root.acao.kind === "provideToken" ? "" : root.acao.label
            primaryBusy: root.acao.kind === "waiting"
            secondaryLabel: root.setupPinned ? qsTr("ocultar ajustes")
                                             : qsTr("configurar…")
            onPrimaryRequested: root.executarPrimaria()
            onSecondaryRequested: root.setupPinned = !root.setupPinned
            onCloseRequested: root.closeRequested()
        }

        GrafanaSetupSection {
            id: ajustes

            width: parent.width
            // A REGRA DECIDE, O AUTOR TEM A ULTIMA PALAVRA: a configuracao se
            // abre sozinha quando o proximo gesto mora nela, e o `configurar…`
            // a mantem aberta quando ele quer mexer com tudo em ordem.
            visible: (root.controller ? root.controller.setupExpanded : true)
                     || root.setupPinned
            controller: root.controller
            draft: root.draft
            alturaCampo: root.alturaCampo
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
            reasonText: root.acao.hint
            labelText: root.acao.label
            onAccepted: token => root.controller.probeWithToken(token)
        }

        GrafanaVerdict {
            width: parent.width
            controller: root.controller
        }

        GrafanaMatches {
            width: parent.width
            matches: root.controller ? root.controller.matches : []
            dataSources: root.controller ? root.controller.dataSources : []
            authenticated: root.controller ? root.controller.authenticated : false
        }

        // AREA VAZIA NAO PODE SER AREA MUDA: carregando, nada recebido,
        // desatualizado e "sem token a API nao lista" sao respostas
        // diferentes, e cada uma tem a sua frase.
        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            // Column NAO reserva espaco para filho invisivel, entao nao ha'
            // altura para zerar — e zera-la fazia `height` e `implicitHeight`
            // se perseguirem num laco de binding.
            visible: text !== ""
            text: root.controller ? root.controller.contentPhrase : ""
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }

        // O FILTRO SO' EXISTE QUANDO HA' O QUE FILTRAR. Uma caixa de busca
        // sobre lista vazia e' convite a procurar o que nao chegou.
        Rectangle {
            width: parent.width
            height: root.alturaCampo
            visible: root.temAchados
            radius: Theme.radius
            color: Theme.background0
            border.width: 1
            border.color: campoFiltro.activeFocus ? Theme.accent : Theme.borderSoft

            TextInput {
                id: campoFiltro

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
                clip: true
                selectByMouse: true
                // ESC LIMPA, e nao fecha nada: e' o gesto que devolve a lista
                // inteira sem tirar a mao do teclado.
                Keys.onEscapePressed: campoFiltro.text = ""
            }

            Text {
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                anchors.verticalCenter: parent.verticalCenter
                visible: campoFiltro.text === ""
                text: qsTr("filtrar dashboards, pasta ou fonte…")
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeSmall
            }
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
            dataSources: root.controller ? root.controller.dataSources : []
            dashboards: root.controller ? root.controller.dashboards : []
            filtro: campoFiltro.text

            onDashboardActivated: caminho =>
                Qt.openUrlExternally(root.controller.dashboardUrl(caminho))
        }
    }
}
