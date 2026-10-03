pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Os recentes da tela de boas-vindas (Etapa 2 F7; mais interativos em
// 2026-10-03, pedido do autor): o ultimo aberto em destaque (Enter abre), os
// de caminho ausente ocultos com desfazer. A logica e' do
// RecentWorkspacesController; aqui so' a forma.
//
//   Projetos recentes  (3)                                   Limpar
//   ▌[▣] meu-app                                  há 2 h   📌  ×
//        ~/projetos/meu-app
//
// Cada linha acende ao pairar (transicao curta, barra ambar a esquerda), e o
// mouse e o teclado falam do MESMO destaque: pairar destaca, Enter abre o
// destacado. Fixar e remover aparecem com o mouse; o fixado tem a placa em
// ambar e fica sempre no topo (o controller ordena).
Rectangle {
    id: root

    required property var controller

    readonly property var shown: controller.visibleWorkspaces.slice(0, 5)

    height: header.height + list.height + 2 * Theme.spacingMedium + Theme.spacingXSmall
            + (missing.visible ? missing.height : 0) + (errorLine.visible ? errorLine.height : 0)
    radius: Theme.radiusLarge
    // Translucido: a arte da tela de boas-vindas aparece por baixo.
    color: Qt.rgba(Theme.background1.r, Theme.background1.g, Theme.background1.b, 0.86)
    border.color: Theme.borderSoft
    border.width: 1

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingXSmall

        Item {
            id: header

            width: parent.width
            height: 28

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Projetos recentes")
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizePanelTitle
                    font.bold: true
                }

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: root.controller.workspaces.length > 0
                    width: countText.implicitWidth + 2 * Theme.spacingSmall
                    height: 16
                    radius: height / 2
                    color: Theme.surface2

                    Text {
                        id: countText

                        anchors.centerIn: parent
                        text: root.controller.visibleWorkspaces.length
                        color: Theme.textSecondary
                        font.pixelSize: Theme.fontSizeMicro
                    }
                }
            }

            KvButton {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                visible: root.controller.workspaces.length > 0
                text: qsTr("Limpar")
                onClicked: root.controller.clearAll()
            }
        }

        Column {
            id: list

            width: parent.width
            spacing: 2

            // Vazio: o que vai aparecer aqui, e por onde comecar.
            Row {
                visible: root.shown.length === 0
                height: 44
                spacing: Theme.spacingSmall

                KvIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    name: "recent"
                    size: 18
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: qsTr("Os projetos que você abrir aparecem aqui — comece criando ou abrindo um, acima.")
                    color: Theme.textMuted
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            Repeater {
                model: root.shown

                delegate: RecentWorkspaceRow {
                    width: list.width
                    controller: root.controller
                }
            }
        }

        Row {
            id: missing

            visible: root.controller.hiddenMissingCount > 0
            width: parent.width
            height: 28
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: root.controller.hiddenMissingCount === 1
                      ? qsTr("1 recente sem caminho foi ocultado")
                      : qsTr("%1 recentes sem caminho foram ocultados").arg(root.controller.hiddenMissingCount)
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeCaption
            }

            KvButton {
                anchors.verticalCenter: parent.verticalCenter
                compact: true
                text: qsTr("Desfazer")
                onClicked: root.controller.restoreMissing()
            }
        }

        Text {
            id: errorLine

            visible: root.controller.errorText !== ""
            width: parent.width
            text: root.controller.errorText
            color: Theme.errorSoft
            font.pixelSize: Theme.fontSizeCaption
            elide: Text.ElideRight
        }
    }
}
