pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A lista de containers com as acoes de ciclo de vida, e as imagens. Dono
// proprio pela mesma regra do EmbeddedSizeView: o painel cuida do motor e do
// compose; listar e agir por container e' outra responsabilidade.
//
// Burro: le do controller e pede por funcao dele. O que se oferece depende do
// ESTADO que o motor declarou: parar so' ao que roda, iniciar so' ao que parou,
// remover so' ao que parou (remover o que roda e' dois gestos, de proposito).
Item {
    id: root

    property var controller: null

    Flickable {
        anchors.fill: parent
        contentHeight: conteudo.implicitHeight
        clip: true

        Column {
            id: conteudo

            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                text: qsTr("Containers")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            Repeater {
                model: root.controller ? root.controller.containers : []

                Row {
                    id: linha

                    required property var modelData

                    width: conteudo.width
                    spacing: Theme.spacingSmall

                    readonly property bool rodando: root.controller.isRunning(linha.modelData)
                    readonly property string alvo: linha.modelData.names !== undefined
                                                   && linha.modelData.names.length > 0
                                                   ? linha.modelData.names[0] : linha.modelData.id

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 8
                        height: 8
                        radius: 4
                        color: linha.rodando ? Theme.successSoft : Theme.textDisabled
                    }

                    Column {
                        width: parent.width - 8 - 5 * 26 - 6 * Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter

                        Text {
                            width: parent.width
                            text: root.controller.containerSummary(linha.modelData)
                            color: Theme.textPrimary
                            font.family: Theme.monoFont
                            font.pixelSize: 11
                            elide: Text.ElideMiddle
                        }

                        Text {
                            width: parent.width
                            text: linha.modelData.status !== undefined ? linha.modelData.status : ""
                            color: Theme.textMuted
                            font.pixelSize: 10
                            elide: Text.ElideRight
                        }
                    }

                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: linha.rodando ? "stop" : "run"
                        tooltip: linha.rodando ? qsTr("Parar") : qsTr("Iniciar")
                        compact: true
                        onClicked: root.controller.act(linha.alvo, linha.rodando ? "stop" : "start")
                    }

                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "refresh"
                        tooltip: qsTr("Reiniciar")
                        compact: true
                        enabled: linha.rodando
                        onClicked: root.controller.act(linha.alvo, "restart")
                    }

                    // Logs e shell nascem numa ABA DE TERMINAL, e a aba e' do
                    // projeto aberto: sem workspace o core recusava com
                    // "nenhum workspace aberto" DEPOIS do clique (medido em
                    // 2026-09-13). Agora o botao diz antes.
                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "file"
                        tooltip: root.controller.canOpenTerminals
                                 ? qsTr("Logs (aba de terminal)")
                                 : qsTr("Logs: abra um projeto — a aba de terminal é do projeto")
                        compact: true
                        enabled: root.controller.canOpenTerminals
                        onClicked: root.controller.openLogs(linha.alvo)
                    }

                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "terminal"
                        tooltip: root.controller.canOpenTerminals
                                 ? qsTr("Shell dentro do container (só rodando)")
                                 : qsTr("Shell: abra um projeto — a aba de terminal é do projeto")
                        compact: true
                        enabled: linha.rodando && root.controller.canOpenTerminals
                        onClicked: root.controller.openShell(linha.alvo)
                    }

                    KvIconButton {
                        anchors.verticalCenter: parent.verticalCenter
                        iconName: "close"
                        tooltip: qsTr("Remover (só parado)")
                        compact: true
                        danger: true
                        enabled: !linha.rodando
                        onClicked: root.controller.act(linha.alvo, "remove")
                    }
                }
            }

            Text {
                width: parent.width
                wrapMode: Text.WordWrap
                visible: root.controller && !root.controller.containerFound
                text: root.controller
                      ? (root.controller.listBusy ? qsTr("procurando…")
                         : (root.controller.containersHint !== "" ? root.controller.containersHint
                                                                  : qsTr("nenhum container")))
                      : ""
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            // A saida CRUA aparece quando nada foi reconhecido: e' o que deixa
            // o usuario ver se ha' um container ali e o formato mudou.
            Text {
                width: parent.width
                wrapMode: Text.WrapAnywhere
                visible: root.controller && !root.controller.containerFound
                         && root.controller.containersRaw.trim() !== "" && !root.controller.listBusy
                text: root.controller ? root.controller.containersRaw.trim() : ""
                color: Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 10
            }

            Text {
                text: qsTr("Imagens")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            // As imagens na GRADE comum (F8): repositorio, tag, tamanho, criada.
            KvDataGrid {
                width: parent.width
                visible: root.controller && root.controller.images.length > 0
                columns: [
                    { key: "repository", label: qsTr("repositório") },
                    { key: "tag", label: qsTr("tag") },
                    { key: "size", label: qsTr("tamanho") },
                    { key: "created", label: qsTr("criada") }
                ]
                rows: root.controller ? root.controller.imageRows() : []
                maxHeight: 140
            }

            Text {
                width: parent.width
                visible: root.controller && root.controller.images.length === 0
                text: root.controller
                      ? (root.controller.imagesBusy ? qsTr("procurando…")
                         : (root.controller.imagesHint !== "" ? root.controller.imagesHint
                                                              : qsTr("nenhuma imagem")))
                      : ""
                color: Theme.textSecondary
                font.pixelSize: 11
            }
        }
    }
}
