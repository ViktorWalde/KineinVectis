pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// D1b (docs/24): a barra de busca/substituição do arquivo aberto. Puro
// renderer + entrada — toda a lógica vive no EditorFindController.
Rectangle {
    id: root

    property bool replaceMode: false
    property string query: ""
    property string replacement: ""
    property bool caseSensitive: false
    property bool wholeWord: false
    property bool useRegex: false
    property bool invalidRegex: false
    property int matchCount: 0
    property int currentMatch: 0
    property real maxAvailableWidth: 460

    signal queryEdited(string text)
    signal replacementEdited(string text)
    signal findNextRequested()
    signal findPreviousRequested()
    signal replaceRequested()
    signal replaceAllRequested()
    signal caseToggleRequested()
    signal wholeWordToggleRequested()
    signal regexToggleRequested()
    signal closeRequested()

    width: Math.min(460, maxAvailableWidth)
    height: barColumn.height + 2 * Theme.spacingSmall
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.borderStrong
    border.width: 1

    function focusQuery() {
        queryInput.forceActiveFocus();
        queryInput.selectAll();
    }

    // "3 de 17" · "Nenhum resultado" · "Regex invalida" — o estado da busca
    // numa frase só, do jeito que o VS Code faz.
    readonly property string statusText: {
        if (invalidRegex) {
            return qsTr("Regex invalida");
        }
        if (query === "") {
            return "";
        }
        if (matchCount === 0) {
            return qsTr("Nenhum resultado");
        }
        return qsTr("%1 de %2").arg(currentMatch).arg(matchCount);
    }

    component ToggleButton: Rectangle {
        id: toggle

        property string labelText: ""
        property bool active: false
        property string tooltip: ""

        signal toggled()

        width: 22
        height: 22
        radius: Theme.radiusXSmall
        color: active ? Theme.surfaceSelected : "transparent"
        border.color: active ? Theme.accent : "transparent"
        border.width: 1

        Text {
            anchors.centerIn: parent
            text: toggle.labelText
            color: toggle.active ? Theme.accent : Theme.textMuted
            font.family: Theme.monoFont
            font.pixelSize: 11
            font.bold: toggle.active
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            hoverEnabled: true
            onClicked: toggle.toggled()
        }
    }

    component ActionButton: Rectangle {
        id: action

        property string labelText: ""
        property string iconName: ""
        property bool enabledAction: true

        signal activated()

        width: iconName !== "" ? 22
                               : Math.max(22, actionLabel.implicitWidth
                                              + 2 * Theme.spacingSmall)
        height: 22
        radius: Theme.radiusXSmall
        color: actionMouse.containsMouse && action.enabledAction
               ? Theme.surfaceSelected : "transparent"
        border.color: Theme.borderSoft
        border.width: 1
        opacity: action.enabledAction ? 1.0 : 0.4

        Text {
            id: actionLabel

            anchors.centerIn: parent
            visible: action.iconName === ""
            text: action.labelText
            color: Theme.textSecondary
            font.pixelSize: 11
        }

        KvIcon {
            anchors.centerIn: parent
            visible: action.iconName !== ""
            name: action.iconName
            size: 14
            disabled: !action.enabledAction
        }

        MouseArea {
            id: actionMouse

            anchors.fill: parent
            hoverEnabled: true
            cursorShape: action.enabledAction ? Qt.PointingHandCursor
                                              : Qt.ArrowCursor
            onClicked: {
                if (action.enabledAction) {
                    action.activated();
                }
            }
        }
    }

    Column {
        id: barColumn

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            width: parent.width
            spacing: Theme.spacingSmall

            Rectangle {
                width: parent.width - findControls.width - Theme.spacingSmall
                height: 26
                radius: Theme.radius
                color: Theme.background0
                border.width: 1
                border.color: root.invalidRegex
                              ? Theme.errorSoft
                              : (queryInput.activeFocus ? Theme.accent
                                                        : Theme.borderSoft)

                TextInput {
                    id: queryInput

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    text: root.query
                    color: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    selectedTextColor: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.queryEdited(text)
                    Keys.onEscapePressed: root.closeRequested()
                    Keys.onReturnPressed: function(event) {
                        if (event.modifiers & Qt.ShiftModifier) {
                            root.findPreviousRequested();
                        } else {
                            root.findNextRequested();
                        }
                        event.accepted = true;
                    }
                    Keys.onEnterPressed: function(event) {
                        if (event.modifiers & Qt.ShiftModifier) {
                            root.findPreviousRequested();
                        } else {
                            root.findNextRequested();
                        }
                        event.accepted = true;
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: queryInput.text === ""
                        text: qsTr("Localizar")
                        color: Theme.textDisabled
                        font.family: Theme.monoFont
                        font.pixelSize: 12
                    }
                }
            }

            Row {
                id: findControls

                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXSmall

                ToggleButton {
                    labelText: "Aa"
                    active: root.caseSensitive
                    onToggled: root.caseToggleRequested()
                }

                ToggleButton {
                    labelText: "W"
                    active: root.wholeWord
                    onToggled: root.wholeWordToggleRequested()
                }

                ToggleButton {
                    labelText: ".*"
                    active: root.useRegex
                    onToggled: root.regexToggleRequested()
                }

                ActionButton {
                    iconName: "chevron-up"
                    enabledAction: root.matchCount > 0
                    onActivated: root.findPreviousRequested()
                }

                ActionButton {
                    iconName: "chevron-down"
                    enabledAction: root.matchCount > 0
                    onActivated: root.findNextRequested()
                }

                ActionButton {
                    iconName: "close"
                    onActivated: root.closeRequested()
                }
            }
        }

        Row {
            width: parent.width
            visible: root.replaceMode
            spacing: Theme.spacingSmall

            Rectangle {
                width: parent.width - replaceControls.width - Theme.spacingSmall
                height: 26
                radius: Theme.radius
                color: Theme.background0
                border.width: 1
                border.color: replaceInput.activeFocus ? Theme.accent
                                                       : Theme.borderSoft

                TextInput {
                    id: replaceInput

                    anchors.fill: parent
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    verticalAlignment: TextInput.AlignVCenter
                    text: root.replacement
                    color: Theme.textPrimary
                    selectionColor: Theme.accentDim
                    selectedTextColor: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 12
                    clip: true
                    selectByMouse: true
                    onTextEdited: root.replacementEdited(text)
                    Keys.onEscapePressed: root.closeRequested()
                    onAccepted: root.replaceRequested()

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: replaceInput.text === ""
                        text: qsTr("Substituir por")
                        color: Theme.textDisabled
                        font.family: Theme.monoFont
                        font.pixelSize: 12
                    }
                }
            }

            Row {
                id: replaceControls

                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXSmall

                ActionButton {
                    labelText: qsTr("Substituir")
                    enabledAction: root.matchCount > 0
                    onActivated: root.replaceRequested()
                }

                ActionButton {
                    labelText: qsTr("Tudo")
                    enabledAction: root.matchCount > 0
                    onActivated: root.replaceAllRequested()
                }
            }
        }

        Text {
            width: parent.width
            visible: root.statusText !== ""
            text: root.statusText
            color: root.invalidRegex ? Theme.errorSoft : Theme.textMuted
            font.pixelSize: 10
        }
    }
}
