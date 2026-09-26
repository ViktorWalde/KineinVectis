pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// OS INVENTARIOS: o que o Grafana tem, filtrado pelo que o autor digitou.
//
// Vem DEPOIS do cruzamento (`GrafanaMatches`) e dentro da area que rola,
// porque sao listas que podem ter dezenas de linhas — e porque sao coisas que
// o navegador tambem mostra. O que a IDE tem de proprio esta' la' em cima.
Column {
    id: root

    property var matches: []
    property var dataSources: []
    property var dashboards: []
    // O FILTRO MORA NO PAINEL e chega aqui pronto: a caixa de texto fica onde
    // o §5.2 a poe, acima das listas, e esta Column so' desenha o resultado.
    property string filtro: ""

    readonly property var dashboardsVisiveis:
        peneira.apply(root.dashboards, peneira.camposDashboard, root.filtro)
    readonly property var fontesVisiveis:
        peneira.apply(root.dataSources, peneira.camposFonte, root.filtro)

    // A grade recebe linhas por CHAVE. `tipo` junta o nome bonito e o id
    // tecnico numa coluna so' porque a pergunta e' uma: que tipo de fonte e'
    // esta. `padrão` vira texto porque a grade desenha texto — um booleano
    // apareceria como "true", que nao e' portugues nem ingles.
    readonly property var linhasFontes: root.fontesVisiveis.map(fonte => ({
        "nome": fonte.name,
        "tipo": fonte.typeName !== "" ? fonte.typeName : fonte.typeId,
        "endereço": fonte.url,
        "padrão": fonte.isDefault ? qsTr("sim") : ""
    }))

    readonly property var linhasDashboards: root.dashboardsVisiveis.map(painel => ({
        "dashboard": painel.title,
        "pasta": painel.folderTitle
    }))

    property int dashboardEscolhido: -1
    property int fonteEscolhida: -1

    GrafanaFilterRules {
        id: peneira
    }

    signal dashboardActivated(string path)

    spacing: Theme.spacingMedium

    component Titulo: Text {
        color: Theme.textMuted
        font.pixelSize: 10
        font.bold: true
    }

    // DASHBOARDS PRIMEIRO entre os inventarios: e' o que se abre todo dia.
    Column {
        width: parent.width
        spacing: 2
        visible: root.dashboards.length > 0

        Titulo { text: qsTr("DASHBOARDS") }

        KvDataGrid {
            id: gradeDashboards

            width: parent.width
            columns: [
                { key: "dashboard", label: qsTr("dashboard") },
                { key: "pasta", label: qsTr("pasta") }
            ]
            rows: root.linhasDashboards
            selectable: true
            selectedIndex: root.dashboardEscolhido
            mono: false
            maxHeight: 180
            emptyText: ""

            onRowClicked: indice => root.dashboardEscolhido = indice
            // ABRIR NO NAVEGADOR, e nao dentro da IDE. A licenca AGPL do
            // Grafana decide a forma da integracao: a IDE CONVERSA com ele,
            // nunca o embute (DocsPublic/integracoes/37 §2).
            onRowActivated: indice => {
                if (indice >= 0 && indice < root.dashboardsVisiveis.length) {
                    root.dashboardActivated(root.dashboardsVisiveis[indice].url);
                }
            }
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.linhasDashboards.length === 0
            text: peneira.emptyPhrase(root.dashboards.length, root.filtro)
            color: Theme.textMuted
            font.pixelSize: 9
        }

        Text {
            width: parent.width
            visible: gradeDashboards.activeFocus
            text: qsTr("Enter abre no navegador")
            color: Theme.textMuted
            font.pixelSize: 9
        }
    }

    Column {
        width: parent.width
        spacing: 2
        visible: root.dataSources.length > 0

        Titulo { text: qsTr("FONTES DE DADOS NO GRAFANA") }

        KvDataGrid {
            width: parent.width
            columns: [
                { key: "nome", label: qsTr("fonte") },
                { key: "tipo", label: qsTr("tipo") },
                { key: "endereço", label: qsTr("endereço") },
                { key: "padrão", label: qsTr("padrão") }
            ]
            rows: root.linhasFontes
            selectable: true
            selectedIndex: root.fonteEscolhida
            mono: false
            maxHeight: 140
            emptyText: ""

            onRowClicked: indice => root.fonteEscolhida = indice
        }

        Text {
            width: parent.width
            wrapMode: Text.WordWrap
            visible: root.linhasFontes.length === 0
            text: peneira.emptyPhrase(root.dataSources.length, root.filtro)
            color: Theme.textMuted
            font.pixelSize: 9
        }
    }
}
