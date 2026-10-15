pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O CONTAINER ESCOLHIDO, embaixo da lista da janela de Containers
// (2026-10-03, inspiracao no Docker Desktop: escolher um container mostra o
// que ele e' e o que se faz com ele, sem sair da lista). Estado, imagem, id,
// status do motor, as portas — as do host viram link para o navegador,
// so' `localhost` — e as acoes que o ESTADO permite: parar so' o que roda,
// iniciar e remover so' o que parou.
Item {
    id: root

    property var row: null
    property var controller: null

    signal closeRequested()

    readonly property bool running: root.row !== null && ContainerStates.isActive(root.row.state)
    readonly property bool terminals: root.controller !== null && root.controller.canOpenTerminals
    readonly property bool busy: root.row !== null && root.controller !== null && root.controller.isPending(root.row.target)
    property bool confirmRemove: false
    onRowChanged: root.confirmRemove = false

    // O "Remover" que pergunta (16b.3, roadmaps/59 §2.5.2): armado, com volume
    // anonimo, a escolha "com os volumes" ou "so' o container".
    readonly property bool volumesKnown: root.row !== null && root.controller !== null
                                         && root.controller.volumesKnown(root.row.target)
    readonly property int anonymousVolumes: root.volumesKnown
                                            ? root.controller.removeChoice(root.row.target).anonymous : 0
    readonly property bool asksVolumes: root.confirmRemove && root.anonymousVolumes > 0

    // O segundo clique do Remover tem prazo (o mesmo da linha); com a
    // escolha, o dobro: sao duas frases para ler.
    Timer {
        interval: Theme.motionProgress * (root.asksVolumes ? 8 : 4)
        running: root.confirmRemove
        onTriggered: root.confirmRemove = false
    }

    // A linha leva a pergunta para ca' (a linha nao tem largura para ela).
    Connections {
        target: root.controller
        function onRemoveAskedChanged() {
            if (root.row !== null && root.controller.removeAsked !== ""
                    && root.controller.removeAsked === root.row.target) {
                root.controller.removeAsked = "";
                root.confirmRemove = true;
            }
        }
    }

    implicitHeight: column.implicitHeight

    Column {
        id: column

        width: parent.width
        spacing: Theme.spacingXSmall

        Item {
            width: parent.width
            height: 22

            Rectangle {
                id: stateDot

                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: 8
                height: 8
                radius: width / 2
                color: root.row ? ContainerStates.color(root.row.state) : Theme.textMuted
            }

            Text {
                anchors.left: stateDot.right
                anchors.leftMargin: Theme.spacingSmall
                anchors.right: closeButton.left
                anchors.verticalCenter: parent.verticalCenter
                text: root.row ? root.row.name + "  ·  " + ContainerStates.label(root.row.state) : ""
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizeSmall
                font.weight: Font.DemiBold
                elide: Text.ElideRight
            }

            KvIconButton {
                id: closeButton

                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                iconName: "close"
                iconSize: 12
                tooltip: qsTr("Fechar o detalhe")
                onClicked: root.closeRequested()
            }
        }

        // Imagem, id e o status do motor, uma linha cada: o que se copia.
        Repeater {
            model: root.row === null ? [] : [
                { label: qsTr("imagem"), value: root.row.image },
                { label: qsTr("id"), value: root.row.id.substring(0, 12) },
                { label: qsTr("status"), value: root.row.status }
            ]

            delegate: Row {
                id: field

                required property var modelData

                width: column.width
                spacing: Theme.spacingSmall

                Text {
                    width: 52
                    text: field.modelData.label
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeCaption
                }

                Text {
                    width: parent.width - 52 - parent.spacing
                    text: field.modelData.value
                    color: Theme.textSecondary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                    elide: Text.ElideMiddle
                }
            }
        }

        // As portas: "8080 → 80/tcp"; a do host abre no navegador.
        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall
            visible: root.row !== null && root.row.ports.length > 0

            Repeater {
                model: root.row ? root.row.ports : []

                delegate: Rectangle {
                    id: portChip

                    required property var modelData

                    width: portText.implicitWidth + 2 * Theme.spacingSmall
                    height: 20
                    radius: Theme.radius
                    readonly property bool link: portChip.modelData.url !== "" && root.running

                    color: portArea.containsMouse && portChip.link ? Theme.surface2 : Theme.background0
                    border.width: 1
                    border.color: Theme.borderSoft

                    Text {
                        id: portText

                        anchors.centerIn: parent
                        text: (portChip.modelData.host !== "" ? portChip.modelData.host + " → " : "")
                              + portChip.modelData.target
                        color: portChip.link ? Theme.accent : Theme.textSecondary
                        font.family: Theme.monoFont
                        font.pixelSize: Theme.fontSizeCaption
                    }

                    MouseArea {
                        id: portArea

                        anchors.fill: parent
                        hoverEnabled: true
                        enabled: portChip.link
                        cursorShape: Qt.PointingHandCursor
                        onClicked: Qt.openUrlExternally(portChip.modelData.url)
                    }
                }
            }
        }

        Flow {
            width: parent.width
            spacing: Theme.spacingXSmall
            // Enquanto o motor trabalha, nada se clica de novo.
            enabled: !root.busy

            KvButton {
                compact: true
                primary: !root.running
                iconName: root.running ? "stop" : "run"
                text: root.running ? qsTr("Parar") : qsTr("Iniciar")
                onClicked: root.controller.act(root.row.target, root.running ? "stop" : "start")
            }

            KvButton {
                compact: true
                iconName: "refresh"
                text: qsTr("Reiniciar")
                enabled: root.running
                onClicked: root.controller.act(root.row.target, "restart")
            }

            KvButton {
                compact: true
                iconName: "documents"
                text: qsTr("Logs")
                enabled: root.terminals
                tooltip: root.terminals ? "" : qsTr("Abra um projeto: a aba de terminal é do projeto")
                onClicked: root.controller.openLogs(root.row.target)
            }

            KvButton {
                compact: true
                iconName: "terminal"
                text: qsTr("Shell")
                enabled: root.running && root.terminals
                onClicked: root.controller.openShell(root.row.target)
            }

            // Remover arma no primeiro clique e pergunta pelos volumes; o
            // segundo, dentro do prazo, age. Antes da resposta, nada se
            // confirma: o clique removeria so' o container (o defeito 1).
            KvButton {
                compact: true
                danger: true
                visible: !root.asksVolumes
                iconName: root.confirmRemove ? "check" : "trash"
                text: !root.confirmRemove ? qsTr("Remover")
                      : (root.volumesKnown ? qsTr("Confirmar remoção") : qsTr("verificando volumes…"))
                enabled: !root.running && (!root.confirmRemove || root.volumesKnown)
                onClicked: {
                    if (!root.confirmRemove) {
                        root.confirmRemove = true;
                        root.controller.requestVolumes(root.row.target);
                        return;
                    }
                    root.confirmRemove = false;
                    root.controller.act(root.row.target, "remove", false);
                }
            }

            KvButton {
                compact: true
                danger: true
                visible: root.asksVolumes
                iconName: "trash"
                text: root.asksVolumes ? root.controller.removeWithVolumesText(root.row.target) : ""
                onClicked: {
                    root.confirmRemove = false;
                    root.controller.act(root.row.target, "remove", true);
                }
            }

            KvButton {
                compact: true
                visible: root.asksVolumes
                text: qsTr("Só o container")
                onClicked: {
                    root.confirmRemove = false;
                    root.controller.act(root.row.target, "remove", false);
                }
            }
        }
    }
}
