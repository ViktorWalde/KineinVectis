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
        replaceControls.focusInput();
    }

    Row {
        id: searchControls

        width: parent.width
        spacing: Theme.spacingSmall

        KvTextField {
            id: searchInput

            width: parent.width - caseChip.width
                   - searchStatus.width - 2 * Theme.spacingSmall
            height: 30
            pixelSize: Theme.fontSizeTerminal
            iconName: "search"
            placeholder: qsTr("Buscar no projeto (Enter) — \\n quebra linha")
            onAccepted: panel.searchRequested(searchInput.text)
            onTextChanged: replaceControls.disarm()
        }

        KvToggleChip {
            id: caseChip

            anchors.verticalCenter: parent.verticalCenter
            height: 30
            labelText: "Aa"
            tooltip: qsTr("Diferenciar maiúsculas")
            active: panel.caseSensitive
            onToggled: panel.caseSensitivityToggleRequested(searchInput.text)
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
                return panel.resultsModel.count === 1 ? qsTr("1 resultado")
                                                      : qsTr("%1 resultados").arg(panel.resultsModel.count);
            }
            color: Theme.textMuted
            font.pixelSize: Theme.fontSizeCaption
        }
    }

    SearchReplaceBar {
        id: replaceControls

        anchors.top: searchControls.bottom
        anchors.topMargin: Theme.spacingSmall
        width: parent.width
        visible: panel.replaceMode
        busy: panel.replacing

        onReplaceRequested: replacement =>
            panel.replaceRequested(searchInput.text, replacement)
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
        font.pixelSize: Theme.fontSizeCaption
        elide: Text.ElideRight
    }

    ListView {
        id: searchResultsView

        FlickableScrollBar {
            id: scrollBar_searchResultsView

            view: searchResultsView
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
            font.pixelSize: Theme.fontSizeSmall
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
                    font.pixelSize: Theme.fontSizeSmall
                }

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    width: parent.width - x
                    text: searchResultDelegate.preview
                    color: Theme.textPrimary
                    font.family: Theme.monoFont
                    font.pixelSize: Theme.fontSizeSmall
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
