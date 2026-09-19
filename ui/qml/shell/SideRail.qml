pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    // O trilho da Etapa 3 (E3-4, roadmaps/44 §4): Projeto · Git ·
    // Embarcados · Banco · Containers · Grafana · Ferramentas. Busca,
    // Build e Debug SAIRAM daqui (pedido do autor: "tem muito atalho
    // repetido; melhor deixar os do canto superior direito") — a busca no
    // projeto e' Ctrl+Shift+F e a aba de baixo; Build/Debug sao o widget
    // Executar do cabecalho, o menu e o painel de baixo.
    property bool workspaceOpen: false
    property bool explorerActive: false
    property bool gitActive: false
    property bool embeddedActive: false
    property bool toolsActive: false
    property bool databaseActive: false
    property bool containersActive: false
    property bool observabilityActive: false

    signal explorerToggled()
    signal gitRequested()
    signal embeddedRequested()
    signal toolsRequested()
    signal databaseRequested()
    signal containersRequested()
    signal observabilityRequested()
    // O modo EXPANDIDO (F1, fechamento da Etapa 2): o rotulo ao lado do
    // icone, como a referencia; o chevron do pe' alterna e a escolha e'
    // persistida pelo ShellController.
    property bool expanded: false
    signal expandedToggled()

    width: expanded ? 168 : 52
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    component RailButton: Rectangle {
        id: railButton

        property string iconName: "file"
        property string tooltip: ""
        // O rotulo do modo expandido; sem ele, o tooltip ate' o primeiro parentese.
        property string label: ""
        property bool active: false

        signal activated()

        width: root.expanded ? root.width - 2 * Theme.spacingSmall : 32
        height: 32
        radius: Theme.radius
        opacity: enabled ? 1.0 : 0.72
        color: active ? Theme.surfaceSelected
                      : (railButtonArea.containsMouse ? Theme.surface2 : "transparent")

        KvIcon {
            id: railIcon

            anchors.verticalCenter: parent.verticalCenter
            anchors.left: parent.left
            anchors.leftMargin: 5
            name: railButton.iconName
            size: 22
            active: railButton.active
            disabled: !railButton.enabled
            iconColor: railButton.active ? Theme.accent
                                         : (railButtonArea.containsMouse
                                            ? Theme.textPrimary
                                            : Theme.textSecondary)
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            anchors.left: railIcon.right
            anchors.leftMargin: Theme.spacingSmall
            anchors.right: parent.right
            visible: root.expanded
            text: railButton.label !== "" ? railButton.label : railButton.tooltip.split(" (")[0]
            color: railButton.active ? Theme.accent : Theme.textSecondary
            font.pixelSize: 12
            elide: Text.ElideRight
        }

        MouseArea {
            id: railButtonArea

            anchors.fill: parent
            enabled: railButton.enabled
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onContainsMouseChanged: {
                if (containsMouse && !root.expanded) {
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
            iconName: "git"
            tooltip: qsTr("Git — Commit e Log")
            label: qsTr("Git")
            active: root.gitActive
            enabled: root.workspaceOpen
            onActivated: root.gitRequested()
        }

        // Embarcados e' por projeto (o kit mora no .kinein), como o Git.
        RailButton {
            iconName: "embedded"
            tooltip: qsTr("Embarcados — placa, projeto, gravar, kit (Ctrl+Alt+M)")
            label: qsTr("Embarcados")
            active: root.embeddedActive
            enabled: root.workspaceOpen
            onActivated: root.embeddedRequested()
        }

        // Ferramentas NATIVAS com atalho visual (decisao do autor,
        // 2026-09-12; a ordem e o banco em 2026-09-13): banco de dados,
        // containers e observabilidade abrem daqui, sem projeto aberto — os
        // perfis, o motor e o Grafana sao da maquina, nao do workspace.
        // "Ferramentas" fecha a lista, por decisao do autor.
        RailButton {
            iconName: "database"
            tooltip: qsTr("Banco de dados (Ctrl+Alt+J)")
            label: qsTr("Banco")
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
            label: qsTr("Grafana")
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

    // O chevron do pe': compacto <-> expandido.
    Rectangle {
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Theme.spacingSmall
        anchors.horizontalCenter: parent.horizontalCenter
        width: root.expanded ? root.width - 2 * Theme.spacingSmall : 32
        height: 24
        radius: Theme.radius
        color: chevronArea.containsMouse ? Theme.surface2 : "transparent"

        Text {
            anchors.centerIn: parent
            text: root.expanded ? qsTr("‹ recolher") : "›"
            color: Theme.textMuted
            font.pixelSize: 12
        }

        MouseArea {
            id: chevronArea

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.expandedToggled()
        }
    }
}
