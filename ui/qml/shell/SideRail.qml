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
    // As entradas sao DADO (V3, 2026-09-24): antes cada uma custava uma
    // propriedade `xActive`, um sinal `xRequested` e um bloco de botao aqui,
    // mais o binding e o handler do outro lado. Quem declara e' o
    // `ToolWindows`; este arquivo so' desenha.
    property var entries: []

    signal activated(string id)
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

        Repeater {
            model: root.entries

            delegate: RailButton {
                required property var modelData

                iconName: modelData.icon
                tooltip: modelData.tooltip
                label: modelData.label
                active: modelData.active
                enabled: modelData.available
                onActivated: root.activated(modelData.id)
            }
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
