pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// D1b (docs/roadmaps/24): a barra de busca/substituição do arquivo aberto. Puro
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

                KvToggleChip {
                    labelText: "Aa"
                    active: root.caseSensitive
                    onToggled: root.caseToggleRequested()
                }

                KvToggleChip {
                    labelText: "W"
                    active: root.wholeWord
                    onToggled: root.wholeWordToggleRequested()
                }

                KvToggleChip {
                    labelText: ".*"
                    active: root.useRegex
                    onToggled: root.regexToggleRequested()
                }

                KvBarButton {
                    iconName: "chevron-up"
                    enabledAction: root.matchCount > 0
                    onActivated: root.findPreviousRequested()
                }

                KvBarButton {
                    iconName: "chevron-down"
                    enabledAction: root.matchCount > 0
                    onActivated: root.findNextRequested()
                }

                KvBarButton {
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

                KvBarButton {
                    labelText: qsTr("Substituir")
                    enabledAction: root.matchCount > 0
                    onActivated: root.replaceRequested()
                }

                KvBarButton {
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
