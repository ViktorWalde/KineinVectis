pragma ComponentBehavior: Bound
import QtQuick

// O consumidor guarda o valor; abrir, cancelar e atualizar não o alteram.
FocusScope {
    id: root
    property var choices: [] // strings ou { value, label }
    property string value: ""
    property string emptyLabel: qsTr("Sem seleção")
    property string customPlaceholder: ""
    property string accessibleLabel: ""
    property bool allowCustom: true
    property bool busy: false
    property bool customRequested: false
    readonly property int selectedIndex: normalizedChoices.findIndex(choice => choice.value === root.value)
    readonly property bool customMode: allowCustom && (customRequested || (value !== "" && selectedIndex < 0))
    readonly property var normalizedChoices: choices.map(choice => typeof choice === "string"
        ? { value: choice, label: choice } : choice).filter(choice => choice.value !== "")
    readonly property var menuItems: [{ label: root.emptyLabel, action: "empty", enabled: true }]
        .concat(normalizedChoices.map((choice, index) => ({
            label: choice.label,
            action: String(index),
            enabled: true,
            icon: choice.value === root.value ? "check" : ""
        })))
        .concat(allowCustom ? [{ label: qsTr("Personalizado…"), action: "custom", enabled: true }] : [])
    readonly property string displayLabel: customMode ? qsTr("Personalizado…")
        : selectedIndex >= 0 ? normalizedChoices[selectedIndex].label : emptyLabel
    readonly property bool menuOpen: menu.visible
    signal edited(string value)

    implicitWidth: 220
    implicitHeight: 30 + (customMode ? 36 : 0)
    activeFocusOnTab: true
    Accessible.role: Accessible.ComboBox
    Accessible.name: accessibleLabel || emptyLabel
    Accessible.description: value
    Accessible.onPressAction: root.open()
    onVisibleChanged: if (!visible) { close(false); customRequested = false; }
    onEnabledChanged: if (!enabled) close(false)
    onBusyChanged: if (busy) close(false)
    onChoicesChanged: close(false)

    function open() {
        if (!enabled || busy || !visible || !root.Window.window) return;
        root.forceActiveFocus();
        const point = root.mapToItem(menu.parent, 0, 30);
        menu.menuX = point.x;
        menu.menuY = point.y;
        menu.menuWidth = Math.max(280, root.width);
        menu.visible = true;
    }
    function close(restoreFocus) {
        if (!menu.visible) return;
        menu.visible = false;
        if (restoreFocus && root.visible && root.enabled) root.forceActiveFocus();
    }
    function choose(action) {
        close(true);
        if (action === "custom") {
            customRequested = true;
            customInput.forceActiveFocus();
            customInput.selectAll();
        } else if (action === "empty") {
            customRequested = false;
            edited("");
        } else {
            const index = Number(action);
            if (!Number.isInteger(index) || index < 0 || index >= normalizedChoices.length) return;
            customRequested = false;
            edited(normalizedChoices[index].value);
        }
    }
    Keys.onPressed: event => {
        if (!customInput.activeFocus && (event.key === Qt.Key_Down || event.key === Qt.Key_Up
                || event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space)) {
            root.open();
            event.accepted = true;
        }
    }
    Rectangle {
        width: root.width
        height: 30
        radius: Theme.radius
        color: area.containsMouse ? Theme.surface2 : Theme.background0
        border.width: 1
        border.color: root.activeFocus || root.menuOpen ? Theme.accent : Theme.borderStrong
        opacity: root.enabled && !root.busy ? 1 : 0.72
        Text {
            anchors.left: parent.left
            anchors.leftMargin: Theme.spacingMedium
            anchors.right: chevron.left
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            text: root.busy ? qsTr("Buscando…") : root.displayLabel
            textFormat: Text.PlainText
            color: root.value === "" ? Theme.textSecondary : Theme.textPrimary
            font.pixelSize: Theme.fontSizeBody
            elide: Text.ElideMiddle
        }
        KvIcon {
            id: chevron
            anchors.right: parent.right
            anchors.rightMargin: Theme.spacingSmall
            anchors.verticalCenter: parent.verticalCenter
            size: 14
            name: root.menuOpen ? "chevron-up" : "chevron-down"
            active: root.activeFocus || root.menuOpen
        }
        MouseArea {
            id: area
            anchors.fill: parent
            hoverEnabled: true
            enabled: !root.busy
            cursorShape: Qt.PointingHandCursor
            onClicked: root.open()
        }
    }
    KvTextField {
        id: customInput
        y: 36
        width: root.width
        height: 30
        visible: root.customMode
        text: root.value
        placeholder: root.customPlaceholder
        onEdited: value => root.edited(value)
    }
    AppMenuPopup {
        id: menu
        // Fora do Flickable dos parâmetros, sem recorte pelo formulário.
        parent: root.Window.window ? root.Window.window.contentItem : root
        anchors.fill: parent
        z: 1000
        visible: false
        items: root.menuItems
        onActionRequested: action => root.choose(action)
        onDismissRequested: restoreFocus => root.close(restoreFocus)
    }
}
