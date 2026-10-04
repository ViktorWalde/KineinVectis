pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O painel de observabilidade: qual Grafana observa este projeto, e o que ele
// ja' sabe sobre os bancos daqui. Desde 2026-10-04 e' a aba "Painel" da
// GrafanaWindow acoplada (59 §6.1): o cabecalho e o fechar sao da janela.
//
// Burro de proposito, como o `DataSourcePanel`: recebe estado e emite pedidos.
// Quem guarda e' o controller; quem decide e' o core.
Item {
    id: root

    property var controller: null

    // Um dashboard dos achados: a janela decide onde abre (aba Web ou
    // navegador do sistema).
    signal dashboardActivated(string path)

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

    // A ENGRENAGEM da janela (2026-10-04): "" = a regra decide; "open" e
    // "closed" = o autor decidiu, e vence a regra. O antigo `configurar…` so'
    // sabia fixar ABERTO — com a regra ja' abrindo, o clique nao mudava nada
    // (o "configurar nao faz nada" do 59 §6).
    property string setupChoice: ""
    readonly property bool setupShown: root.setupChoice === "open"
            || (root.setupChoice === "" && (root.controller ? root.controller.setupExpanded : true))

    function toggleSetup() {
        root.setupChoice = root.setupShown ? "closed" : "open";
    }

    readonly property bool temAchados: root.controller !== null
            && (root.controller.dashboards.length > 0
                || root.controller.dataSources.length > 0)

    readonly property var action: root.controller
            ? root.controller.primaryAction
            : ({ "kind": "", "label": "", "hint": "", "section": "" })

    // O gesto primario NAO decide nada: ele leva ao dono do que o estado pede.
    function executarPrimaria() {
        if (root.controller === null) {
            return;
        }
        switch (root.action.kind) {
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
            root.setupChoice = "open";
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

        // A ACAO PRIMARIA, uma so', derivada do estado (GrafanaActionRules).
        // Rotulo vazio esconde o botao: quando o gesto primario E' o campo do
        // token, quem o desenha e' o `GrafanaTokenPrompt`.
        KvButton {
            objectName: "grafanaPrimary"

            width: parent.width
            primary: true
            visible: root.action.label !== "" && root.action.kind !== "provideToken"
            enabled: root.action.kind !== "waiting"
            text: root.action.label
            tooltip: root.action.hint
            onClicked: root.executarPrimaria()
        }

        GrafanaSetupSection {
            id: ajustes

            width: parent.width
            // A REGRA DECIDE, O AUTOR TEM A ULTIMA PALAVRA: a configuracao se
            // abre sozinha quando o proximo gesto mora nela, e a engrenagem
            // abre ou fecha por cima da regra.
            visible: root.setupShown
            controller: root.controller
            draft: root.draft
        }

        GrafanaAuthSection {
            width: parent.width
            // Sozinha so' quando o servidor pede (a regra); a engrenagem a
            // mostra junto do endereco — sem isso, um Grafana que responde
            // sem token nao deixava dar um para listar os dashboards.
            visible: root.setupShown || (root.controller ? root.controller.authVisible : false)
            controller: root.controller
            draft: root.draft
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

            // Tambem quando a pessoa ESCOLHEU "Pedir na sessao" e ainda nao deu
            // um: antes a escolha nao abria campo nenhum (beco sem saida
            // achado na tela em 2026-10-04).
            readonly property bool chosen: root.draft.tokenSource === "prompt"
                                           && root.controller !== null && !root.controller.hasSessionToken
            width: parent.width
            visible: root.action.kind === "provideToken" || chosen
            reasonText: root.action.kind === "provideToken" ? root.action.hint : ""
            labelText: root.action.kind === "provideToken" ? root.action.label : qsTr("Usar o token")
            primaryGesture: root.action.kind === "provideToken"
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
        KvTextField {
            id: campoFiltro

            width: parent.width
            height: root.alturaCampo
            visible: root.temAchados
            codeFont: false
            pixelSize: Theme.fontSizeSmall
            iconName: "search"
            clearable: true
            placeholder: qsTr("filtrar dashboards, pasta ou fonte…")
            // ESC LIMPA, e nao fecha nada: e' o gesto que devolve a lista
            // inteira sem tirar a mao do teclado.
            Keys.onEscapePressed: campoFiltro.clear()
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

            onDashboardActivated: caminho => root.dashboardActivated(caminho)
        }
    }
}
