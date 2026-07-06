import QtQuick

Rectangle {
    id: root

    property var messagesModel

    signal closeRequested()
    signal messageSubmitted(string body)

    implicitWidth: 300
    radius: Theme.radiusLarge
    color: Theme.background1
    border.color: Theme.borderSoft
    border.width: 1

    function submitMessage() {
        const body = assistantInput.text;
        if (body.trim() === "") {
            return;
        }
        root.messageSubmitted(body);
        assistantInput.text = "";
    }

    Column {
        anchors.fill: parent
        anchors.margins: Theme.spacingSmall
        spacing: Theme.spacingSmall

        Row {
            id: assistantHeader

            width: parent.width
            height: 26
            spacing: Theme.spacingSmall

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "✦"
                color: Theme.accent
                font.pixelSize: 13
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: qsTr("Assistente KW")
                color: Theme.textPrimary
                font.pixelSize: 13
                font.bold: true
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: assistantBadge.width + 10
                height: 16
                radius: 8
                color: Theme.surface2

                Text {
                    id: assistantBadge

                    anchors.centerIn: parent
                    text: qsTr("offline")
                    color: Theme.textMuted
                    font.pixelSize: 9
                    font.bold: true
                }
            }

            Item {
                width: parent.width - x - assistantClose.width
                height: 1
            }

            Rectangle {
                id: assistantClose

                anchors.verticalCenter: parent.verticalCenter
                width: 22
                height: 22
                radius: Theme.radius
                color: assistantCloseArea.containsMouse
                       ? Theme.surface2 : "transparent"

                Text {
                    anchors.centerIn: parent
                    text: "×"
                    color: Theme.textSecondary
                    font.pixelSize: 13
                }

                MouseArea {
                    id: assistantCloseArea

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.closeRequested()
                }
            }
        }

        ListView {
            id: assistantView

            width: parent.width
            height: parent.height - assistantHeader.height
                    - assistantInputBox.height - 2 * Theme.spacingSmall
            clip: true
            spacing: Theme.spacingSmall
            model: root.messagesModel
            onCountChanged: positionViewAtEnd()

            delegate: Rectangle {
                required property string role
                required property string body

                width: assistantView.width
                height: messageText.height + 2 * Theme.spacingSmall
                radius: Theme.radius
                color: role === "user" ? Theme.surface2 : Theme.surface1
                border.color: role === "user" ? Theme.accentDim : Theme.borderSoft
                border.width: 1

                Text {
                    id: messageText

                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.margins: Theme.spacingSmall
                    text: parent.body
                    color: parent.role === "user"
                           ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: 11
                    wrapMode: Text.WordWrap
                }
            }
        }

        Rectangle {
            id: assistantInputBox

            width: parent.width
            height: 34
            radius: Theme.radius
            color: Theme.background0
            border.color: assistantInput.activeFocus ? Theme.accent : Theme.borderSoft
            border.width: 1

            TextInput {
                id: assistantInput

                anchors.left: parent.left
                anchors.right: assistantSend.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.margins: Theme.spacingSmall
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.pixelSize: 11
                clip: true
                selectByMouse: true
                onAccepted: root.submitMessage()
            }

            Rectangle {
                id: assistantSend

                anchors.verticalCenter: parent.verticalCenter
                anchors.right: parent.right
                anchors.rightMargin: 4
                width: 26
                height: 26
                radius: Theme.radius
                color: assistantSendArea.pressed ? Theme.accentDim : Theme.accent

                Text {
                    anchors.centerIn: parent
                    text: "➤"
                    color: Theme.background0
                    font.pixelSize: 11
                    font.bold: true
                }

                MouseArea {
                    id: assistantSendArea

                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.submitMessage()
                }
            }
        }
    }
}
