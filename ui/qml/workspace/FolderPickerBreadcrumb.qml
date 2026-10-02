pragma ComponentBehavior: Bound
import QtQuick
import KineinVectis

// O caminho como MIGALHAS (0.3.9): cada pasta ate' a atual e' clicavel, e o
// fim fica sempre a vista. Clicar no vazio, Ctrl+L ou digitar "/" ou "~" na
// lista troca pelo campo de texto; Enter navega, Esc volta as migalhas.
Rectangle {
    id: root

    property var controller

    // Enter ou Esc no campo: o foco volta para a lista (setas vivas).
    signal editingFinished()

    radius: Theme.radius
    color: Theme.background0
    border.color: pathField.activeFocus ? Theme.accent : Theme.borderSoft
    border.width: 1

    function beginEditing(seed) {
        root.controller.editingPath = true;
        pathField.text = seed !== undefined ? seed : root.controller.currentPath;
        pathField.forceActiveFocus();
        if (seed === undefined) pathField.selectAll();
    }

    MouseArea {
        anchors.fill: parent
        visible: !root.controller.editingPath
        cursorShape: Qt.IBeamCursor
        onClicked: root.beginEditing()
    }

    ListView {
        id: crumbList

        anchors.fill: parent
        anchors.leftMargin: Theme.spacingXSmall
        anchors.rightMargin: Theme.spacingXSmall
        visible: !root.controller.editingPath
        orientation: ListView.Horizontal
        interactive: contentWidth > width
        boundsBehavior: Flickable.StopAtBounds
        clip: true
        model: root.controller.crumbs
        onCountChanged: Qt.callLater(crumbList.positionViewAtEnd)
        onWidthChanged: Qt.callLater(crumbList.positionViewAtEnd)

        delegate: Row {
            id: crumb

            required property var modelData
            required property int index
            readonly property bool last: crumb.index === crumbList.count - 1

            height: crumbList.height

            Text {
                visible: crumb.index > 1
                anchors.verticalCenter: parent.verticalCenter
                text: "›"
                color: Theme.textMuted
                font.pixelSize: Theme.fontSizeLarge
                rightPadding: 2
                leftPadding: 2
            }

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: crumbText.width + 2 * Theme.spacingSmall
                height: parent.height - 2 * Theme.spacingXSmall
                radius: Theme.radiusXSmall
                color: crumbMouse.containsMouse ? Theme.surface2 : "transparent"

                Text {
                    id: crumbText

                    anchors.centerIn: parent
                    text: crumb.modelData.label
                    color: crumb.last ? Theme.textPrimary : Theme.textSecondary
                    font.pixelSize: Theme.fontSizeBody
                    font.bold: crumb.last
                }

                MouseArea {
                    id: crumbMouse

                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.controller.browsePath(crumb.modelData.path)
                }
            }
        }
    }

    // Caminho maior que a barra: o comeco esmaece em vez de cortar seco.
    Rectangle {
        anchors.left: crumbList.left
        anchors.top: crumbList.top
        anchors.bottom: crumbList.bottom
        anchors.topMargin: 1
        anchors.bottomMargin: 1
        width: 28
        visible: crumbList.visible && !crumbList.atXBeginning
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: Theme.background0 }
            GradientStop { position: 1.0; color: "transparent" }
        }
    }

    TextInput {
        id: pathField

        anchors.fill: parent
        anchors.leftMargin: Theme.spacingSmall
        anchors.rightMargin: Theme.spacingSmall
        visible: root.controller.editingPath
        verticalAlignment: TextInput.AlignVCenter
        color: Theme.textPrimary
        selectedTextColor: Theme.textPrimary
        selectionColor: Theme.accentDim
        font.family: Theme.monoFont
        font.pixelSize: Theme.fontSizeBody
        clip: true
        selectByMouse: true
        onAccepted: {
            const typed = text.trim();
            root.controller.browsePath(typed.startsWith("~")
                                       ? root.controller.homePath + typed.slice(1) : typed);
            root.editingFinished();
        }
        Keys.onEscapePressed: function(event) {
            root.controller.editingPath = false;
            event.accepted = true;
            root.editingFinished();
        }
        onActiveFocusChanged: if (!activeFocus) root.controller.editingPath = false
    }
}
