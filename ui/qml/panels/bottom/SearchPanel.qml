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

    signal searchRequested(string query)
    signal caseSensitivityToggleRequested(string query)
    signal resultOpenRequested(string path, int line, int column)

    function clearInput() {
        searchInput.text = "";
    }

    function focusInput() {
        searchInput.forceActiveFocus();
        searchInput.selectAll();
    }

    Row {
        id: searchControls

        width: parent.width
        spacing: Theme.spacingSmall

        Rectangle {
            width: parent.width - caseChip.width
                   - searchStatus.width - 2 * Theme.spacingSmall
            height: 24
            radius: Theme.radius
            color: Theme.background0
            border.color: searchInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: searchInput

                anchors.fill: parent
                anchors.margins: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 11
                clip: true
                selectByMouse: true
                onAccepted: panel.searchRequested(text)

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
            height: 24
            radius: Theme.radius
            color: panel.caseSensitive ? Theme.accentDim : "transparent"
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

    ListView {
        id: searchResultsView

        anchors.top: searchControls.bottom
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
