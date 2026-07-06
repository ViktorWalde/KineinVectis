import QtQuick

Rectangle {
    id: root

    property var resultsModel
    property int resultCount: 0
    property int currentIndex: 0
    property bool loading: false
    property bool truncated: false
    property string errorText: ""
    property real maxAvailableWidth: 620
    property real maxAvailableHeight: 420

    signal queryChanged(string query)
    signal acceptRequested()
    signal dismissRequested()
    signal moveDownRequested()
    signal moveUpRequested()
    signal resultHovered(int index)
    signal resultActivated(int index)

    width: Math.min(620, maxAvailableWidth)
    height: Math.min(420, maxAvailableHeight)
    radius: Theme.radius
    color: Theme.background2
    border.color: Theme.accent
    border.width: 1

    function resetAndFocus() {
        searchInput.text = "";
        searchInput.forceActiveFocus();
    }

    function currentQuery() {
        return searchInput.text.trim();
    }

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Text {
            text: qsTr("Search Everywhere")
            color: Theme.textPrimary
            font.pixelSize: 13
            font.bold: true
        }

        Rectangle {
            width: parent.width
            height: 34
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
                font.pixelSize: 13
                clip: true
                selectByMouse: true
                onTextChanged: root.queryChanged(text)
                onAccepted: root.acceptRequested()
                Keys.onEscapePressed: root.dismissRequested()
                Keys.onDownPressed: root.moveDownRequested()
                Keys.onUpPressed: root.moveUpRequested()
            }
        }

        Text {
            width: parent.width
            visible: root.errorText !== ""
            text: root.errorText
            color: Theme.errorSoft
            font.pixelSize: 10
            wrapMode: Text.WordWrap
        }

        Text {
            width: parent.width
            text: {
                if (root.loading) {
                    return qsTr("Buscando arquivos...");
                }
                if (root.truncated) {
                    return qsTr("Mostrando os primeiros resultados.");
                }
                if (searchInput.text.trim() === "") {
                    return qsTr("Digite para buscar arquivos por nome.");
                }
                return qsTr("%1 arquivos").arg(root.resultCount);
            }
            color: Theme.textMuted
            font.pixelSize: 10
        }

        ListView {
            id: resultsList

            width: parent.width
            height: parent.height - y
            clip: true
            model: root.resultsModel
            currentIndex: root.currentIndex

            delegate: Rectangle {
                required property int index
                required property string path
                required property string title
                required property string subtitle
                required property string kind

                width: resultsList.width
                height: 38
                radius: Theme.radius
                color: index === root.currentIndex
                       ? Theme.accentDim
                       : (resultArea.containsMouse
                          ? Theme.surface2 : "transparent")

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.spacingSmall
                    anchors.rightMargin: Theme.spacingSmall
                    spacing: 2

                    Text {
                        width: parent.width
                        text: title
                        color: Theme.textPrimary
                        font.pixelSize: 12
                        font.bold: true
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: subtitle !== "" ? subtitle : path
                        color: Theme.textMuted
                        font.family: Theme.monoFont
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                    }
                }

                MouseArea {
                    id: resultArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onEntered: root.resultHovered(parent.index)
                    onClicked: root.resultActivated(parent.index)
                }
            }
        }
    }
}
