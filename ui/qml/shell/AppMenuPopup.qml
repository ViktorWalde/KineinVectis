pragma ComponentBehavior: Bound
import QtQuick

FocusScope {
    id: root

    property real menuX: 0
    property real menuY: 0
    property real menuWidth: 220
    property var items: []
    property int currentIndex: -1
    property Item previousFocusItem: null

    signal dismissRequested(bool restoreFocus)
    signal actionRequested(string action)

    onVisibleChanged: if (visible) Qt.callLater(prepare)
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

    Keys.onShortcutOverride: function(event) { event.accepted = true; }
    Keys.onPressed: function(event) { root.handleKey(event); }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        onClicked: root.dismissRequested(true)
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

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.AllButtons
        }

        ListView {
            id: menuList

            anchors.fill: parent
            anchors.margins: Theme.spacingSmall
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: root.items
            currentIndex: root.currentIndex

            delegate: Rectangle {
                id: menuItem

                required property var modelData
                required property int index

                width: menuList.width
                height: 28
                radius: Theme.radius
                color: (itemArea.containsMouse || index === root.currentIndex) && modelData.enabled
                       ? Theme.surfaceSelected : "transparent"
                opacity: modelData.enabled ? 1.0 : 0.72
                Accessible.role: Accessible.MenuItem
                Accessible.name: modelData.label
                Accessible.onPressAction: root.activate(menuItem.index)

                Text {
                    id: itemLabel

                    anchors.left: parent.left
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.right: itemShortcut.left
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    text: menuItem.modelData.label
                    color: menuItem.modelData.enabled
                           ? Theme.textPrimary : Theme.textDisabled
                    font.pixelSize: 12
                    elide: Text.ElideRight
                }

                Text {
                    id: itemShortcut

                    anchors.right: parent.right
                    anchors.rightMargin: Theme.spacingSmall
                    anchors.verticalCenter: parent.verticalCenter
                    text: menuItem.modelData.shortcut !== undefined
                          ? menuItem.modelData.shortcut : ""
                    color: Theme.textMuted
                    font.pixelSize: 11
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
