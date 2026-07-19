import QtQuick

Rectangle {
    id: root

    property bool workspaceOpen: false
    property bool explorerActive: false
    // Estado CRU do painel inferior; o rail mapeia estado -> chip aceso
    // sozinho (visao burra: quem decide o que abre continua sendo o host).
    property bool bottomOpen: false
    property string bottomTab: ""

    signal explorerToggled()
    signal searchRequested()
    signal gitRequested()
    signal buildRequested()
    signal debugRequested()
    signal terminalRequested()
    signal toolsRequested()

    function bottomActive(tab) {
        return bottomOpen && bottomTab === tab;
    }

    // §4.2: o rail NAO e um cartao — ele e a borda da janela. Mesmo fundo do
    // app (background0), sem contorno; os icones ficam "no plano de fundo" e
    // so o chip de hover/ativo e desenhado (arredondamento para dentro).
    width: 52
    color: Theme.background0

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
            active: root.bottomActive("search")
            enabled: root.workspaceOpen
            onActivated: root.searchRequested()
        }

        RailButton {
            iconName: "git"
            tooltip: qsTr("Git")
            active: root.bottomActive("git")
            enabled: root.workspaceOpen
            onActivated: root.gitRequested()
        }

        RailButton {
            iconName: "build"
            tooltip: qsTr("Build e jobs")
            active: root.bottomActive("build") || root.bottomActive("jobs")
            enabled: root.workspaceOpen
            onActivated: root.buildRequested()
        }

        RailButton {
            iconName: "debug"
            tooltip: qsTr("Debug")
            active: root.bottomActive("debug")
            enabled: root.workspaceOpen
            onActivated: root.debugRequested()
        }

        RailButton {
            iconName: "terminal"
            tooltip: qsTr("Terminal")
            active: root.bottomActive("terminal")
            enabled: root.workspaceOpen
            onActivated: root.terminalRequested()
        }

        RailButton {
            iconName: "tools"
            tooltip: qsTr("Ferramentas")
            active: root.bottomActive("tools")
            onActivated: root.toolsRequested()
        }

    }
}
