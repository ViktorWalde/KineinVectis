pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A lista de containers e as imagens (E3-6, roadmaps/44): o filtro por
// nome/imagem/id, os containers na GRADE comum (estado · nome · imagem ·
// portas · status), a linha escolhida e UMA barra de acoes para ela — em
// vez de cinco icones por linha. Dono proprio pela mesma regra do
// EmbeddedSizeView: o painel cuida do motor e do compose.
//
// Burro: le do controller e pede por funcao dele. O que se oferece depende
// do ESTADO que o motor declarou: parar so' ao que roda, iniciar so' ao que
// parou, remover so' ao que parou (remover o que roda e' dois gestos).
Item {
    id: root

    property var controller: null

    readonly property bool hasSelection: controller !== null && controller.selected !== null
    readonly property bool running: controller !== null && controller.selectedRunning

    Column {
        id: conteudo

        anchors.fill: parent
        spacing: Theme.spacingSmall

        // O filtro e o titulo na mesma linha.
        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.controller && root.controller.filter.trim() !== ""
                      ? qsTr("Containers (%1 de %2)").arg(root.controller.visibleContainers.length).arg(root.controller.containers.length)
                      : qsTr("Containers")
                color: Theme.textSecondary
                font.pixelSize: 11
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 200
                height: 22
                radius: Theme.radiusXSmall
                color: Theme.background0
                border.width: 1
                border.color: filtro.activeFocus ? Theme.accent : Theme.borderSoft

                TextInput {
                    id: filtro

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.textPrimary
                    font.pixelSize: 11
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.controller.setFilter(text)
                    Keys.onEscapePressed: { text = ""; root.controller.setFilter(""); }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: filtro.text === ""
                        text: qsTr("filtrar por nome, imagem ou id")
                        color: Theme.textMuted
                        font.pixelSize: 11
                    }
                }
            }
        }

        KvDataGrid {
            id: grade

            width: parent.width
            visible: root.controller && root.controller.containerFound
            selectable: true
            selectedIndex: root.controller ? root.controller.selectedIndex : -1
            columns: [
                { key: "state", label: " " },
                { key: "name", label: qsTr("nome") },
                { key: "image", label: qsTr("imagem") },
                { key: "ports", label: qsTr("portas") },
                { key: "status", label: qsTr("status") }
            ]
            rows: root.controller ? root.controller.containerRows() : []
            maxHeight: 200
            onRowClicked: function(index) { root.controller.selectRow(index); }
        }

        // A barra da linha escolhida: o que o estado dela permite.
        Row {
            width: parent.width
            height: 26
            visible: root.controller && root.controller.containerFound
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.hasSelection ? root.controller.selectedTarget : qsTr("escolha um container")
                color: root.hasSelection ? Theme.textPrimary : Theme.textMuted
                font.family: Theme.monoFont
                font.pixelSize: 11
                elide: Text.ElideMiddle
                width: Math.min(implicitWidth, parent.width * 0.4)
            }

            KvButton {
                text: root.running ? qsTr("Parar") : qsTr("Iniciar")
                compact: true
                enabled: root.hasSelection
                onClicked: root.controller.act(root.controller.selectedTarget, root.running ? "stop" : "start")
            }

            KvButton {
                text: qsTr("Reiniciar")
                compact: true
                enabled: root.hasSelection && root.running
                onClicked: root.controller.act(root.controller.selectedTarget, "restart")
            }

            // Logs e shell nascem numa ABA DE TERMINAL, e a aba e' do projeto
            // aberto (o core exige workspace em container.open; medido de
            // novo em 2026-09-19). Sem projeto o botao diz o porque.
            KvButton {
                text: qsTr("Logs")
                compact: true
                enabled: root.hasSelection && root.controller.canOpenTerminals
                tooltip: root.controller && root.controller.canOpenTerminals
                         ? qsTr("Os logs numa aba do terminal")
                         : qsTr("Abra um projeto — a aba de terminal é do projeto")
                onClicked: root.controller.openLogs(root.controller.selectedTarget)
            }

            KvButton {
                text: qsTr("Shell")
                compact: true
                enabled: root.hasSelection && root.running && root.controller.canOpenTerminals
                tooltip: root.controller && root.controller.canOpenTerminals
                         ? qsTr("Um shell dentro do container (só rodando)")
                         : qsTr("Abra um projeto — a aba de terminal é do projeto")
                onClicked: root.controller.openShell(root.controller.selectedTarget)
            }

            KvButton {
                text: qsTr("Remover")
                compact: true
                danger: true
                enabled: root.hasSelection && !root.running
                tooltip: qsTr("Só parado: remover o que roda é dois gestos, de propósito")
                onClicked: root.controller.act(root.controller.selectedTarget, "remove")
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
