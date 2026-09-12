pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O QUE A SONDA ACHOU. Tres listas, e a ORDEM delas e' a resposta.
//
// O cruzamento vem primeiro de proposito. As outras duas — fontes de dados e
// dashboards — sao coisas que o navegador tambem mostra; o cruzamento e' a
// unica que exige as DUAS metades, e so' a IDE tem a segunda: o catalogo de
// bancos deste workspace.
//
// DIMENSIONAMENTO (decisao do autor, 2026-09-04): cada linha mede o proprio
// conteudo e tem um PISO. Altura fixa cortaria o nome longo de um dashboard;
// altura puramente automatica deixaria a lista com linhas de alturas
// diferentes, dificil de varrer com o olho.
Column {
    id: root

    property var matches: []
    property var dataSources: []
    property var dashboards: []
    property bool authenticated: false

    // Altura minima de uma linha. Abaixo disto o alvo de clique fica menor que
    // o dedo/cursor consegue mirar com conforto.
    readonly property int alturaMinima: 26

    signal dashboardActivated(string path)

    spacing: Theme.spacingMedium

    component Titulo: Text {
        color: Theme.textMuted
        font.pixelSize: 10
        font.bold: true
    }

    // O ACHADO PRINCIPAL.
    Column {
        width: parent.width
        spacing: 2
        visible: root.matches.length > 0

        Titulo {
            text: qsTr("O GRAFANA JÁ OBSERVA DESTE PROJETO")
            color: Theme.successSoft
        }

        Repeater {
            model: root.matches

            delegate: Item {
                id: casamento

                required property var modelData

                width: root.width
                height: Math.max(root.alturaMinima, linhaCasamento.implicitHeight + 6)

                Rectangle {
                    anchors.fill: parent
                    color: Theme.surface1
                    radius: Theme.radiusXSmall
                    border.width: 1
                    border.color: Theme.successSoft
                    opacity: 0.9
                }

                Column {
                    id: linhaCasamento

                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 1

                    Text {
                        width: parent.width
                        elide: Text.ElideRight
                        text: casamento.modelData.profileName + "  →  "
                              + casamento.modelData.dataSourceName
                        color: Theme.textPrimary
                        font.pixelSize: 11
                    }

                    // A JUSTIFICATIVA FICA VISIVEL. Um falso positivo aqui
                    // diria "seu banco ja' esta' observado" apontando para
                    // outro ambiente; mostrar a comparacao deixa o autor
                    // conferir sem abrir o Grafana.
                    Text {
                        width: parent.width
                        elide: Text.ElideRight
                        text: casamento.modelData.reason
                        color: Theme.textMuted
                        font.pixelSize: 9
                    }
                }
            }
        }
    }

    // A AUSENCIA TAMBEM E' RESPOSTA, e ela so' vale quando houve leitura.
    Text {
        width: parent.width
        wrapMode: Text.WordWrap
        visible: root.authenticated && root.matches.length === 0
                 && root.dataSources.length > 0
        text: qsTr("Nenhum banco deste projeto aparece nas fontes de dados do Grafana.")
        color: Theme.textMuted
        font.pixelSize: 10
    }

    Column {
        width: parent.width
        spacing: 2
        visible: root.dataSources.length > 0

        Titulo { text: qsTr("FONTES DE DADOS NO GRAFANA") }

        Repeater {
            model: root.dataSources

            delegate: Item {
                id: fonte

                required property var modelData

                width: root.width
                height: Math.max(root.alturaMinima - 6, textoFonte.implicitHeight + 4)

                Text {
                    id: textoFonte

                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideRight
                    text: "• " + fonte.modelData.name + "  "
                          + (fonte.modelData.typeName !== ""
                             ? fonte.modelData.typeName : fonte.modelData.typeId)
                          + (fonte.modelData.url !== "" ? "  " + fonte.modelData.url : "")
                          + (fonte.modelData.isDefault ? qsTr("  · padrão") : "")
                    color: Theme.textSecondary
                    font.pixelSize: 10
                }
            }
        }
    }

    Column {
        width: parent.width
        spacing: 2
        visible: root.dashboards.length > 0

        Titulo { text: qsTr("DASHBOARDS") }

        Repeater {
            model: root.dashboards

            delegate: Rectangle {
                id: painel

                required property var modelData

                width: root.width
                height: Math.max(root.alturaMinima - 6, textoPainel.implicitHeight + 4)
                radius: Theme.radiusXSmall
                color: areaPainel.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    id: textoPainel

                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingXSmall
                    anchors.rightMargin: Theme.spacingXSmall
                    anchors.verticalCenter: parent.verticalCenter
                    elide: Text.ElideRight
                    text: "↗ " + painel.modelData.title
                          + (painel.modelData.folderTitle !== ""
                             ? "  · " + painel.modelData.folderTitle : "")
                    color: areaPainel.containsMouse ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                }

                // ABRIR NO NAVEGADOR, e nao dentro da IDE. A licenca AGPL do
                // Grafana decide a forma da integracao: a IDE CONVERSA com ele,
                // nunca o embute (DocsPublic/integracoes/37 §2).
                MouseArea {
                    id: areaPainel

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.dashboardActivated(painel.modelData.url)
                }
            }
        }
    }
}
