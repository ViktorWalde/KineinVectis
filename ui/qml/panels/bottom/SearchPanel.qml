pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

Item {
    id: panel

    ListModel {
        id: emptyResultsModel
    }

    property var resultsModel: emptyResultsModel
    property bool caseSensitive: false
    property bool searching: false
    property bool truncated: false
    property bool replaceMode: false
    property bool replacing: false
    property string replaceError: ""
    property string replaceSummary: ""

    signal searchRequested(string query)
    signal caseSensitivityToggleRequested(string query)
    signal resultOpenRequested(string path, int line, int column)
    signal replaceRequested(string query, string replacement)

    function clearInput() {
        searchInput.text = "";
    }

    function focusInput() {
        searchInput.forceActiveFocus();
        searchInput.selectAll();
    }

    function focusReplaceInput() {
        replaceInput.forceActiveFocus();
        replaceInput.selectAll();
    }

    Row {
        id: searchControls

        width: parent.width
        spacing: Theme.spacingSmall

        Rectangle {
            width: parent.width - caseChip.width
                   - searchStatus.width - 2 * Theme.spacingSmall
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: searchInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: searchInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeTerminal
                clip: true
                selectByMouse: true
                onAccepted: panel.searchRequested(text)
                onTextChanged: replaceButton.replaceArmed = false

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: searchInput.text === ""
                    text: qsTr("Buscar texto no workspace (Enter)")
                    color: Theme.textMuted
                    font.pixelSize: 11
                }
            }
        }

        Rectangle {
            id: caseChip

            width: caseChipLabel.width + 2 * Theme.spacingSmall
            height: 30
            radius: Theme.radius
            color: panel.caseSensitive ? Theme.surfaceSelected : "transparent"
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: caseChipLabel

                anchors.centerIn: parent
                text: qsTr("Aa")
                color: panel.caseSensitive ? Theme.accent : Theme.textSecondary
                font.pixelSize: 10
                font.bold: true
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.caseSensitivityToggleRequested(searchInput.text)
            }
        }

        Text {
            id: searchStatus

            anchors.verticalCenter: parent.verticalCenter
            text: {
                if (panel.searching) {
                    return qsTr("buscando...");
                }
                if (panel.resultsModel.count === 0) {
                    return "";
                }
                if (panel.truncated) {
                    return qsTr("%1+ resultados").arg(panel.resultsModel.count);
                }
                return qsTr("%1 resultados").arg(panel.resultsModel.count);
            }
            color: Theme.textMuted
            font.pixelSize: 10
        }
    }

    Row {
        id: replaceControls

        anchors.top: searchControls.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        height: visible ? 30 : 0
        spacing: Theme.spacingSmall
        visible: panel.replaceMode

        Rectangle {
            width: parent.width - replaceButton.width - Theme.spacingSmall
            height: 30
            radius: Theme.radius
            color: Theme.background0
            border.color: replaceInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: replaceInput

                anchors.fill: parent
                anchors.leftMargin: Theme.spacingSmall
                anchors.rightMargin: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: Theme.fontSizeTerminal
                clip: true
                selectByMouse: true
                onTextChanged: replaceButton.replaceArmed = false

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    visible: replaceInput.text === ""
                    text: qsTr("Substituir por (vazio remove)")
                    color: Theme.textMuted
                    font.pixelSize: 11
                }
            }
        }

        Rectangle {
            id: replaceButton

            property bool replaceArmed: false

            width: replaceLabel.width + 2 * Theme.spacingMedium
            height: 30
            radius: Theme.radius
            opacity: panel.replacing ? 0.55 : 1.0
            color: replaceArmed ? Theme.errorSoft
                                : (replaceArea.pressed ? Theme.accentDim : Theme.accent)

            Text {
                id: replaceLabel

                anchors.centerIn: parent
                text: panel.replacing ? qsTr("Substituindo...")
                      : (replaceButton.replaceArmed
                         ? qsTr("Confirmar") : qsTr("Substituir tudo"))
                color: Theme.background0
                font.pixelSize: 10
                font.bold: true
            }

            MouseArea {
                id: replaceArea

                anchors.fill: parent
                enabled: !panel.replacing
                cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: {
                    if (!replaceButton.replaceArmed) {
                        replaceButton.replaceArmed = true;
                        return;
                    }
                    replaceButton.replaceArmed = false;
                    panel.replaceRequested(searchInput.text, replaceInput.text);
                }
            }
        }
    }

    Text {
        id: replaceFeedback

        anchors.top: replaceControls.visible ? replaceControls.bottom : searchControls.bottom
        anchors.topMargin: visible ? Theme.spacingXSmall : 0
        anchors.left: parent.left
        anchors.right: parent.right
        visible: panel.replaceError !== "" || panel.replaceSummary !== ""
        text: panel.replaceError !== "" ? panel.replaceError : panel.replaceSummary
        color: panel.replaceError !== "" ? Theme.errorSoft : Theme.successSoft
        font.pixelSize: 10
        elide: Text.ElideRight
    }

    ListView {
        id: searchResultsView


        // B2 (docs/24): barra de rolagem. `parent: searchResultsView` é OBRIGATÓRIO — um filho
        // declarado dentro de um ListView vira filho do contentItem e ROLARIA
        // junto com a lista. O ListView segue sendo a fonte da verdade.
        VerticalScrollBar {
            id: scrollBar_searchResultsView

            parent: searchResultsView
            anchors.right: searchResultsView.right
            anchors.top: searchResultsView.top
            anchors.bottom: searchResultsView.bottom

            contentSize: searchResultsView.contentHeight
            viewportSize: searchResultsView.height
            position: searchResultsView.contentY

            onMoveRequested: function(position) {
                searchResultsView.contentY = position;
            }
        }
        anchors.top: replaceFeedback.visible ? replaceFeedback.bottom
                    : (replaceControls.visible ? replaceControls.bottom
                                               : searchControls.bottom)
        anchors.topMargin: Theme.spacingSmall
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        clip: true
        spacing: 2
        model: panel.resultsModel

        Text {
            anchors.centerIn: parent
            visible: panel.resultsModel.count === 0 && !panel.searching
            text: qsTr("Digite um termo e pressione Enter (Ctrl+Shift+F).")
            color: Theme.textMuted
            font.pixelSize: 11
        }

        delegate: Rectangle {
            id: searchResultDelegate

            required property string path
            required property int line
            required property int column
            required property string preview

            width: searchResultsView.width
            height: searchRow.height + Theme.spacingSmall
            radius: Theme.radius
            color: searchArea.containsMouse ? Theme.surface2 : "transparent"

            Row {
                id: searchRow

                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                spacing: Theme.spacingSmall
                width: parent.width - 2 * Theme.spacingSmall

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: searchResultDelegate.path + ":" + searchResultDelegate.line
                    color: Theme.accent
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - x
                    text: searchResultDelegate.preview
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: 11
                    elide: Text.ElideRight
                }
            }

            MouseArea {
                id: searchArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: panel.resultOpenRequested(searchResultDelegate.path,
                                                     searchResultDelegate.line,
                                                     searchResultDelegate.column)
            }
        }
    }
}
