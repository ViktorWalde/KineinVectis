pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// Escolha da CLI de IA do Assistente. Reconstruido do AssistantPanel que saiu no
// 0.59.0 (git show ab3becc^:ui/qml/assistant/AssistantPanel.qml) — o card list
// era bom e foi removido por associacao, junto com o mecanismo ruim.
//
// CUIDADO COM O NOME. O produto voltou a se chamar "Assistente" em 2026-07-17, e
// o antigo `ui/qml/assistant/` tinha justamente `AssistantPanel` +
// `AssistantController`. NAO sao a mesma coisa e o nome parecido nao autoriza
// ressuscitar o que morreu: aquele painel era um terminal PARALELO.
//
// SO A ESCOLHA VOLTOU. O painel antigo carregava terminalRender,
// terminalKeyPressed, terminalResizeRequested e terminalScrollRequested: um
// TERMINAL PARALELO ao terminal de verdade. E isso que o §0.2c proibiu ("sem
// criar input ou terminal paralelo") e o que custou dois dias de estabilizacao
// de TUI.
//
// Este componente so existe ANTES da sessao. Escolhido o agente, ele some e o
// que roda e o terminal normal: mesmo terminal.open, mesmo renderer, mesma
// grade, mesma roda, mesmo cursor.
Rectangle {
    id: root

    property var agentsModel: null

    signal chosen(string agentId)
    signal dismissRequested()
    signal redetectRequested()

    implicitWidth: 420
    implicitHeight: conteudo.implicitHeight + 2 * Theme.spacingMedium
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Column {
        id: conteudo

        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingMedium

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            KvIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: "assistant"
                size: 20
                active: true
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Escolha a IA CLI")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizePanelTitle
                font.bold: true
            }
        }

        Text {
            width: parent.width
            text: qsTr("A CLI precisa estar instalada e disponível no PATH. A Kinein não "
                       + "embute chat nem API, e abrir este seletor não executa nada.")
            color: Theme.textSecondary
            font.pixelSize: 11
            wrapMode: Text.WordWrap
        }

        Repeater {
            model: root.agentsModel

            delegate: Rectangle {
                id: card

                required property string agentId
                required property string displayName
                required property string detail
                required property bool available

                width: conteudo.width
                height: 62
                radius: Theme.radius
                color: cardMouse.containsMouse && card.available
                       ? Theme.surface2 : Theme.background1
                border.color: Theme.borderSoft
                border.width: 1
                opacity: card.available ? 1.0 : 0.72

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingMedium
                    anchors.rightMargin: Theme.spacingMedium
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: Theme.spacingXSmall

                    Text {
                        text: card.displayName
                        color: card.available ? Theme.textPrimary : Theme.textDisabled
                        font.pixelSize: 12
                        font.bold: true
                    }

                    Text {
                        width: parent.width
                        // Detectado mostra ONDE esta; ausente mostra como
                        // instalar — e so texto, a Kinein nunca roda a sugestao.
                        text: card.available
                              ? qsTr("Comando detectado: %1").arg(card.detail)
                              : (card.detail !== ""
                                 ? qsTr("Não encontrado — instale com: %1").arg(card.detail)
                                 : qsTr("Não encontrado no PATH"))
                        color: card.available ? Theme.successSoft : Theme.warningSoft
                        font.family: Theme.monoFont
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                    }
                }

                MouseArea {
                    id: cardMouse

                    anchors.fill: parent
                    enabled: card.available
                    hoverEnabled: true
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onClicked: root.chosen(card.agentId)
                }
            }
        }

        Row {
            spacing: Theme.spacingSmall

            KvButton {
                text: qsTr("Redetectar")
                iconName: "refresh"
                onClicked: root.redetectRequested()
            }

            KvButton {
                text: qsTr("Cancelar")
                onClicked: root.dismissRequested()
            }
        }
    }
}
