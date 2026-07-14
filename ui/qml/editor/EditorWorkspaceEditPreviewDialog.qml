pragma ComponentBehavior: Bound
import QtQuick

Rectangle {
    id: root

    property string operationTitle: ""
    property var files: []
    property int editCount: 0
    property string errorText: ""
    property real maxAvailableWidth: 760
    property real maxAvailableHeight: 540

    signal applyRequested()
    signal cancelRequested()

    width: Math.min(760, maxAvailableWidth)
    height: Math.min(540, maxAvailableHeight)
    radius: Theme.radiusLarge
    color: Theme.background2
    border.color: root.errorText === "" ? Theme.accent : Theme.errorSoft
    border.width: 1
    focus: visible
    Keys.onEscapePressed: root.cancelRequested()

    Text {
        id: titleLabel

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingMedium
        text: root.operationTitle === "" ? qsTr("Revisar alteracoes") : root.operationTitle
        color: Theme.textPrimary
        font.pixelSize: 14
        font.bold: true
        elide: Text.ElideRight
    }

    Text {
        id: summaryLabel

        anchors.top: titleLabel.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: Theme.spacingXSmall
        anchors.leftMargin: Theme.spacingMedium
        anchors.rightMargin: Theme.spacingMedium
        text: qsTr("%1 arquivo(s), %2 edit(s). Nenhuma alteracao foi gravada ainda.")
              .arg(root.files.length).arg(root.editCount)
        color: Theme.textSecondary
        font.pixelSize: 11
    }

    ListView {
        id: fileList

        anchors.top: summaryLabel.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: errorLabel.visible ? errorLabel.top : actionRow.top
        anchors.margins: Theme.spacingMedium
        anchors.bottomMargin: Theme.spacingSmall
        spacing: Theme.spacingSmall
        clip: true
        model: root.files

        delegate: Rectangle {
            id: fileDelegate

            required property var modelData

            width: fileList.width
            height: 246
            radius: Theme.radius
            color: Theme.background1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: pathLabel

                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.margins: Theme.spacingSmall
                text: fileDelegate.modelData.path + "  ·  "
                      + qsTr("%1 edit(s)").arg(fileDelegate.modelData.edits)
                color: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
                elide: Text.ElideMiddle
            }

            Row {
                anchors.top: pathLabel.bottom
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.margins: Theme.spacingSmall
                anchors.topMargin: Theme.spacingXSmall
                spacing: Theme.spacingSmall

                PreviewPane {
                    width: (parent.width - parent.spacing) / 2
                    height: parent.height
                    heading: qsTr("Antes")
                    content: fileDelegate.modelData.beforeContent
                    headingColor: Theme.textSecondary
                }

                PreviewPane {
                    width: (parent.width - parent.spacing) / 2
                    height: parent.height
                    heading: qsTr("Depois")
                    content: fileDelegate.modelData.afterContent
                    headingColor: Theme.successSoft
                }
            }
        }
    }

    Text {
        id: errorLabel

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: actionRow.top
        anchors.leftMargin: Theme.spacingMedium
        anchors.rightMargin: Theme.spacingMedium
        anchors.bottomMargin: Theme.spacingSmall
        visible: root.errorText !== ""
        text: root.errorText
        color: Theme.errorSoft
        font.pixelSize: 10
        wrapMode: Text.WordWrap
    }

    Row {
        id: actionRow

        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.spacingMedium
        spacing: Theme.spacingSmall

        Rectangle {
            width: cancelText.width + 2 * Theme.spacingMedium
            height: 26
            radius: Theme.radius
            color: cancelArea.containsMouse ? Theme.surface2 : Theme.surface1
            border.color: Theme.borderSoft
            border.width: 1

            Text {
                id: cancelText

                anchors.centerIn: parent
                text: qsTr("Cancelar")
                color: Theme.textSecondary
                font.pixelSize: 11
            }

            MouseArea {
                id: cancelArea

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.cancelRequested()
            }
        }

        Rectangle {
            width: applyText.width + 2 * Theme.spacingMedium
            height: 26
            radius: Theme.radius
            color: applyArea.pressed ? Theme.accentDim : Theme.accent

            Text {
                id: applyText

                anchors.centerIn: parent
                text: qsTr("Aplicar alteracoes")
                color: Theme.background0
                font.pixelSize: 11
                font.bold: true
            }

            MouseArea {
                id: applyArea

                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: root.applyRequested()
            }
        }
    }

    component PreviewPane: Rectangle {
        id: previewPane

        property string heading: ""
        property string content: ""
        property color headingColor: Theme.textSecondary

        radius: Theme.radius
        color: Theme.background0
        border.color: Theme.borderSoft
        border.width: 1
        clip: true

        Text {
            id: paneHeading

            anchors.top: parent.top
            anchors.left: parent.left
            anchors.margins: Theme.spacingSmall
            text: parent.heading
            color: parent.headingColor
            font.pixelSize: 10
            font.bold: true
        }

        Flickable {
            anchors.top: paneHeading.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.spacingSmall
            contentWidth: previewText.paintedWidth
            contentHeight: previewText.paintedHeight
            boundsBehavior: Flickable.StopAtBounds
            clip: true

            TextEdit {
                id: previewText

                width: Math.max(parent.width, paintedWidth)
                text: previewPane.content
                readOnly: true
                selectByMouse: true
                color: Theme.textPrimary
                selectionColor: Theme.accentDim
                selectedTextColor: Theme.textPrimary
                font.family: Theme.monoFont
                font.pixelSize: 10
                wrapMode: TextEdit.NoWrap
            }
        }
    }
}
