pragma ComponentBehavior: Bound
import QtQuick

// A tela inicial (Etapa 2 F7, 2026-09-18): comecar em 1 clique — ou em
// Enter, no ultimo recente em destaque — e o ambiente numa linha ("37 de
// 62 ferramentas · Ver"); a lista inteira fica no painel Ferramentas.
Rectangle {
    id: root

    property var tools: []
    required property var recentWorkspacesController
    property bool scanning: false

    signal openWorkspaceRequested()
    signal newProjectRequested(string templateId)
    signal settingsRequested()
    signal detectToolsRequested()
    signal toolsPanelRequested()

    focus: visible
    // Sem workspace, o teclado e' desta tela: Enter abre o recente em destaque.
    onVisibleChanged: if (visible) forceActiveFocus()
    Component.onCompleted: if (visible) forceActiveFocus()
    Keys.onUpPressed: recentWorkspacesController.moveHighlight(-1)
    Keys.onDownPressed: recentWorkspacesController.moveHighlight(1)
    Keys.onReturnPressed: recentWorkspacesController.openHighlighted()
    Keys.onEnterPressed: recentWorkspacesController.openHighlighted()

    color: Theme.backgroundEditor
    radius: Theme.radiusLarge
    border.color: Theme.borderSoft
    border.width: 1

    function detectedCount() {
        let count = 0;
        for (let i = 0; i < tools.length; i++) {
            if (tools[i].status === "detected" || tools[i].status === "ready") {
                count++;
            }
        }
        return count;
    }

    Flickable {
        id: startFlick

        anchors.fill: parent
        contentWidth: width
        contentHeight: Math.max(height,
                                contentColumn.y + contentColumn.height
                                + Theme.spacingRegion)
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.VerticalFlick

        Column {
            id: contentColumn

            x: (startFlick.width - width) / 2
            y: Math.max(Theme.spacingRegion,
                        (startFlick.height - height) / 2)
            width: Math.min(760, startFlick.width - 2 * Theme.spacingRegion)
            spacing: Theme.spacingLarge

        Row {
            width: parent.width
            height: 54
            spacing: Theme.spacingMedium

            Column {
                anchors.verticalCenter: parent.verticalCenter

                Text {
                    text: qsTr("Kinein Vectis")
                    color: Theme.textPrimary
                    font.pixelSize: 22
                    font.bold: true
                }

                Text {
                    text: qsTr("IDE para C, C++, Rust, Python e sistemas embarcados")
                    color: Theme.textSecondary
                    font.pixelSize: 12
                }
            }
        }

            Rectangle {
            width: parent.width
            height: 142
            radius: Theme.radiusLarge
            color: Theme.background1
            border.color: Theme.borderSoft
            border.width: 1

            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingMedium

                Text {
                    text: qsTr("Começar")
                    color: Theme.textPrimary
                    font.pixelSize: Theme.fontSizePanelTitle
                    font.bold: true
                }

                Row {
                    spacing: Theme.spacingSmall

                    KvButton {
                        text: qsTr("Novo C++ / CMake")
                        iconName: "configure"
                        primary: true
                        onClicked: root.newProjectRequested("cppCmake")
                    }

                    KvButton {
                        text: qsTr("Novo Rust / Cargo")
                        iconName: "build"
                        onClicked: root.newProjectRequested("rustCargo")
                    }

                    KvButton {
                        text: qsTr("Abrir workspace")
                        iconName: "project"
                        onClicked: root.openWorkspaceRequested()
                    }

                    KvButton {
                        text: qsTr("Configurações")
                        iconName: "settings"
                        onClicked: root.settingsRequested()
                    }
                }

                Text {
                    text: qsTr("Projetos novos mostram os arquivos reais antes de abrir; projetos existentes não são alterados automaticamente.")
                    color: Theme.textMuted
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                    width: parent.width
                }
            }
        }

            RecentWorkspacesCard {
                width: parent.width
                controller: root.recentWorkspacesController
            }

            Rectangle {
                width: parent.width
                height: 64
                radius: Theme.radiusLarge
                color: Theme.background1
                border.color: Theme.borderSoft
                border.width: 1

                Row {
                    anchors.fill: parent
                    anchors.margins: Theme.spacingLarge
                    spacing: Theme.spacingMedium

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 8
                        height: 8
                        radius: 4
                        color: root.tools.length === 0 ? Theme.textMuted
                               : root.detectedCount() === root.tools.length
                                 ? Theme.successSoft : Theme.warningSoft
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: root.tools.length === 0
                              ? qsTr("Ambiente: compiladores, CMake, depuradores e ferramentas ainda não detectados — a IDE só inspeciona, nunca instala sem confirmação.")
                              : qsTr("Ambiente: %1 de %2 ferramentas detectadas")
                                .arg(root.detectedCount()).arg(root.tools.length)
                        color: Theme.textSecondary
                        font.pixelSize: 12
                        width: parent.width - x - environmentActions.width - parent.spacing
                        elide: Text.ElideRight
                    }

                    Row {
                        id: environmentActions

                        anchors.verticalCenter: parent.verticalCenter
                        spacing: Theme.spacingSmall

                        KvButton {
                            compact: true
                            visible: root.tools.length > 0
                            text: qsTr("Ver")
                            iconName: "tools"
                            onClicked: root.toolsPanelRequested()
                        }

                        KvButton {
                            compact: true
                            enabled: !root.scanning
                            text: root.scanning ? qsTr("Detectando...") : qsTr("Redetectar")
                            iconName: "refresh"
                            onClicked: root.detectToolsRequested()
                        }
                    }
                }
            }
        }
    }
}
