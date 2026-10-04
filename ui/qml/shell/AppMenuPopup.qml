pragma ComponentBehavior: Bound
import QtQuick

FocusScope {
    id: root

    // A altura da faixa de cima que recebe o clique depois de fechar o menu.
    property real passThroughTop: 0
    property real menuX: 0
    property real menuY: 0
    property real menuWidth: 280
    property var items: []
    property int currentIndex: -1
    property Item previousFocusItem: null

    signal dismissRequested(bool restoreFocus)
    signal actionRequested(string action)

    // Fechado, o menu NUNCA fica com o foco (2026-10-04, achado na tela:
    // aberto quando nada tinha foco — clique na area vazia do editor —, ao
    // fechar ninguem recebia o foco de volta; o menu invisivel continuava
    // com ele e aceitando todo ShortcutOverride, e Ctrl+O/Ctrl+Alt+S morriam).
    onVisibleChanged: {
        if (visible) {
            Qt.callLater(prepare);
        } else if (activeFocus) {
            restorePreviousFocus();
            if (activeFocus) focus = false;
        }
    }
    Component.onCompleted: if (visible) Qt.callLater(prepare)
    onActiveFocusChanged: if (visible && !activeFocus) dismissRequested(false)

    function prepare() {
        if (!visible) return;
        if (!activeFocus && root.Window.window)
            previousFocusItem = root.Window.window.activeFocusItem;
        currentIndex = -1;
        moveCurrent(1);
        forceActiveFocus();
    }

    // O host restaura antes de executar a acao, que pode abrir outro dialogo.
    // Perda de foco para outra superficie fecha sem chamar esta funcao.
    function restorePreviousFocus() {
        if (previousFocusItem && previousFocusItem.visible && previousFocusItem.enabled)
            previousFocusItem.forceActiveFocus();
    }

    function moveCurrent(direction) {
        for (let step = 1; step <= items.length; step++) {
            const next = (currentIndex + direction * step + items.length) % items.length;
            if (items[next].enabled) {
                currentIndex = next;
                menuList.positionViewAtIndex(next, ListView.Contain);
                return;
            }
        }
        currentIndex = -1;
    }

    function activate(index) {
        if (index >= 0 && index < items.length && items[index].enabled)
            actionRequested(items[index].action);
    }

    function handleKey(event) {
        event.accepted = true; // Um menu aberto nunca envia teclas ao PTY atras dele.
        if (event.key === Qt.Key_Escape) dismissRequested(true);
        else if (event.key === Qt.Key_Down || event.key === Qt.Key_Tab) moveCurrent(1);
        else if (event.key === Qt.Key_Up || event.key === Qt.Key_Backtab) moveCurrent(-1);
        else if (event.key === Qt.Key_Home) { currentIndex = -1; moveCurrent(1); }
        else if (event.key === Qt.Key_End) { currentIndex = 0; moveCurrent(-1); }
        else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter
                 || event.key === Qt.Key_Space) activate(currentIndex);
    }

    Keys.onShortcutOverride: function(event) { event.accepted = root.visible; }
    Keys.onPressed: function(event) { root.handleKey(event); }

    // Clicar fora fecha o menu. Na faixa de cima (a barra do ☰, 0.3.9) o
    // clique fecha E segue para a barra: o ☰ recolhe e outro titulo abre o
    // menu dele num clique so', como numa barra de menus.
    KvBackdrop {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onPressed: function(mouse) {
            root.dismissRequested(true);
            if (mouse.y < root.passThroughTop) mouse.accepted = false;
        }
    }

    Rectangle {
        id: menuFrame
        x: Math.max(Theme.spacingSmall,
                    Math.min(root.menuX,
                             root.width - width - Theme.spacingSmall))
        y: Math.max(Theme.spacingSmall,
                    Math.min(root.menuY,
                             root.height - height - Theme.spacingSmall))
        width: Math.max(0, Math.min(root.menuWidth, root.width - 2 * Theme.spacingSmall))
        height: Math.max(0, Math.min(menuList.contentHeight + 2 * Theme.spacingSmall,
                                   root.height - 2 * Theme.spacingSmall))
        radius: Theme.radiusLarge
        color: Theme.background2
        border.color: Theme.borderStrong
        border.width: 1

        // A moldura segura clique, roda e pairar: nada atravessa o menu
        // (nem nas frestas entre os itens) para o que esta' por baixo.
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
            hoverEnabled: true
            onWheel: wheel => wheel.accepted = true
        }

        ListView {
            id: menuList

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.items
            currentIndex: root.currentIndex

            // Um item: icone, rotulo e o atalho a direita; o separador e' uma
            // linha fina entre os grupos (2026-10-03: os menus eram listas
            // planas numa cor so'). Ao pairar, a superficie acende com
            // transicao e a barra ambar marca o item.
            delegate: Rectangle {
                id: menuItem

                required property var modelData
                required property int index

                readonly property bool separator: menuItem.modelData.separator === true
                readonly property bool current: index === root.currentIndex && menuItem.modelData.enabled

                width: menuList.width
                height: menuItem.separator ? 9 : 30
                radius: Theme.radius
                color: menuItem.current ? Theme.surfaceSelected : "transparent"
                Accessible.role: menuItem.separator ? Accessible.Separator : Accessible.MenuItem
                Accessible.name: menuItem.modelData.label
                Accessible.onPressAction: root.activate(menuItem.index)

                Behavior on color {
                    ColorAnimation { duration: Theme.motionFast }
                }

                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    visible: menuItem.separator
                    height: 1
                    color: Theme.borderSoft
                }

                Rectangle {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !menuItem.separator
                    width: 3
                    height: menuItem.current ? parent.height - 12 : 0
                    radius: width / 2
                    color: Theme.accent

                    Behavior on height {
                        NumberAnimation { duration: Theme.motionFast }
                    }
                }

                KvIcon {
                    id: itemIcon

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall + 4
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !menuItem.separator
                    width: 16
                    size: 16
                    name: menuItem.modelData.icon !== undefined && menuItem.modelData.icon !== ""
                          ? menuItem.modelData.icon : "fill"
                    opacity: menuItem.modelData.icon !== undefined && menuItem.modelData.icon !== "" ? 1 : 0
                    active: menuItem.current
                    disabled: !menuItem.modelData.enabled
                }

                Text {
                    id: itemLabel

                    anchors.left: itemIcon.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.right: itemShortcut.left
                    anchors.rightMargin: Theme.spacingMedium
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !menuItem.separator
                    text: menuItem.modelData.label
                    color: menuItem.modelData.enabled ? Theme.textPrimary : Theme.textDisabled
                    font.pixelSize: Theme.fontSizeBody
                    elide: Text.ElideRight
                }

                Text {
                    id: itemShortcut

                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    visible: !menuItem.separator
                    text: menuItem.modelData.shortcut !== undefined ? menuItem.modelData.shortcut : ""
                    color: menuItem.current ? Theme.textSecondary : Theme.textMuted
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeCaption
                }

                MouseArea {
                    id: itemArea

                    anchors.fill: parent
                    enabled: menuItem.modelData.enabled
                    hoverEnabled: true
                    cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                    onEntered: root.currentIndex = menuItem.index
                    onClicked: root.activate(menuItem.index)
                }
            }
        }
    }
}
