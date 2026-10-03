pragma ComponentBehavior: Bound
import QtQuick

Column {
    id: root

    property int selectionCount: 1
    property bool pasteAvailable: false
    property bool menuFocused: false
    property int currentAction: 0

    signal actionRequested(int index)

    Repeater {
        model: 7

        delegate: Rectangle {
            id: fileAction
            required property int index
            readonly property int actionIndex: index + 6
            readonly property bool actionEnabled: index === 2
                ? root.pasteAvailable
                : index >= 5 ? root.selectionCount === 1
                : root.selectionCount >= 1 && root.selectionCount <= 128

            width: root.width
            height: 26
            radius: Theme.radius
            color: entryFileHover.containsMouse
                   || (root.menuFocused && root.currentAction === actionIndex)
                   ? Theme.surface2 : "transparent"

            Text {
                anchors.verticalCenter: parent.verticalCenter
                anchors.left: parent.left
                anchors.leftMargin: Theme.spacingSmall
                text: fileAction.index === 0 ? qsTr("Copiar")
                      : fileAction.index === 1 ? qsTr("Recortar")
                      : fileAction.index === 2
                        ? (root.pasteAvailable ? qsTr("Colar")
                           : qsTr("Colar: itens do projeto"))
                      : fileAction.index === 3 ? qsTr("Copiar caminho absoluto")
                      : fileAction.index === 4 ? qsTr("Copiar caminho relativo")
                      : fileAction.index === 5 ? qsTr("Abrir pasta no gerenciador")
                      : qsTr("Abrir terminal nesta pasta")
                color: fileAction.actionEnabled ? Theme.textPrimary : Theme.textMuted
                font.pixelSize: Theme.fontSizeBody
            }

            MouseArea {
                id: entryFileHover
                anchors.fill: parent
                hoverEnabled: true
                enabled: fileAction.actionEnabled
                cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: root.actionRequested(fileAction.actionIndex)
            }
        }
    }
}
