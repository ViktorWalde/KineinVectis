pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// A barra de sessoes da aba Terminal: um chip por PTY aberto, mais "Execucao",
// "+" e "limpar".
//
// Saiu do BottomPanelHost em 2026-07-17. Nao foi por tamanho: aquele host compoe
// PAINEIS, e cada painel dele ja e um componente proprio (GitPanel, SearchPanel,
// TerminalPanel...). Esta barra era a unica escrita inline — 183 linhas, maior
// que qualquer painel — entao extrai-la e aplicar a regra que o arquivo ja
// seguia, nao inventar estrutura.
Row {
    id: root

    // D2.3: uma linha por terminal aberto — { termId, title }.
    property var sessionsModel: null
    property string activeTerminalId: ""
    property string terminalSession: "shell"
    property bool running: false

    signal selectRequested(string id)
    signal closeRequested(string id)
    signal newRequested()
    signal sessionRequested(string session)
    signal clearRequested()

    spacing: Theme.spacingSmall

    // D2.3 (docs/roadmaps/24): uma aba por terminal aberto. A "Execução" continua
    // sendo uma sessão à parte — ela NÃO é um PTY (é o backend run.*),
    // por isso não entra no mesmo modelo.
    Repeater {
        model: root.sessionsModel

        delegate: Rectangle {
            id: termChip

            required property string termId
            required property string title

            readonly property bool current: root.terminalSession === "shell"
                                            && root.activeTerminalId === termChip.termId

            width: termChipRow.width + 2 * Theme.spacingSmall
            height: 20
            radius: Theme.radiusXSmall
            color: termChip.current ? Theme.surfaceSelected : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            // Fica ATRÁS do conteúdo (z: -1) pra não engolir o clique do
            // ícone: selecionar e fechar são gestos diferentes na mesma aba.
            MouseArea {
                anchors.fill: parent
                z: -1
                cursorShape: Qt.PointingHandCursor
                onClicked: root.selectRequested(termChip.termId)
            }

            Row {
                id: termChipRow

                anchors.centerIn: parent
                spacing: Theme.spacingXSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: termChip.title
                    color: termChip.current ? Theme.accent : Theme.textSecondary
                    font.pixelSize: 10
                    font.bold: true
                }

                KvIcon {
                    id: termCloseLabel

                    anchors.verticalCenter: parent.verticalCenter
                    name: "close"
                    size: 14
                    iconColor: termCloseArea.containsMouse
                               ? Theme.textPrimary : Theme.textMuted

                    MouseArea {
                        id: termCloseArea

                        anchors.fill: parent
                        anchors.margins: -3
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.closeRequested(termChip.termId)
                    }
                }
            }
        }
    }

    // Novo terminal.
    Rectangle {
        width: 20
        height: 20
        radius: Theme.radiusXSmall
        color: newTerminalArea.containsMouse ? Theme.surface2 : "transparent"
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            anchors.centerIn: parent
            text: "+"
            color: newTerminalArea.containsMouse ? Theme.accent : Theme.textSecondary
            font.pixelSize: 12
            font.bold: true
        }

        MouseArea {
            id: newTerminalArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.newRequested()
        }
    }

    Rectangle {
        id: runSessionChip

        readonly property bool current: root.terminalSession === "run"

        width: runSessionRow.width + 2 * Theme.spacingSmall
        height: 20
        radius: Theme.radiusXSmall
        color: runSessionChip.current ? Theme.surfaceSelected : "transparent"
        border.color: Theme.borderSoft
        border.width: 1

        Row {
            id: runSessionRow
            anchors.centerIn: parent
            spacing: Theme.spacingXSmall

            Text {
                id: runSessionLabel

                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Execução")
                color: runSessionChip.current ? Theme.accent : Theme.textSecondary
                font.pixelSize: 10
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                visible: root.running
                width: 6
                height: 6
                radius: 3
                color: Theme.successSoft
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: root.sessionRequested("run")
        }
    }

    Rectangle {
        width: clearSessionLabel.width + 2 * Theme.spacingSmall
        height: 20
        radius: Theme.radiusXSmall
        color: clearSessionArea.containsMouse ? Theme.surface2 : "transparent"
        border.color: Theme.borderSoft
        border.width: 1

        Text {
            id: clearSessionLabel

            anchors.centerIn: parent
            text: qsTr("limpar")
            color: Theme.textMuted
            font.pixelSize: 10
        }

        MouseArea {
            id: clearSessionArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.clearRequested()
        }
    }
}
