import QtQuick
import KineinVectis

// A ABA WEB do Grafana (0.154.0, roadmaps/59 §6.1): o Grafana local dentro da
// IDE, sobre a pagina web local comum (KvLocalWebPage, 52 §14.45), com os
// textos dele. A regra (opcional, so' local, links ao navegador) e' da base.
KvLocalWebPage {
    id: root

    iconName: "observability"
    viewName: "GrafanaWebView"
    disabledTitle: qsTr("Grafana dentro da IDE está desligado")
    disabledText: qsTr("Ligado, os dashboards abrem nesta aba, ao lado do código. Desligado, o navegador embutido nem é carregado; a opção também fica em Configurações → Interface.")
    enableLabel: qsTr("Ligar o Grafana dentro da IDE")
    remoteTitle: qsTr("Só um Grafana local abre aqui")
    remoteText: qsTr("A aba Web abre só localhost, 127.0.0.1 e ::1 — é para o Grafana que você desenvolve neste projeto. %1 abre no navegador.").arg(root.baseUrl)
}
