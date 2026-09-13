import QtQuick

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool explorerActive: false
    property bool searchActive: false
    property bool gitActive: false
    property bool buildActive: false
    property bool debugActive: false
    property bool toolsActive: false
    property bool databaseActive: false
    property bool containersActive: false
    property bool observabilityActive: false

    signal explorerToggled()
    signal searchRequested()
    signal gitRequested()
    signal buildRequested()
    signal debugRequested()
    signal toolsRequested()
    signal databaseRequested()
    signal containersRequested()
    signal observabilityRequested()

    width: 52
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    component RailButton: Rectangle {
        id: railButton

        property string iconName: "file"
        property string tooltip: ""
        property bool active: false

        signal activated()

        width: 32
        height: 32
        radius: Theme.radius
        opacity: enabled ? 1.0 : 0.72
        color: active ? Theme.surfaceSelected
                      : (railButtonArea.containsMouse ? Theme.surface2 : "transparent")

        KvIcon {
            anchors.centerIn: parent
            name: railButton.iconName
            size: 22
            active: railButton.active
            disabled: !railButton.enabled
            iconColor: railButton.active ? Theme.accent
                                         : (railButtonArea.containsMouse
                                            ? Theme.textPrimary
                                            : Theme.textSecondary)
        }

        MouseArea {
            id: railButtonArea

            anchors.fill: parent
            enabled: railButton.enabled
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onContainsMouseChanged: {
                if (containsMouse) {
                    TooltipController.showFor(railButton, railButton.tooltip,
                                              "right");
                } else {
                    TooltipController.hideFor(railButton);
                }
            }
            onClicked: {
                TooltipController.hideFor(railButton);
                railButton.activated();
            }
        }
    }

    Column {
        anchors.top: parent.top
        anchors.topMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: Theme.spacingSmall

        RailButton {
            iconName: "project"
            tooltip: qsTr("Projeto")
            active: root.explorerActive && root.workspaceOpen
            enabled: root.workspaceOpen
            onActivated: root.explorerToggled()
        }

        RailButton {
            iconName: "search"
            tooltip: qsTr("Busca no projeto")
            active: root.searchActive
            enabled: root.workspaceOpen
            onActivated: root.searchRequested()
        }

        RailButton {
            iconName: "git"
            tooltip: qsTr("Git")
            active: root.gitActive
            enabled: root.workspaceOpen
            onActivated: root.gitRequested()
        }

        RailButton {
            iconName: "build"
            tooltip: qsTr("Build e jobs")
            active: root.buildActive
            enabled: root.workspaceOpen
            onActivated: root.buildRequested()
        }

        RailButton {
            iconName: "debug"
            tooltip: qsTr("Debug")
            active: root.debugActive
            enabled: root.workspaceOpen
            onActivated: root.debugRequested()
        }

        // Ferramentas NATIVAS com atalho visual (decisao do autor,
        // 2026-09-12; a ordem e o banco em 2026-09-13): banco de dados,
        // containers e observabilidade abrem daqui, sem projeto aberto — os
        // perfis, o motor e o Grafana sao da maquina, nao do workspace.
        // "Ferramentas" fecha a lista, por decisao do autor.
        RailButton {
            iconName: "database"
            tooltip: qsTr("Banco de dados (Ctrl+Alt+J)")
            active: root.databaseActive
            onActivated: root.databaseRequested()
        }

        RailButton {
            iconName: "container"
            tooltip: qsTr("Containers (Ctrl+Alt+W)")
            active: root.containersActive
            onActivated: root.containersRequested()
        }

        RailButton {
            iconName: "observability"
            tooltip: qsTr("Observabilidade — Grafana (Ctrl+Alt+O)")
            active: root.observabilityActive
            onActivated: root.observabilityRequested()
        }

        RailButton {
            iconName: "tools"
            tooltip: qsTr("Ferramentas")
            active: root.toolsActive
            onActivated: root.toolsRequested()
        }
    }
}
