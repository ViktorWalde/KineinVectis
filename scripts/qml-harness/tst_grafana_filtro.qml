// O FILTRO LOCAL DAS LISTAS DO GRAFANA (§5.2: "filtro atua localmente sobre o
// resultado ja' recebido").
//
// Duas coisas se medem aqui. A regra — o que casa com o que — e a FIACAO, que
// e' onde mora o erro classico: a grade devolve o indice da lista VISIVEL, e
// abrir `dashboards[indice]` em vez de `visiveis[indice]` abre o dashboard
// errado assim que alguem digita qualquer coisa. Ninguem percebe enquanto o
// filtro esta' vazio, porque ai' as duas listas coincidem.
//
// MUTACAO QUE PROVA O GATE: no `GrafanaFindings`, troque
// `root.dashboardsVisiveis[indice].url` por `root.dashboards[indice].url` e a
// assercao do indice cai.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 600
    height: 400

    property int falhas: 0
    property var abertos: []

    readonly property var paineis: [
        { title: "Visão geral da API", folderTitle: "Plataforma", url: "/d/api" },
        { title: "Banco / latência", folderTitle: "Dados", url: "/d/banco" },
        { title: "Fila de jobs", folderTitle: "Dados", url: "/d/fila" }
    ]

    readonly property var fontes: [
        { name: "postgres-prod", typeName: "PostgreSQL", typeId: "postgres",
          url: "prod:5432", isDefault: true },
        { name: "loki", typeName: "Loki", typeId: "loki", url: "loki:3100", isDefault: false }
    ]

    GrafanaFilterRules {
        id: regras
    }

    GrafanaFindings {
        id: achados

        width: parent.width
        dashboards: root.paineis
        dataSources: root.fontes

        onDashboardActivated: caminho => root.abertos.push(caminho)
    }

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    function acharGrade(item) {
        for (let indice = 0; indice < item.children.length; ++indice) {
            const filho = item.children[indice];
            if (filho.rows !== undefined && filho.columns !== undefined
                && filho.selectable === true) {
                return filho;
            }
            const achado = root.acharGrade(filho);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    Component.onCompleted: {
        // FILTRO VAZIO NAO E' FILTRO.
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "").length === 3,
                      "filtro vazio escondeu alguma coisa");
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "   ").length === 3,
                      "so' espaco em branco filtrou");

        // CASA POR QUALQUER CAMPO DECLARADO, sem distinguir maiuscula.
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "dados").length === 2,
                      "procurar pela pasta nao achou os dois dashboards dela");
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "API").length === 1,
                      "a busca distinguiu maiuscula de minuscula");
        root.conferir(regras.apply(root.fontes, regras.camposFonte, "postgres").length === 1,
                      "procurar pelo tipo da fonte nao achou");

        // TEXTO E' TEXTO: um ponto nao vira "qualquer caractere", e um
        // asterisco digitado por engano nao vira sintaxe.
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "b.nco").length === 0,
                      "o ponto do filtro virou regex");
        root.conferir(regras.apply(root.paineis, regras.camposDashboard, "*").length === 0,
                      "o asterisco do filtro virou curinga");

        // "NADA CASA" E' DIFERENTE DE "NAO HA' NADA".
        root.conferir(regras.emptyPhrase(3, "zzz") !== "",
                      "lista cheia com filtro sem resultado ficou muda");
        root.conferir(regras.emptyPhrase(0, "zzz") === "",
                      "lista vazia culpou o filtro");
        root.conferir(regras.emptyPhrase(3, "") === "",
                      "sem filtro, a frase de filtro apareceu");

        // A FIACAO: a lista desenhada encolhe junto.
        achados.filtro = "dados";
        root.conferir(achados.dashboardsVisiveis.length === 2,
                      "o filtro nao chegou na lista desenhada");
        root.conferir(achados.linhasDashboards.length === 2,
                      "as linhas da grade nao seguiram o filtro");

        // O INDICE PERTENCE A' LISTA VISIVEL. Com "dados", a linha 0 e' o
        // "Banco / latência" — nunca o primeiro da lista inteira.
        const grade = root.acharGrade(achados);
        root.conferir(grade !== null, "a grade de dashboards sumiu do painel");
        if (grade !== null) {
            grade.rowActivated(0);
            root.conferir(root.abertos.length === 1 && root.abertos[0] === "/d/banco",
                          "abriu o dashboard errado da lista filtrada: "
                          + (root.abertos.length === 0 ? "nenhum" : root.abertos[0]));
        }

        // INDICE FORA DA LISTA NAO ABRE NADA: o filtro pode encolher a lista
        // depois de a selecao ter sido feita.
        achados.filtro = "zzz";
        if (grade !== null) {
            grade.rowActivated(0);
        }
        root.conferir(root.abertos.length === 1,
                      "abriu alguma coisa com a lista filtrada vazia");

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
