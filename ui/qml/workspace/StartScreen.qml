pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property var tools: []
    property var recentWorkspaces: []
    property string recentWorkspacesError: ""
    property bool scanning: false

    signal openWorkspaceRequested()
    signal recentWorkspaceOpenRequested(string rootPath)
    signal recentWorkspacePinRequested(string rootPath)
    signal recentWorkspaceRemoveRequested(string rootPath)
    signal recentWorkspacesClearRequested()
    signal newProjectRequested(string templateId)
    signal settingsRequested()
    signal detectToolsRequested()

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
                workspaces: root.recentWorkspaces
                errorText: root.recentWorkspacesError
                onOpenRequested: function(rootPath) {
                    root.recentWorkspaceOpenRequested(rootPath);
                }
                onPinRequested: function(rootPath) {
                    root.recentWorkspacePinRequested(rootPath);
                }
                onRemoveRequested: function(rootPath) {
                    root.recentWorkspaceRemoveRequested(rootPath);
                }
                onClearRequested: root.recentWorkspacesClearRequested()
            }

            Rectangle {
            width: parent.width
            height: 220
            radius: Theme.radiusLarge
            color: Theme.background1
            border.color: Theme.borderSoft
            border.width: 1

            Column {
                anchors.fill: parent
                anchors.margins: Theme.spacingLarge
                spacing: Theme.spacingSmall

                Row {
                    width: parent.width
                    height: 28

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: qsTr("Estado do ambiente")
                        color: Theme.textPrimary
                        font.pixelSize: Theme.fontSizePanelTitle
                        font.bold: true
                    }

                    Item {
                        width: parent.width - x - detectButton.width
                        height: 1
                    }

                    KvButton {
                        id: detectButton

                        compact: true
                        enabled: !root.scanning
                        text: root.scanning ? qsTr("Detectando...") : qsTr("Redetectar")
                        iconName: "refresh"
                        onClicked: root.detectToolsRequested()
                    }
                }

                Text {
                    visible: root.tools.length === 0
                    text: qsTr("Detecte compiladores, CMake, Ninja, Git, depuradores e ferramentas Rust. A IDE apenas inspeciona; nunca instala sem confirmação.")
                    color: Theme.textSecondary
                    font.pixelSize: 12
                    width: parent.width
                    wrapMode: Text.WordWrap
                }

                Text {
                    visible: root.tools.length > 0
                    text: qsTr("%1 de %2 ferramentas detectadas")
                          .arg(root.detectedCount()).arg(root.tools.length)
                    color: Theme.textSecondary
                    font.pixelSize: 11
                }

                Grid {
                    id: environmentGrid

                    width: parent.width
                    columns: 2
                    columnSpacing: Theme.spacingMedium
                    rowSpacing: Theme.spacingXSmall

                    Repeater {
                        // Mantém o cartão estável em telas compactas; o painel
                        // Ferramentas continua sendo a lista completa.
                        model: root.tools.slice(0, 10)

                        delegate: Row {
                            id: toolRow

                            required property var modelData

                            width: (environmentGrid.width - environmentGrid.columnSpacing) / 2
                            height: 22
                            spacing: Theme.spacingSmall

                            Rectangle {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 7
                                height: 7
                                radius: 3.5
                                color: toolRow.modelData.status === "detected"
                                       || toolRow.modelData.status === "ready"
                                       ? Theme.successSoft : Theme.warningSoft
                            }

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 15
                                text: toolRow.modelData.displayName
                                      + (toolRow.modelData.version !== undefined
                                         ? "  " + toolRow.modelData.version : "")
                                color: Theme.textSecondary
                                font.pixelSize: 11
                                elide: Text.ElideRight
                            }
                        }
                    }
                }

                Text {
                    visible: root.tools.length > 10
                    text: qsTr("Mais %1 ferramentas no painel Ferramentas")
                          .arg(root.tools.length - 10)
                    color: Theme.textMuted
                    font.pixelSize: 10
                }
            }
            }
        }
    }
}
