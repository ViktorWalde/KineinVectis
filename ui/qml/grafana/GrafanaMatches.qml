pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CRUZAMENTO: quais bancos DESTE projeto o Grafana ja' observa.
//
// E' a unica coisa no painel que exige as DUAS metades — o catalogo de fontes
// do Grafana e o catalogo de bancos do workspace —, e so' a IDE tem a segunda.
// Um link para o Grafana e' um favorito; isto e' integracao.
//
// Mora FORA da area que rola (§5.2 poe o cruzamento antes do filtro): sao duas
// ou tres linhas, e sao a manchete. Empurra-las para baixo de quarenta
// dashboards seria esconder a resposta atras do inventario.
//
// DIMENSIONAMENTO (decisao do autor, 2026-09-04): cada linha mede o proprio
// conteudo e tem um PISO. Altura fixa cortaria o nome longo; altura puramente
// automatica deixaria linhas de alturas diferentes, dificeis de varrer.
Column {
    id: root

    property var matches: []
    property var dataSources: []
    property bool authenticated: false

    // Altura minima de uma linha. Abaixo disto o alvo de clique fica menor que
    // o dedo/cursor consegue mirar com conforto.
    readonly property int alturaMinima: 26

    spacing: Theme.spacingSmall

    component Titulo: Text {
        color: Theme.textMuted
        font.pixelSize: Theme.fontSizeCaption
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
                        font.pixelSize: Theme.fontSizeSmall
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
                        font.pixelSize: Theme.fontSizeMicro
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
        font.pixelSize: Theme.fontSizeCaption
    }
}
