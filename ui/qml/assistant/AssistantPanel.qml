pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// KV Context: seletor e terminal dedicado para CLIs de IA externas. A IDE não
// embute chat nem API; Claude/Codex precisam estar instalados pelo usuário.
Rectangle {
    id: root

    property var profilesModel
    property string selectedProfileId: "claude"
    property string sessionId: ""
    property string activeProfileName: ""
    property string activeCommand: ""
    property var terminalRender: ({})
    property string errorText: ""
    property bool loading: false

    signal closeRequested()
    signal refreshRequested()
    signal profileSelected(string profileId)
    signal startRequested()
    signal switchRequested()
    signal exitRequested()
    signal terminalKeyPressed(string data)
    signal terminalResizeRequested(int cols, int rows)
    signal terminalScrollRequested(int offset)

    implicitWidth: 360
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            height: 30
            spacing: Theme.spacingSmall

            KvIcon {
                anchors.verticalCenter: parent.verticalCenter
                name: "context"
                size: 20
                active: true
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("KV Context")
                color: Theme.textPrimary
                font.pixelSize: Theme.fontSizePanelTitle
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: bridgeLabel.implicitWidth + 2 * Theme.spacingSmall
                height: 18
                radius: 9
                color: Theme.surface2

                Text {
                    id: bridgeLabel

                    anchors.centerIn: parent
                    text: qsTr("AI CLI externa")
                    color: Theme.textMuted
                    font.pixelSize: 9
                    font.bold: true
                }
            }

            Item {
                width: parent.width - x - closeButton.width
                height: 1
            }

            KvIconButton {
                id: closeButton

                width: 28
                height: 28
                iconName: "close"
                iconSize: 16
                tooltip: qsTr("Fechar KV Context")
                onClicked: root.closeRequested()
            }
        }

        Rectangle {
            width: parent.width
            height: parent.height - y
            radius: Theme.radius
            color: Theme.backgroundEditor
            border.color: Theme.borderSoft
            border.width: 1

            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingMedium
                spacing: Theme.spacingMedium
                visible: root.sessionId === ""

                Text {
                    width: parent.width
                    text: qsTr("Escolha a IA CLI")
                    color: Theme.textPrimary
                    font.pixelSize: 14
                    font.bold: true
                }

                Text {
                    width: parent.width
                    text: qsTr("Claude ou Codex deve estar previamente instalado e disponível no PATH. Abrir o painel nunca executa um comando automaticamente.")
                    color: Theme.textSecondary
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                }

                Repeater {
                    model: root.profilesModel

                    delegate: Rectangle {
                        id: profileCard

                        required property string profileId
                        required property string name
                        required property string command
                        required property bool available

                        width: parent.width
                        height: 64
                        radius: Theme.radius
                        color: root.selectedProfileId === profileId
                               ? Theme.surfaceSelected
                               : (profileMouse.containsMouse
                                  ? Theme.surface2 : Theme.background1)
                        border.color: root.selectedProfileId === profileId
                                      ? Theme.accent : Theme.borderSoft
                        border.width: 1
                        opacity: available ? 1.0 : 0.72

                        Column {
                            anchors.left: parent.left
                            anchors.leftMargin: Theme.spacingMedium
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: Theme.spacingXSmall

                            Text {
                                text: profileCard.name
                                color: profileCard.available
                                       ? Theme.textPrimary : Theme.textDisabled
                                font.pixelSize: 12
                                font.bold: true
                            }

                            Text {
                                text: profileCard.available
                                      ? qsTr("Comando detectado: %1").arg(profileCard.command)
                                      : qsTr("Não encontrado — instale %1 primeiro").arg(profileCard.command)
                                color: profileCard.available
                                       ? Theme.successSoft : Theme.warningSoft
                                font.family: Theme.monoFont
                                font.pixelSize: 10
                            }
                        }

                        MouseArea {
                            id: profileMouse

                            anchors.fill: parent
                            enabled: profileCard.available
                            hoverEnabled: true
                            cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                            onClicked: root.profileSelected(profileCard.profileId)
                        }
                    }
                }

                Text {
                    width: parent.width
                    visible: root.errorText !== ""
                    text: root.errorText
                    color: Theme.errorSoft
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                }

                Row {
                    spacing: Theme.spacingSmall

                    KvButton {
                        text: root.loading ? qsTr("Abrindo...") : qsTr("Iniciar")
                        iconName: "run"
                        primary: true
                        enabled: !root.loading
                        onClicked: root.startRequested()
                    }

                    KvButton {
                        text: qsTr("Redetectar")
                        iconName: "refresh"
                        enabled: !root.loading
                        onClicked: root.refreshRequested()
                    }
                }

                Text {
                    width: parent.width
                    text: qsTr("Outras ferramentas de IA entrarão depois deste ciclo funcional, por meio de um AI Terminal genérico com comando informado pelo usuário.")
                    color: Theme.textMuted
                    font.pixelSize: 10
                    wrapMode: Text.WordWrap
                }
            }

            Column {
                anchors.fill: parent
                anchors.margins: 1
                spacing: Theme.spacingXSmall
                visible: root.sessionId !== ""

                Row {
                    width: parent.width
                    height: 34
                    spacing: Theme.spacingSmall

                    Text {
                        anchors.left: parent.left
                        anchors.leftMargin: Theme.spacingSmall
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.activeProfileName
                        color: Theme.textPrimary
                        font.pixelSize: 12
                        font.bold: true
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        width: Math.max(0, parent.width - x - switchButton.width
                                        - exitButton.width - 3 * Theme.spacingSmall)
                        text: root.activeCommand
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: 9
                        elide: Text.ElideMiddle
                    }

                    KvButton {
                        id: switchButton

                        compact: true
                        text: qsTr("Trocar")
                        onClicked: root.switchRequested()
                    }

                    KvButton {
                        id: exitButton

                        compact: true
                        text: qsTr("Sair")
                        onClicked: root.exitRequested()
                    }
                }

                TerminalPanel {
                    width: parent.width
                    height: parent.height - 34 - parent.spacing
                    render: root.terminalRender
                    terminalActive: root.sessionId !== ""
                    workspaceAvailable: true
                    emptyText: qsTr("Inicializando %1...").arg(root.activeProfileName)
                    onKeyPressed: function(data) {
                        root.terminalKeyPressed(data);
                    }
                    onResizeRequested: function(cols, rows) {
                        root.terminalResizeRequested(cols, rows);
                    }
                    onScrollRequested: function(offset) {
                        root.terminalScrollRequested(offset);
                    }
                }
            }
        }
    }
}
