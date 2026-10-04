import QtQuick

Rectangle {
    id: root

    property string iconName: "file"
    property string tooltip: ""
    property string accessibleName: tooltip
    property bool active: false
    property bool primary: false
    property bool danger: false
    // Verde de "liga/inicia" (o par do `danger` vermelho de "para/remove").
    property bool success: false
    property bool compact: false
    property int iconSize: 20
    property real iconRotation: 0
    // Botao de barra que abre popover: o clique do mouse nao deixa um anel de
    // foco preso nele (o teclado continua focando pelo Tab).
    property bool focusOnClick: true
    property string tooltipPlacement: "bottom"

    signal clicked()

    // Desligado, nem o primario veste o acento nem o perigoso veste o
    // vermelho: o icone fica APAGADO (Theme.textDisabled). Antes, `iconColor`
    // sobrescrevia o `disabled` do KvIcon e um botao desligado so' perdia 28%
    // de opacidade — na fileira de acoes de um container, "Logs" sem projeto
    // parecia tao clicavel quanto "Iniciar" (medido em 2026-09-13).
    readonly property bool accented: primary && enabled

    implicitWidth: compact ? 24 : 32
    implicitHeight: compact ? 24 : 32
    radius: Theme.radius
    // Pairar acende, apertar escurece e encolhe um pouco, com transicao curta
    // (2026-10-03; a linguagem do KvToggleChip e do KvButton).
    color: accented ? (buttonArea.pressed ? Theme.accentDim : Theme.accent)
                    : (buttonArea.pressed || active) ? Theme.surfaceSelected
                   : (buttonArea.containsMouse || activeFocus
                      ? Theme.surface2 : "transparent")
    border.color: activeFocus ? Theme.accent : "transparent"
    border.width: 1
    opacity: enabled ? 1.0 : 0.72
    scale: buttonArea.pressed ? 0.94 : 1

    Behavior on color {
        ColorAnimation { duration: Theme.motionFast }
    }

    Behavior on scale {
        NumberAnimation { duration: Theme.motionFast }
    }
    focus: true
    Accessible.role: Accessible.Button
    Accessible.name: accessibleName
    Keys.onSpacePressed: root.clicked()
    Keys.onReturnPressed: root.clicked()

    KvIcon {
        anchors.centerIn: parent
        name: root.iconName
        rotation: root.iconRotation
        size: root.compact ? Math.min(root.iconSize, 16) : root.iconSize
        active: root.active
        disabled: !root.enabled
        error: root.danger
        iconColor: !root.enabled ? Theme.textDisabled
                                 : root.primary ? Theme.background0
                                 : (root.danger ? Theme.errorSoft
                                    : root.success ? Theme.successSoft
                                    : (root.active ? Theme.accent
                                       : (buttonArea.containsMouse
                                          ? Theme.iconHover
                                          : Theme.iconDefault)))
    }

    MouseArea {
        id: buttonArea

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onContainsMouseChanged: {
            if (containsMouse) {
                TooltipController.showFor(root, root.tooltip,
                                          root.tooltipPlacement);
            } else {
                TooltipController.hideFor(root);
            }
        }
        onClicked: {
            if (root.focusOnClick) root.forceActiveFocus();
            TooltipController.hideFor(root);
            root.clicked();
        }
    }
}
