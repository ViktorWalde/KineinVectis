pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A coluna da esquerda: o que responde NESTA MAQUINA (descoberto pelo core:
// servidor no loopback, container de banco, arquivo SQLite do projeto) e
// os perfis salvos neste workspace (0.124.0 — antes so' os salvos).
//
// Cada perfil mostra a linha de conexao (`usuario@host:porta/banco`), porque
// dois perfis "local" e "local-2" nao se distinguem pelo nome — e' o destino
// que o autor precisa ver antes de clicar em Testar. Clicar num descoberto
// poe o perfil dele no formulario; salvar e' do autor.
Item {
    id: root

    property var profiles: []
    property string selectedName: ""
    property var candidates: []
    property bool discovering: false
    property string discoverHint: ""

    signal profileSelected(string name)
    signal candidateSelected(int index)
    signal discoverRequested()
    signal newRequested()
    signal createRequested()

    Flickable {
        id: rolagem

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: rodape.top
        anchors.bottomMargin: Theme.spacingSmall
        clip: true
        contentWidth: width
        contentHeight: coluna.implicitHeight
        boundsBehavior: Flickable.StopAtBounds

        Column {
            id: coluna

            width: rolagem.width
            spacing: 2

            Row {
                width: parent.width
                height: 22

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.discovering ? qsTr("Nesta máquina — procurando…") : qsTr("Nesta máquina")
                    color: Theme.textMuted
                    font.pixelSize: 10
                }

                Item { width: parent.width - x - atualizar.width; height: 1 }

                KvIconButton {
                    id: atualizar

                    anchors.verticalCenter: parent.verticalCenter
                    compact: true
                    iconName: "refresh"
                    tooltip: qsTr("Procurar de novo (sockets, portas, containers, .sqlite)")
                    enabled: !root.discovering
                    onClicked: root.discoverRequested()
                }
            }

            Repeater {
                model: root.candidates

                delegate: Rectangle {
                    id: achado

                    required property int index
                    required property var modelData

                    width: coluna.width
                    height: 32
                    radius: Theme.radius
                    color: areaAchado.containsMouse ? Theme.surface2 : "transparent"

                    Rectangle {
                        id: bolinha

                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        width: 7
                        height: 7
                        radius: 3.5
                        color: achado.modelData.running ? Theme.successSoft : Theme.textDisabled
                    }

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: bolinha.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.right: parent.right
                        anchors.rightMargin: Theme.spacingSmall

                        Text {
                            width: parent.width
                            text: achado.modelData.label
                            color: Theme.textPrimary
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            text: achado.modelData.detail
                            color: Theme.textMuted
                            font.family: Theme.monoFont
                            font.pixelSize: 9
                            elide: Text.ElideMiddle
                        }
                    }

                    MouseArea {
                        id: areaAchado

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.candidateSelected(achado.index)
                    }
                }
            }

            Text {
                width: parent.width
                visible: !root.discovering && root.candidates.length === 0
                wrapMode: Text.WordWrap
                text: root.discoverHint !== "" ? root.discoverHint : qsTr("nada respondeu")
                color: Theme.textMuted
                font.pixelSize: 10
            }

            Item { width: 1; height: Theme.spacingSmall }

            Text {
                text: qsTr("Salvos")
                color: Theme.textMuted
                font.pixelSize: 10
            }

            Repeater {
                model: root.profiles

                delegate: Rectangle {
                    id: linha

                    required property var modelData

                    width: coluna.width
                    height: 32
                    radius: Theme.radius
                    color: linha.modelData.name === root.selectedName
                           ? Theme.surfaceSelected
                           : (area.containsMouse ? Theme.surface2 : "transparent")

                    Column {
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.rightMargin: Theme.spacingSmall

                        Text {
                            width: parent.width
                            text: linha.modelData.name
                            color: Theme.textPrimary
                            font.pixelSize: 11
                            elide: Text.ElideRight
                        }

                        Text {
                            width: parent.width
                            // Sem host e' arquivo: a linha e' o caminho.
                            text: linha.modelData.host === ""
                                  ? linha.modelData.database
                                  : linha.modelData.user + "@" + linha.modelData.host
                                    + ":" + linha.modelData.port + "/" + linha.modelData.database
                            color: Theme.textMuted
                            font.family: Theme.monoFont
                            font.pixelSize: 9
                            elide: Text.ElideMiddle
                        }
                    }

                    MouseArea {
                        id: area

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.profileSelected(linha.modelData.name)
                    }
                }
            }

            Text {
                width: parent.width
                visible: root.profiles.length === 0
                wrapMode: Text.WordWrap
                text: qsTr("Nenhum perfil salvo. Clique num descoberto, ou preencha ao lado e salve.")
                color: Theme.textMuted
                font.pixelSize: 10
            }
        }
    }

    Row {
        id: rodape

        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Theme.spacingSmall

        KvButton {
            width: (parent.width - parent.spacing) / 2
            compact: true
            text: qsTr("Nova fonte")
            onClicked: root.newRequested()
        }

        KvButton {
            width: (parent.width - parent.spacing) / 2
            compact: true
            primary: true
            text: qsTr("Novo banco")
            onClicked: root.createRequested()
        }
    }
}
