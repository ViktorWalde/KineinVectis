import QtQuick

Rectangle {
    id: root

    property string text: ""
    property string iconName: ""
    property bool primary: false
    property bool selected: false
    property bool compact: false
    // Acao destrutiva (remover): o texto e a borda em vermelho brando.
    property bool danger: false
    // O tooltip aparece tambem com o botao DESLIGADO — e' ai' que ele diz
    // o porque (E3-6: "abra um projeto — a aba de terminal e' do projeto").
    property string tooltip: ""

    signal clicked()

    // Um botao primario DESLIGADO nao veste o acento: com 72% de opacidade o
    // ambar continuava sendo a coisa mais chamativa do painel, e o clique que
    // nao fazia nada parecia defeito da funcao ("compose up" sem projeto,
    // medido em 2026-09-13). Desligado, ele e' um botao comum e apagado.
    readonly property bool accented: primary && enabled

    implicitWidth: buttonContent.implicitWidth + 2 * Theme.spacingMedium
    implicitHeight: compact ? 28 : 32
    radius: Theme.radius
    color: accented ? (buttonArea.pressed ? Theme.accentDim : Theme.accent)
                    : selected ? Theme.surfaceSelected
                    : (buttonArea.containsMouse || activeFocus
                       ? Theme.surface2 : Theme.surface1)
    border.color: activeFocus || selected ? Theme.accent : (danger && enabled ? Theme.errorSoft : Theme.borderSoft)
    border.width: accented ? 0 : 1
    opacity: enabled ? 1.0 : 0.72
    focus: true
    Accessible.role: Accessible.Button
    Accessible.name: text
    Keys.onSpacePressed: root.clicked()
    Keys.onReturnPressed: root.clicked()

    Row {
        id: buttonContent

        anchors.centerIn: parent
        spacing: Theme.spacingSmall

        KvIcon {
            anchors.verticalCenter: parent.verticalCenter
            visible: root.iconName !== ""
            name: root.iconName
            size: 16
            active: root.selected
            disabled: !root.enabled
            iconColor: root.accented ? Theme.background0
                                     : root.selected ? Theme.accent
                                     : (root.enabled ? Theme.textSecondary
                                        : Theme.textDisabled)
        }

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            color: root.accented ? Theme.background0
                                 : (root.enabled ? (root.danger ? Theme.errorSoft : Theme.textPrimary)
                                    : Theme.textDisabled)
            font.pixelSize: 12
            font.bold: root.primary || root.selected
        }
    }

    MouseArea {
        id: buttonArea

        anchors.fill: parent
        enabled: root.enabled
        hoverEnabled: true
        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
        onClicked: {
            root.forceActiveFocus();
            root.clicked();
        }
    }

    // O pairar do tooltip e' separado do clique: continua vivo desligado.
    MouseArea {
        anchors.fill: parent
        hoverEnabled: root.tooltip !== ""
        acceptedButtons: Qt.NoButton
        onContainsMouseChanged: {
            if (containsMouse && root.tooltip !== "") {
                TooltipController.showFor(root, root.tooltip, "bottom");
            } else {
                TooltipController.hideFor(root);
            }
        }
    }
}
